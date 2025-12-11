use axum::{
  Router,
  http::{HeaderName, HeaderValue},
  routing::get,
};
use tower_http::{
  services::{ServeDir, ServeFile},
  set_header::SetResponseHeaderLayer,
};
use tower_sessions::{
  Expiry, MemoryStore, SessionManagerLayer, cookie::time::Duration,
};

use crate::{
  config::{core_config, core_host, cors_layer},
  ts_client,
};

pub mod auth;
pub mod execute;
pub mod read;
pub mod user;
pub mod write;

mod listener;
mod terminal;
mod ws;

#[derive(serde::Deserialize)]
struct Variant {
  variant: String,
}

pub fn app() -> Router {
  let config = core_config();

  // Setup static frontend services
  let frontend_path = &config.frontend_path;
  let frontend_index =
    ServeFile::new(format!("{frontend_path}/index.html"));
  let serve_frontend = ServeDir::new(frontend_path)
    .not_found_service(frontend_index.clone());

  Router::new()
    .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
    .nest("/auth", auth::router())
    .nest("/user", user::router())
    .nest("/read", read::router())
    .nest("/write", write::router())
    .nest("/execute", execute::router())
    .nest("/terminal", terminal::router())
    .nest("/listener", listener::router())
    .nest("/ws", ws::router())
    .nest("/client", ts_client::router())
    .fallback_service(serve_frontend)
    .layer(cors_layer())
    .layer(SetResponseHeaderLayer::overriding(
      HeaderName::from_static("x-content-type-options"),
      HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      HeaderName::from_static("x-frame-options"),
      HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      HeaderName::from_static("x-xss-protection"),
      HeaderValue::from_static("1; mode=block"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      HeaderName::from_static("referrer-policy"),
      HeaderValue::from_static("strict-origin-when-cross-origin"),
    ))
}

fn memory_session_layer(
  expiry: i64,
) -> SessionManagerLayer<MemoryStore> {
  let config = core_config();
  let mut layer = SessionManagerLayer::new(MemoryStore::default())
    .with_expiry(Expiry::OnInactivity(Duration::seconds(expiry)))
    .with_secure(config.host.starts_with("https://"));
  if let Some(domain) = core_host().and_then(|url| url.domain()) {
    layer = layer.with_domain(domain);
  }
  layer
}

pub const SESSION_KEY_USER_ID: &str = "user-id";
pub const SESSION_KEY_TOTP_LOGIN: &str = "totp-user-id";
pub const SESSION_KEY_TOTP_ENROLLMENT: &str = "totp-enrollment";
pub const SESSION_KEY_PASSKEY_LOGIN: &str = "passkey-user-id";
pub const SESSION_KEY_PASSKEY_ENROLLMENT: &str = "passkey-enrollment";
