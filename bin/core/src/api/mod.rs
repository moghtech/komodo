use axum::{Extension, Router, routing::get};
use komodo_client::entities::user::User;
use mogh_auth_server::middleware::authenticate_request;
use mogh_error::Json;
use mogh_server::{
  cors::cors_layer, session::memory_session_layer,
  ui::serve_static_ui,
};

use crate::{auth::KomodoAuthImpl, config::core_config, ts_client};

pub mod execute;
pub mod read;
pub mod write;

mod listener;
mod openapi;
mod terminal;
mod ws;

#[derive(serde::Deserialize)]
struct Variant {
  variant: String,
}

pub fn app() -> Router {
  let config = core_config();
  Router::new()
    .merge(openapi::serve_docs())
    .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
    .nest("/auth", mogh_auth_server::api::router::<KomodoAuthImpl>())
    .nest("/user", user_router())
    .nest("/read", read::router())
    .nest("/write", write::router())
    .nest("/execute", execute::router())
    .nest("/terminal", terminal::router())
    .nest("/listener", listener::router())
    .nest("/ws", ws::router())
    .nest("/client", ts_client::router())
    .layer(memory_session_layer(config))
    .fallback_service(serve_static_ui(
      &config.frontend_path,
      config.ui_index_force_no_cache,
    ))
    .layer(cors_layer(config))
}

fn user_router() -> Router {
  Router::new()
    .route(
      "/",
      get(|Extension(user): Extension<User>| async { Json(user) }),
    )
    .layer(axum::middleware::from_fn(
      authenticate_request::<KomodoAuthImpl, false>,
    ))
}
