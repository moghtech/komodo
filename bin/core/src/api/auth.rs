use std::{
  net::{IpAddr, SocketAddr},
  sync::OnceLock,
  time::Instant,
};

use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::{ConnectInfo, Path},
  http::HeaderMap,
  routing::post,
};
use data_encoding::BASE32_NOPAD;
use database::{
  bson::{doc, to_bson},
  mungos::by_id::update_one_by_id,
};
use derive_variants::{EnumVariants, ExtractVariant};
use komodo_client::{api::auth::*, entities::user::User};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{AddStatusCode, AddStatusCodeError, Json};
use tower_sessions::Session;
use typeshare::typeshare;
use uuid::Uuid;
use webauthn_rs::prelude::PasskeyAuthentication;

use crate::{
  api::{
    SESSION_KEY_PASSKEY_LOGIN, SESSION_KEY_TOTP_LOGIN,
    SESSION_KEY_USER_ID, memory_session_layer,
  },
  auth::{
    get_user_id_from_headers,
    github::{self, client::github_oauth_client},
    google::{self, client::google_oauth_client},
    oidc::{self, client::oidc_client},
    totp::make_totp,
  },
  config::core_config,
  helpers::query::get_user,
  state::{auth_rate_limiter, db_client, jwt_client, webauthn},
};

use super::Variant;

pub struct AuthArgs {
  pub headers: HeaderMap,
  /// Prefer extracting IP from headers.
  /// This IP will be the IP of reverse proxy itself.
  pub ip: IpAddr,
  /// Per-client session state
  pub session: Option<Session>,
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[args(AuthArgs)]
#[response(Response)]
#[error(serror::Error)]
#[variant_derive(Debug)]
#[serde(tag = "type", content = "params")]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
pub enum AuthRequest {
  GetLoginOptions(GetLoginOptions),
  SignUpLocalUser(SignUpLocalUser),
  LoginLocalUser(LoginLocalUser),
  ExchangeForJwt(ExchangeForJwt),
  CompleteTotpLogin(CompleteTotpLogin),
  CompletePasskeyLogin(CompletePasskeyLogin),
  GetUser(GetUser),
}

pub fn router() -> Router {
  let mut router = Router::new()
    .route("/", post(handler))
    .route("/{variant}", post(variant_handler));

  if core_config().local_auth {
    info!("🔑 Local Login Enabled");
  }

  if github_oauth_client().is_some() {
    info!("🔑 Github Login Enabled");
    router = router.nest("/github", github::router())
  }

  if google_oauth_client().is_some() {
    info!("🔑 Google Login Enabled");
    router = router.nest("/google", google::router())
  }

  if core_config().oidc_enabled {
    info!("🔑 OIDC Login Enabled");
    router = router.nest("/oidc", oidc::router())
  }

  router.layer(memory_session_layer(60))
}

async fn variant_handler(
  headers: HeaderMap,
  session: Session,
  info: ConnectInfo<SocketAddr>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<axum::response::Response> {
  let req: AuthRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(headers, session, info, Json(req)).await
}

async fn handler(
  headers: HeaderMap,
  session: Session,
  ConnectInfo(info): ConnectInfo<SocketAddr>,
  Json(request): Json<AuthRequest>,
) -> serror::Result<axum::response::Response> {
  let timer = Instant::now();
  let req_id = Uuid::new_v4();
  debug!(
    "/auth request {req_id} | METHOD: {:?}",
    request.extract_variant()
  );
  let res = request
    .resolve(&AuthArgs {
      headers,
      ip: info.ip(),
      session: Some(session),
    })
    .await;
  if let Err(e) = &res {
    debug!("/auth request {req_id} | error: {:#}", e.error);
  }
  let elapsed = timer.elapsed();
  debug!("/auth request {req_id} | resolve time: {elapsed:?}");
  res.map(|res| res.0)
}

fn login_options_reponse() -> &'static GetLoginOptionsResponse {
  static GET_LOGIN_OPTIONS_RESPONSE: OnceLock<
    GetLoginOptionsResponse,
  > = OnceLock::new();
  GET_LOGIN_OPTIONS_RESPONSE.get_or_init(|| {
    let config = core_config();
    GetLoginOptionsResponse {
      local: config.local_auth,
      github: github_oauth_client().is_some(),
      google: google_oauth_client().is_some(),
      oidc: oidc_client().load().is_some(),
      registration_disabled: config.disable_user_registration,
    }
  })
}

impl Resolve<AuthArgs> for GetLoginOptions {
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<GetLoginOptionsResponse> {
    Ok(*login_options_reponse())
  }
}

impl Resolve<AuthArgs> for ExchangeForJwt {
  async fn resolve(
    self,
    AuthArgs {
      headers,
      ip,
      session,
    }: &AuthArgs,
  ) -> serror::Result<ExchangeForJwtResponse> {
    async {
      let session = session.as_ref().context(
        "Method called in invalid context. This should not happen",
      )?;

      let user_id = session
      .remove::<String>(SESSION_KEY_USER_ID)
      .await
      .context("Internal session type error")?
      .context("Authentication steps must be completed before JWT can be retrieved")?;

      jwt_client().encode(user_id).map_err(Into::into)
    }
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      headers,
      Some(*ip),
    )
    .await
  }
}

impl Resolve<AuthArgs> for CompleteTotpLogin {
  async fn resolve(
    self,
    AuthArgs {
      headers,
      ip,
      session,
    }: &AuthArgs,
  ) -> serror::Result<CompleteTotpLoginResponse> {
    async {
      let session = session.as_ref().context(
        "Method called in invalid context. This should not happen",
      )?;

      let user_id = session
        .get::<String>(SESSION_KEY_TOTP_LOGIN)
        .await
        .context("Internal session type error")?
        .context(
          "Totp login has not been initiated for this session",
        )?;

      let user = get_user(&user_id)
        .await
        .status_code(StatusCode::UNAUTHORIZED)?;

      if user.totp.secret.is_empty() {
        return Err(
          anyhow!("User is not enrolled in totp")
            .status_code(StatusCode::BAD_REQUEST),
        );
      }

      let secret_bytes = BASE32_NOPAD
        .decode(user.totp.secret.as_bytes())
        .context("Failed to decode totp secret to bytes")?;

      let totp = make_totp(secret_bytes, None)?;

      let valid = totp
        .check_current(&self.code)
        .context("Failed to check TOTP code validity")?;

      if !valid {
        return Err(
          anyhow!("Invalid totp code")
            .status_code(StatusCode::UNAUTHORIZED),
        );
      }

      jwt_client().encode(user_id).map_err(Into::into)
    }
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      headers,
      Some(*ip),
    )
    .await
  }
}

impl Resolve<AuthArgs> for CompletePasskeyLogin {
  async fn resolve(
    self,
    AuthArgs {
      headers,
      ip,
      session,
    }: &AuthArgs,
  ) -> serror::Result<CompletePasskeyLoginResponse> {
    async {
      let session = session.as_ref().context(
        "Method called in invalid context. This should not happen",
      )?;

      let webauthn = webauthn().context(
        "No webauthn provider available, invalid KOMODO_HOST config",
      )?;

      let (user_id, server_state) = session
        .get::<(String, PasskeyAuthentication)>(
          SESSION_KEY_PASSKEY_LOGIN,
        )
        .await
        .context("Internal session type error")?
        .context(
          "Passkey login has not been initiated for this session",
        )?;

      // The result of this call must be used to
      // update the stored passkey info on database.
      let update = webauthn
        .finish_passkey_authentication(
          &self.credential,
          &server_state,
        )
        .context("Failed to validate passkey")?;

      let mut passkey = get_user(&user_id)
        .await?
        .passkey
        .passkey
        .context("Could not find passkey on database.")?;

      passkey.update_credential(&update);

      let passkey = to_bson(&passkey)
        .context("Failed to serialize passkey to BSON")?;

      let update = doc! { "$set": { "passkey.passkey": passkey } };

      let _ =
        update_one_by_id(&db_client().users, &user_id, update, None)
          .await
          .context(
            "Failed to update user passkey on database after login",
          )
          .inspect_err(|e| warn!("{e:#}"));

      jwt_client().encode(user_id).map_err(Into::into)
    }
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      headers,
      Some(*ip),
    )
    .await
  }
}

impl Resolve<AuthArgs> for GetUser {
  async fn resolve(
    self,
    AuthArgs {
      headers,
      ip,
      session: _,
    }: &AuthArgs,
  ) -> serror::Result<User> {
    async {
      let user_id = get_user_id_from_headers(headers)
        .await
        .status_code(StatusCode::UNAUTHORIZED)?;
      let mut user = get_user(&user_id)
        .await
        .status_code(StatusCode::UNAUTHORIZED)?;
      // Sanitize before sending to client.
      user.sanitize();
      Ok(user)
    }
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      headers,
      Some(*ip),
    )
    .await
  }
}
