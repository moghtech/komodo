use std::{
  net::{IpAddr, SocketAddr},
  sync::OnceLock,
  time::Instant,
};

use anyhow::{Context, anyhow};
use async_timing_util::unix_timestamp_ms;
use axum::{
  Router,
  extract::{ConnectInfo, Path},
  http::HeaderMap,
  routing::post,
};
use data_encoding::BASE32_NOPAD;
use derive_variants::{EnumVariants, ExtractVariant};
use komodo_client::{api::auth::*, entities::user::User};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{AddStatusCode, AddStatusCodeError, Json};
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::{
    get_user_id_from_headers,
    github::{self, client::github_oauth_client},
    google::{self, client::google_oauth_client},
    oidc::{self, client::oidc_client},
    totp::make_totp,
  },
  config::core_config,
  helpers::query::get_user,
  state::{auth_rate_limiter, jwt_client, totp_pending_login_cache},
};

use super::Variant;

pub struct AuthArgs {
  pub headers: HeaderMap,
  /// Prefer extracting IP from headers.
  /// This IP will be the IP of reverse proxy itself.
  pub ip: IpAddr,
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

  router
}

async fn variant_handler(
  headers: HeaderMap,
  info: ConnectInfo<SocketAddr>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<axum::response::Response> {
  let req: AuthRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(headers, info, Json(req)).await
}

async fn handler(
  headers: HeaderMap,
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
    AuthArgs { headers, ip }: &AuthArgs,
  ) -> serror::Result<ExchangeForJwtResponse> {
    jwt_client()
      .redeem_exchange_token(&self.token)
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
    AuthArgs { headers, ip }: &AuthArgs,
  ) -> serror::Result<CompleteTotpLoginResponse> {
    async {
      let (user_id, expiry) = totp_pending_login_cache()
        .remove(&self.token)
        .await
        .context("Did not find any matching pending totp tokens")
        .status_code(StatusCode::UNAUTHORIZED)?;

      if unix_timestamp_ms() > expiry {
        return Err(
          anyhow!("Totp login flow has expired")
            .status_code(StatusCode::UNAUTHORIZED),
        );
      }

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

impl Resolve<AuthArgs> for GetUser {
  async fn resolve(
    self,
    AuthArgs { headers, ip }: &AuthArgs,
  ) -> serror::Result<User> {
    async {
      let user_id = get_user_id_from_headers(headers)
        .await
        .status_code(StatusCode::UNAUTHORIZED)?;
      get_user(&user_id)
        .await
        .status_code(StatusCode::UNAUTHORIZED)
    }
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      headers,
      Some(*ip),
    )
    .await
  }
}
