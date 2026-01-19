use komodo_client::openapi::KomodoApi;
use utoipa::OpenApi as _;
use utoipa_scalar::{Scalar, Servable as _};

pub fn serve_docs() -> Scalar<utoipa::openapi::OpenApi> {
  Scalar::with_url("/docs", KomodoApi::openapi())
    .custom_html(include_str!("docs.html"))
}
