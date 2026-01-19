use mogh_auth_server::api::openapi::MoghAuthApi;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as _};

mod read {
  pub use crate::api::read::*;
}

#[derive(OpenApi)]
#[openapi(
  nest(
    (path = "/auth", api = MoghAuthApi)
  ),
  paths(
    // ======
    //  AUTH
    // ======
    // auth::get_login_options,
    // ======
    //  READ
    // ======
    read::get_version,
  ),
)]
struct KomodoApi;

pub fn serve_docs() -> Scalar<utoipa::openapi::OpenApi> {
  Scalar::with_url("/docs", KomodoApi::openapi())
    .custom_html(include_str!("docs.html"))
}
