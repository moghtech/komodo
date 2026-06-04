use axum::{
  Extension, Router,
  body::{Body, to_bytes},
  extract::State,
  http::header,
  middleware::{self, Next},
  response::Response,
  routing::get,
};
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

const BASE_PATH_PLACEHOLDER: &str = "{{KOMODO_HOST}}";

#[derive(Clone)]
struct UiPathConfig {
  base_path: String,
}

#[derive(serde::Deserialize)]
struct Variant {
  variant: String,
}

pub fn app() -> Router {
  let config = core_config();
  let base_url = config.host.clone();
  let base_path = normalize_base_path(&base_url);
  let ui_path_config = UiPathConfig { base_path };

  let app_router = Router::new()
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
      &config.ui_path,
      config.ui_index_force_no_cache,
    ))
    .layer(middleware::from_fn_with_state(
      ui_path_config.clone(),
      replace_base_url_in_html,
    ));

  let mut router = Router::new().merge(app_router.clone());
  let nest_base_path = ui_path_config.base_path.trim_end_matches('/');
  if !nest_base_path.is_empty() && nest_base_path != "/" {
    router = router.nest(nest_base_path, app_router);
  }

  router.layer(cors_layer(config))
}

async fn replace_base_url_in_html(
  State(ui_path_config): State<UiPathConfig>,
  request: axum::extract::Request,
  next: Next,
) -> Response {
  let response = next.run(request).await;

  let is_html = response
    .headers()
    .get(header::CONTENT_TYPE)
    .and_then(|value| value.to_str().ok())
    .is_some_and(|value| value.starts_with("text/html"));

  if !is_html {
    return response;
  }

  let (mut parts, body) = response.into_parts();
  let Ok(bytes) = to_bytes(body, 4 * 1024 * 1024).await else {
    return Response::from_parts(parts, Body::empty());
  };

  let Ok(contents) = String::from_utf8(bytes.to_vec()) else {
    return Response::from_parts(parts, Body::from(bytes));
  };

  if !contents.contains(BASE_PATH_PLACEHOLDER) {
    return Response::from_parts(parts, Body::from(contents));
  }

  let replaced =
    contents.replace(BASE_PATH_PLACEHOLDER, &ui_path_config.base_path);
  parts.headers.remove(header::ETAG);
  parts.headers.remove(header::CONTENT_LENGTH);

  Response::from_parts(parts, Body::from(replaced))
}

fn normalize_base_path(base_url: &str) -> String {
  let pathname = url::Url::parse(base_url)
    .ok()
    .map(|url| url.path().trim().to_string())
    .unwrap_or_default();

  let trimmed = pathname.trim_matches('/');
  if trimmed.is_empty() {
    "/".to_string()
  } else {
    format!("/{trimmed}/")
  }
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
