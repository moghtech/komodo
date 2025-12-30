use serde::Serialize;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as _};

use super::{auth, read};

pub fn serve_docs() -> Scalar<utoipa::openapi::OpenApi> {
  Scalar::with_url("/docs", KomodoApi::openapi())
    .custom_html(include_str!("scalar.html"))
}

#[derive(OpenApi)]
#[openapi(
  paths(
    // ======
    //  AUTH
    // ======
    auth::get_login_options,
    // ======
    //  READ
    // ======
    read::get_version,
  ),
  modifiers(&AddSecurityHeaders),
  security(
    ("api-key" = [], "api-secret" = []),
    ("jwt" = [])
  )
)]
struct KomodoApi;

#[derive(Debug, Serialize)]
struct AddSecurityHeaders;

impl utoipa::Modify for AddSecurityHeaders {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    let schema = openapi.components.get_or_insert_default();

    schema.add_security_schemes_from_iter([
      ("api-key", header_security_scheme("X-Api-Key")),
      ("api-secret", header_security_scheme("X-Api-Secret")),
      ("jwt", header_security_scheme("Authorization")),
    ]);
  }
}

fn header_security_scheme(
  header: &str,
) -> utoipa::openapi::security::SecurityScheme {
  utoipa::openapi::security::SecurityScheme::ApiKey(
    utoipa::openapi::security::ApiKey::Header(
      utoipa::openapi::security::ApiKeyValue::new(header),
    ),
  )
}
