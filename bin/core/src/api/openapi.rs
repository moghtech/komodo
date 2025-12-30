use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as _};

use super::read;

#[derive(OpenApi)]
#[openapi(paths(
  // ======
  //  READ
  // ======
  read::get_version,
))]
struct KomodoApi;

pub fn serve_docs() -> Scalar<utoipa::openapi::OpenApi> {
  Scalar::with_url("/openapi", KomodoApi::openapi())
}
