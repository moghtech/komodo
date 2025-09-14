use std::{sync::OnceLock, time::Instant};

use axum::{Router, extract::Path, http::HeaderMap, routing::post};
use derive_variants::{EnumVariants, ExtractVariant};
use komodo_client::{api::auth::*, entities::user::User};
use resolver_api::Resolve;
use response::Response;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{Json, AddStatusCode};
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::{
    get_user_id_from_headers,
    github::{self, client::github_oauth_client},
    google::{self, client::google_oauth_client},
    oidc::{self, client::oidc_client},
  },
  config::core_config,
  helpers::query::get_user,
  state::jwt_client,
};

use super::Variant;

#[derive(Default)]
pub struct AuthArgs {
  pub headers: HeaderMap,
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
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<axum::response::Response> {
  let req: AuthRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(headers, Json(req)).await
}

#[instrument(name = "AuthHandler", level = "debug", skip(headers))]
async fn handler(
  headers: HeaderMap,
  Json(request): Json<AuthRequest>,
) -> serror::Result<axum::response::Response> {
  let timer = Instant::now();
  let req_id = Uuid::new_v4();
  debug!(
    "/auth request {req_id} | METHOD: {:?}",
    request.extract_variant()
  );
  let res = request.resolve(&AuthArgs { headers }).await;
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
  #[instrument(name = "GetLoginOptions", level = "debug", skip(self))]
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<GetLoginOptionsResponse> {
    Ok(*login_options_reponse())
  }
}

impl Resolve<AuthArgs> for ExchangeForJwt {
  #[instrument(name = "ExchangeForJwt", level = "debug", skip(self))]
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<ExchangeForJwtResponse> {
    jwt_client()
      .redeem_exchange_token(&self.token)
      .await
      .map_err(Into::into)
  }
}

impl Resolve<AuthArgs> for GetUser {
  #[instrument(name = "GetUser", level = "debug", skip(self))]
  async fn resolve(
    self,
    AuthArgs { headers }: &AuthArgs,
  ) -> serror::Result<User> {
    let user_id = get_user_id_from_headers(headers).await.status_code(StatusCode::UNAUTHORIZED)?;
    Ok(get_user(&user_id).await?)
  }
}
