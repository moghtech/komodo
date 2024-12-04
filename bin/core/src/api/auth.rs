use std::{sync::OnceLock, time::Instant};

use axum::{http::HeaderMap, routing::post, Router};
use derive_variants::{EnumVariants, ExtractVariant};
use komodo_client::{api::auth::*, entities::user::User};
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serror::Json;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::{
    get_user_id_from_headers,
    github::{self, client::github_oauth_client},
    google::{self, client::google_oauth_client},
    oidc,
  },
  config::core_config,
  helpers::query::get_user,
  state::jwt_client,
};

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
  CreateLocalUser(CreateLocalUser),
  LoginLocalUser(LoginLocalUser),
  ExchangeForJwt(ExchangeForJwt),
  GetUser(GetUser),
}

pub fn router() -> Router {
  let mut router = Router::new().route("/", post(handler));

  if core_config().local_auth {
    info!("🔑 Local Login Enabled");
  }

  if github_oauth_client().is_some() {
    info!("🔑 Github Login Enabled");
    router = router.nest("/github", github::router())
  }

  if google_oauth_client().is_some() {
    info!("🔑 Github Login Enabled");
    router = router.nest("/google", google::router())
  }

  if core_config().oidc_enabled {
    info!("🔑 OIDC Login Enabled");
    router = router.nest("/oidc", oidc::router())
  }

  router
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
      github: config.github_oauth.enabled
        && !config.github_oauth.id.is_empty()
        && !config.github_oauth.secret.is_empty(),
      google: config.google_oauth.enabled
        && !config.google_oauth.id.is_empty()
        && !config.google_oauth.secret.is_empty(),
      oidc: config.oidc_enabled
        && !config.oidc_provider.is_empty()
        && !config.oidc_client_id.is_empty()
        && !config.oidc_client_secret.is_empty(),
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
    let jwt = jwt_client().redeem_exchange_token(&self.token).await?;
    Ok(ExchangeForJwtResponse { jwt })
  }
}

impl Resolve<AuthArgs> for GetUser {
  #[instrument(name = "GetUser", level = "debug", skip(self))]
  async fn resolve(
    self,
    AuthArgs { headers }: &AuthArgs,
  ) -> serror::Result<User> {
    let user_id = get_user_id_from_headers(headers).await?;
    Ok(get_user(&user_id).await?)
  }
}
