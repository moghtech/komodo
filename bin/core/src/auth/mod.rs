use std::sync::{Arc, LazyLock};

use anyhow::{Context as _, anyhow};
use async_timing_util::{
  Timelength, get_timelength_in_ms, unix_timestamp_ms,
};
use database::{
  bson::{Document, doc, to_bson},
  mungos::by_id::update_one_by_id,
};
use komodo_client::entities::{
  komodo_timestamp, optional_str,
  user::{NewUserParams, User, UserConfig, UserConfigVariant},
};
use mogh_auth_client::{
  api::login::LoginProvider,
  config::{NamedOauthConfig, OidcConfig},
  passkey::Passkey,
};
use mogh_auth_server::{
  AuthImpl, RequestClientArgs,
  provider::{
    jwt::JwtProvider, oidc::SubjectIdentifier,
    passkey::PasskeyProvider,
  },
  rand::random_string,
  user::{AuthUserImpl, BoxAuthUser},
  validations::{validate_password, validate_username},
};
use rate_limit::RateLimiter;
use serror::{AddStatusCode, AddStatusCodeError, StatusCode};

use crate::{
  config::core_config,
  helpers::query::{
    find_github_user, find_google_user, find_oidc_user, get_user,
  },
  state::db_client,
};

mod middleware;

pub use middleware::*;

/// Api Key Clock skew tolerance in milliseconds (5 minutes for Api Keys)
const API_KEY_CLOCK_SKEW_TOLERANCE_MS: i64 = 5 * 60 * 1000;

pub static JWT_PROVIDER: LazyLock<JwtProvider> =
  LazyLock::new(|| {
    let config = core_config();
    let secret = if config.jwt_secret.is_empty() {
      random_string(40)
    } else {
      config.jwt_secret.clone()
    };
    JwtProvider::new(
    secret.as_bytes(),
    get_timelength_in_ms(
      config.jwt_ttl.to_string().parse().unwrap_or_else(|e| {
        warn!(
          "Failed to parse 'jwt_ttl' | Using default of 1-day | {e:?}"
        );
        Timelength::OneDay
      }),
    ),
  )
  });

pub static GENERAL_RATE_LIMITER: LazyLock<Arc<RateLimiter>> =
  LazyLock::new(|| {
    let config = core_config();
    RateLimiter::new(
      config.auth_rate_limit_disabled,
      config.auth_rate_limit_max_attempts as usize,
      config.auth_rate_limit_window_seconds,
    )
  });

static LOCAL_LOGIN_RATE_LIMITER: LazyLock<Arc<RateLimiter>> =
  LazyLock::new(|| {
    let config = core_config();
    RateLimiter::new(
      config.auth_rate_limit_disabled,
      config.auth_rate_limit_max_attempts as usize,
      config.auth_rate_limit_window_seconds,
    )
  });

pub struct AuthUser(User);

impl AuthUserImpl for AuthUser {
  fn id(&self) -> &str {
    &self.0.id
  }

  fn username(&self) -> &str {
    &self.0.username
  }

  fn hashed_password(&self) -> Option<&str> {
    if let UserConfig::Local { password } = &self.0.config {
      optional_str(password)
    } else if let Some(UserConfig::Local { password }) =
      self.0.linked_logins.get(UserConfigVariant::Local)
    {
      optional_str(password)
    } else {
      None
    }
  }

  fn passkey(&self) -> Option<Passkey> {
    let passkey = self.0.passkey.passkey.as_ref()?;
    serde_json::from_str(&serde_json::to_string(passkey).ok()?)
      .inspect_err(|e| {
        warn!(
          "User {} ({}) | Invalid passkey on database | {e:?}",
          self.username(),
          self.id(),
        )
      })
      .ok()
  }

  fn totp_secret(&self) -> Option<&str> {
    optional_str(&self.0.totp.secret)
  }

  fn external_skip_2fa(&self) -> bool {
    self.0.external_skip_2fa
  }
}

pub struct KomodoAuthImpl {
  client: RequestClientArgs,
}

impl AuthImpl for KomodoAuthImpl {
  fn from_client(client: RequestClientArgs) -> Self
  where
    Self: Sized,
  {
    Self { client }
  }

  fn client(&self) -> &RequestClientArgs {
    &self.client
  }

  fn app_name(&self) -> &'static str {
    "Komodo"
  }

  fn host(&self) -> &str {
    static AUTH_HOST: LazyLock<String> =
      LazyLock::new(|| format!("{}/auth", core_config().host));
    &AUTH_HOST
  }

  fn post_link_redirect(&self) -> &str {
    static POST_LINK_REDIRECT: LazyLock<String> =
      LazyLock::new(|| format!("{}/settings", core_config().host));
    &POST_LINK_REDIRECT
  }

  fn get_user(
    &self,
    user_id: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<BoxAuthUser>> {
    Box::pin(async move {
      Ok(Box::new(AuthUser(
        get_user(&user_id)
          .await
          .status_code(StatusCode::NOT_FOUND)?,
      )) as BoxAuthUser)
    })
  }

  fn no_users_exist(
    &self,
  ) -> mogh_auth_server::DynFuture<serror::Result<bool>> {
    Box::pin(async {
      Ok(db_client().users.find_one(Document::new()).await?.is_none())
    })
  }

  fn locked_usernames(&self) -> &'static [String] {
    &core_config().lock_login_credentials_for
  }

  fn registration_disabled(&self) -> bool {
    core_config().disable_user_registration
  }

  fn validate_username(&self, username: &str) -> serror::Result<()> {
    validate_username(username).status_code(StatusCode::BAD_REQUEST)
  }

  // =========
  // = STATE =
  // =========

  fn jwt_provider(&self) -> &JwtProvider {
    &JWT_PROVIDER
  }

  fn passkey_provider(&self) -> Option<&PasskeyProvider> {
    static PASSKEY_PROVIDER: LazyLock<Option<PasskeyProvider>> =
      LazyLock::new(|| {
        PasskeyProvider::new(&core_config().host)
          .inspect_err(|e| {
            warn!("Invalid 'host' for passkey provider | {e:#}")
          })
          .ok()
      });
    PASSKEY_PROVIDER.as_ref()
  }

  fn general_rate_limiter(&self) -> &RateLimiter {
    &GENERAL_RATE_LIMITER
  }

  // ==============
  // = LOCAL AUTH =
  // ==============

  fn local_auth_enabled(&self) -> bool {
    core_config().local_auth
  }

  fn local_login_rate_limiter(&self) -> &RateLimiter {
    &LOCAL_LOGIN_RATE_LIMITER
  }

  fn sign_up_local_user(
    &self,
    username: String,
    hashed_password: String,
    no_users_exist: bool,
  ) -> mogh_auth_server::DynFuture<serror::Result<String>> {
    Box::pin(async move {
      let user = User::new(NewUserParams {
        username,
        enabled: no_users_exist || core_config().enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        config: UserConfig::Local {
          password: hashed_password,
        },
        updated_at: unix_timestamp_ms() as i64,
      });
      let user_id = db_client()
        .users
        .insert_one(user)
        .await
        .context("Failed to create user on database")?
        .inserted_id
        .as_object_id()
        .context("The 'inserted_id' is not ObjectId")?
        .to_string();
      Ok(user_id)
    })
  }

  fn find_user_with_username(
    &self,
    username: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<Option<BoxAuthUser>>>
  {
    Box::pin(async move {
      let user = db_client()
        .users
        .find_one(doc! { "username": &username })
        .await?
        .map(|user| Box::new(AuthUser(user)) as BoxAuthUser);
      Ok(user)
    })
  }

  fn update_user_username(
    &self,
    user_id: String,
    username: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let update = doc! { "$set": { "username": username } };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context("Failed to update user username on database.")?;

      Ok(())
    })
  }

  fn validate_password(&self, password: &str) -> serror::Result<()> {
    validate_password(password).status_code(StatusCode::BAD_REQUEST)
  }

  fn update_user_password(
    &self,
    user_id: String,
    hashed_password: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let user = get_user(&user_id).await?;
      db_client()
        .set_user_hashed_password(&user, hashed_password)
        .await?;
      Ok(())
    })
  }

  // =============
  // = OIDC AUTH =
  // =============

  fn oidc_config(&self) -> &OidcConfig {
    static OIDC_CONFIG: LazyLock<OidcConfig> = LazyLock::new(|| {
      let config = core_config();
      OidcConfig {
        enabled: config.oidc_enabled,
        provider: config.oidc_provider.clone(),
        redirect_host: config.oidc_redirect_host.clone(),
        client_id: config.oidc_client_id.clone(),
        client_secret: config.oidc_client_secret.clone(),
        use_full_email: config.oidc_use_full_email,
        additional_audiences: config
          .oidc_additional_audiences
          .clone(),
      }
    });
    &OIDC_CONFIG
  }

  fn find_user_with_oidc_subject(
    &self,
    subject: SubjectIdentifier,
  ) -> mogh_auth_server::DynFuture<serror::Result<Option<BoxAuthUser>>>
  {
    Box::pin(async move {
      let user = find_oidc_user(&subject)
        .await
        .status_code(StatusCode::NOT_FOUND)?
        .map(|user| Box::new(AuthUser(user)) as BoxAuthUser);
      Ok(user)
    })
  }

  fn sign_up_oidc_user(
    &self,
    username: String,
    subject: SubjectIdentifier,
    no_users_exist: bool,
  ) -> mogh_auth_server::DynFuture<serror::Result<String>> {
    Box::pin(async move {
      let user = User::new(NewUserParams {
        username,
        enabled: no_users_exist || core_config().enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        config: UserConfig::Oidc {
          provider: core_config().oidc_provider.clone(),
          user_id: subject.to_string(),
        },
        updated_at: komodo_timestamp(),
      });

      let user_id = db_client()
        .users
        .insert_one(user)
        .await
        .context("failed to create user on database")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();

      Ok(user_id)
    })
  }

  fn link_oidc_login(
    &self,
    user_id: String,
    subject: SubjectIdentifier,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let user = get_user(&user_id).await?;

      if let UserConfig::Oidc { .. } = &user.config {
        return Err(anyhow!(
          "User is primary Oidc user, cannot link another Oidc login."
        ).status_code(StatusCode::UNAUTHORIZED));
      }

      let oidc_provider = &core_config().oidc_provider;
      let oidc_user_id = subject.as_str();

      let update = doc! {
        "$set": {
          "linked_logins.Oidc.type": "Oidc",
          "linked_logins.Oidc.data.provider": oidc_provider,
          "linked_logins.Oidc.data.user_id": oidc_user_id,
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context(
          "Failed to link OIDC login to existing user on database",
        )?;

      Ok(())
    })
  }

  // ===============
  // = GITHUB AUTH =
  // ===============

  fn github_config(&self) -> &NamedOauthConfig {
    &core_config().github_oauth
  }

  fn find_user_with_github_id(
    &self,
    github_id: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<Option<BoxAuthUser>>>
  {
    Box::pin(async move {
      Ok(
        find_github_user(&github_id)
          .await?
          .map(|user| Box::new(AuthUser(user)) as BoxAuthUser),
      )
    })
  }

  fn sign_up_github_user(
    &self,
    username: String,
    github_id: String,
    avatar_url: String,
    no_users_exist: bool,
  ) -> mogh_auth_server::DynFuture<serror::Result<String>> {
    Box::pin(async move {
      let user = User::new(NewUserParams {
        username,
        enabled: no_users_exist || core_config().enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        config: UserConfig::Github {
          github_id,
          avatar: avatar_url,
        },
        updated_at: komodo_timestamp(),
      });

      let user_id = db_client()
        .users
        .insert_one(user)
        .await
        .context("Failed to create user on mongo")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();

      Ok(user_id)
    })
  }

  fn link_github_login(
    &self,
    user_id: String,
    github_id: String,
    avatar_url: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let user = get_user(&user_id).await?;

      if let UserConfig::Github { .. } = &user.config {
        return Err(anyhow!(
          "User is primary Github user, cannot link another Github login."
        ).status_code(StatusCode::UNAUTHORIZED));
      }

      let update = doc! {
        "$set": {
          "linked_logins.Github.type": "Github",
          "linked_logins.Github.data.github_id": &github_id,
          "linked_logins.Github.data.avatar": &avatar_url,
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context(
          "Failed to link Github login to existing user on database",
        )?;

      Ok(())
    })
  }

  // ===============
  // = GOOGLE AUTH =
  // ===============

  fn google_config(&self) -> &NamedOauthConfig {
    &core_config().google_oauth
  }

  fn find_user_with_google_id(
    &self,
    google_id: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<Option<BoxAuthUser>>>
  {
    Box::pin(async move {
      Ok(
        find_google_user(&google_id)
          .await?
          .map(|user| Box::new(AuthUser(user)) as BoxAuthUser),
      )
    })
  }

  fn sign_up_google_user(
    &self,
    username: String,
    google_id: String,
    avatar_url: String,
    no_users_exist: bool,
  ) -> mogh_auth_server::DynFuture<serror::Result<String>> {
    Box::pin(async move {
      let user = User::new(NewUserParams {
        username,
        enabled: no_users_exist || core_config().enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        config: UserConfig::Google {
          google_id,
          avatar: avatar_url,
        },
        updated_at: komodo_timestamp(),
      });

      let user_id = db_client()
        .users
        .insert_one(user)
        .await
        .context("Failed to create user on mongo")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();

      Ok(user_id)
    })
  }

  fn link_google_login(
    &self,
    user_id: String,
    google_id: String,
    avatar_url: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let user = get_user(&user_id).await?;

      if let UserConfig::Google { .. } = &user.config {
        return Err(anyhow!(
          "User is primary Google user, cannot link another Google login."
        ).status_code(StatusCode::UNAUTHORIZED));
      }

      let update = doc! {
        "$set": {
          "linked_logins.Google.type": "Google",
          "linked_logins.Google.data.google_id": &google_id,
          "linked_logins.Google.data.avatar": &avatar_url,
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context(
          "Failed to link Google login to existing user on database",
        )?;

      Ok(())
    })
  }

  // ==========
  // = UNLINK =
  // ==========

  fn unlink_login(
    &self,
    user_id: String,
    provider: LoginProvider,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let field = format!("linked_logins.{provider}");

      let update = doc! {
        "$unset": {
          field: ""
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context("Failed to unlink third partly login on database")?;

      Ok(())
    })
  }

  // ===============
  // = PASSKEY 2FA =
  // ===============

  fn update_user_stored_passkey(
    &self,
    user_id: String,
    passkey: Option<Passkey>,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let update = if let Some(passkey) = passkey {
        let passkey = to_bson(&passkey)
          .context("Failed to serialize passkey to BSON")?;
        doc! {
          "$set": {
            "passkey.passkey": passkey,
            "passkey.created_at": komodo_timestamp()
          }
        }
      } else {
        doc! {
          "$set": {
            "passkey.passkey": null,
            "passkey.created_at": 0
          }
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context(
          "Failed to update user passkey options on database",
        )?;

      Ok(())
    })
  }

  // ============
  // = TOTP 2FA =
  // ============

  fn update_user_stored_totp(
    &self,
    user_id: String,
    totp_secret: String,
    hashed_recovery_codes: Vec<String>,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      update_one_by_id(
        &db_client().users,
        &user_id,
        doc! {
          "$set": {
            "totp.secret": totp_secret,
            "totp.confirmed_at": komodo_timestamp(),
            "totp.recovery_codes": hashed_recovery_codes,
          }
        },
        None,
      )
      .await
      .context("Failed to update user totp fields on database")?;
      Ok(())
    })
  }

  fn remove_user_stored_totp(
    &self,
    user_id: String,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      update_one_by_id(
        &db_client().users,
        &user_id,
        doc! {
          "$set": {
            "totp.secret": "",
            "totp.confirmed_at": 0,
            "totp.recovery_codes": [],
          }
        },
        None,
      )
      .await
      .context("Failed to clear user totp fields on database")?;
      Ok(())
    })
  }

  // ============
  // = SKIP 2FA =
  // ============
  fn update_user_external_skip_2fa(
    &self,
    user_id: String,
    external_skip_2fa: bool,
  ) -> mogh_auth_server::DynFuture<serror::Result<()>> {
    Box::pin(async move {
      let update = doc! {
        "$set": {
          "external_skip_2fa": external_skip_2fa
        }
      };

      update_one_by_id(&db_client().users, &user_id, update, None)
        .await
        .context(
          "Failed to set skip external login 2fa mode on database",
        )?;

      Ok(())
    })
  }
}
