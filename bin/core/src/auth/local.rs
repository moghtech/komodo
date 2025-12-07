use std::sync::{Arc, OnceLock};

use anyhow::{Context, anyhow};
use async_timing_util::{ONE_MIN_MS, unix_timestamp_ms};
use database::{
  hash_password,
  mungos::mongodb::bson::{Document, doc},
};
use komodo_client::{
  api::auth::{
    LoginLocalUser, LoginLocalUserResponse, SignUpLocalUser,
    SignUpLocalUserResponse,
  },
  entities::{
    random_string,
    user::{User, UserConfig},
  },
};
use rate_limit::{RateLimiter, WithFailureRateLimit};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::{AddStatusCode as _, AddStatusCodeError};

use crate::{
  api::auth::AuthArgs,
  config::core_config,
  helpers::validations::{validate_password, validate_username},
  state::{
    auth_rate_limiter, db_client, jwt_client,
    totp_pending_login_cache,
  },
};

impl Resolve<AuthArgs> for SignUpLocalUser {
  #[instrument("SignUpLocalUser", skip(self))]
  async fn resolve(
    self,
    AuthArgs { headers, ip }: &AuthArgs,
  ) -> serror::Result<SignUpLocalUserResponse> {
    sign_up_local_user(self)
      .with_failure_rate_limit_using_headers(
        auth_rate_limiter(),
        headers,
        Some(*ip),
      )
      .await
  }
}

async fn sign_up_local_user(
  req: SignUpLocalUser,
) -> serror::Result<SignUpLocalUserResponse> {
  let config = core_config();

  if !config.local_auth {
    return Err(anyhow!("Local auth is not enabled").into());
  }

  validate_username(&req.username)
    .status_code(StatusCode::BAD_REQUEST)?;
  validate_password(&req.password)
    .status_code(StatusCode::BAD_REQUEST)?;

  let db = db_client();

  let no_users_exist =
    db.users.find_one(Document::new()).await?.is_none();

  if !no_users_exist && config.disable_user_registration {
    return Err(
      anyhow!("User registration is disabled")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  }

  if db
    .users
    .find_one(doc! { "username": &req.username })
    .await
    .context("Failed to query for existing users")?
    .is_some()
  {
    // When user registration is enabled, there is no way around allowing
    // potential attackers to gain some insight about which usernames exist
    // if they are allowed to register accounts. Since this can be easily inferred,
    // might as well be clear. The auth rate limiter is critical here.
    return Err(
      anyhow!("Username already taken.")
        .status_code(StatusCode::BAD_REQUEST),
    );
  }

  let ts = unix_timestamp_ms() as i64;
  let hashed_password = hash_password(req.password)?;

  let user = User {
    id: Default::default(),
    username: req.username,
    enabled: no_users_exist || config.enable_new_users,
    admin: no_users_exist,
    super_admin: no_users_exist,
    create_server_permissions: no_users_exist,
    create_build_permissions: no_users_exist,
    updated_at: ts,
    last_update_view: 0,
    recents: Default::default(),
    all: Default::default(),
    config: UserConfig::Local {
      password: hashed_password,
    },
    totp: Default::default(),
    webauthn: Default::default(),
  };

  let user_id = db_client()
    .users
    .insert_one(user)
    .await
    .context("Failed to create user on database")?
    .inserted_id
    .as_object_id()
    .context("The 'inserted_id' is not ObjectId")?
    .to_string();

  jwt_client()
    .encode(user_id)
    .context("Failed to generate JWT for user")
    .map_err(Into::into)
}

/// Local login method has a dedicated rate limiter
/// so the UI background calls using existing JWT do
/// not influence the number of attempts user has
/// to log in.
fn login_local_user_rate_limiter() -> &'static RateLimiter {
  static LOGIN_LOCAL_USER_RATE_LIMITER: OnceLock<Arc<RateLimiter>> =
    OnceLock::new();
  LOGIN_LOCAL_USER_RATE_LIMITER.get_or_init(|| {
    let config = core_config();
    RateLimiter::new(
      config.auth_rate_limit_disabled,
      config.auth_rate_limit_max_attempts as usize,
      config.auth_rate_limit_window_seconds,
    )
  })
}

impl Resolve<AuthArgs> for LoginLocalUser {
  async fn resolve(
    self,
    AuthArgs { headers, ip }: &AuthArgs,
  ) -> serror::Result<LoginLocalUserResponse> {
    login_local_user(self)
      .with_failure_rate_limit_using_headers(
        login_local_user_rate_limiter(),
        headers,
        Some(*ip),
      )
      .await
  }
}

async fn login_local_user(
  req: LoginLocalUser,
) -> serror::Result<LoginLocalUserResponse> {
  if !core_config().local_auth {
    return Err(
      anyhow!("Local auth is not enabled")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  }

  validate_username(&req.username)
    .status_code(StatusCode::BAD_REQUEST)?;

  let user = db_client()
    .users
    .find_one(doc! { "username": &req.username })
    .await
    .context("Failed at db query for users")?
    .context("Invalid login credentials")
    .status_code(StatusCode::UNAUTHORIZED)?;

  let UserConfig::Local {
    password: user_pw_hash,
  } = user.config
  else {
    return Err(
      anyhow!("Invalid login credentials")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  };

  let verified = bcrypt::verify(req.password, &user_pw_hash)
    .context("Invalid login credentials")
    .status_code(StatusCode::UNAUTHORIZED)?;

  if !verified {
    return Err(
      anyhow!("Invalid login credentials")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  }

  if user.totp.secret.is_empty() {
    jwt_client()
      .encode(user.id)
      // This is in internal error (500), not auth error
      .context("Failed to generate JWT for user")
      .map(|jwt| LoginLocalUserResponse::Jwt(jwt))
      .map_err(Into::into)
  } else {
    let token = random_string(20);
    let expiry = unix_timestamp_ms() + ONE_MIN_MS;
    totp_pending_login_cache()
      .insert(token.clone(), (user.id, expiry))
      .await;
    Ok(LoginLocalUserResponse::TwoFactor { token })
  }
}
