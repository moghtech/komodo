use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::Path,
  http::{HeaderMap, HeaderValue},
  routing::get,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serror::AddStatusCodeError;
use tokio::fs;

use crate::config::core_config;

pub fn router() -> Router {
  Router::new().route("/{path}", get(serve_client_file))
}

const ALLOWED_FILES: &[&str] = &[
  "lib.js",
  "lib.d.ts",
  "types.js",
  "types.d.ts",
  "types_namespace.js",
  "types_namespace.d.ts",
  "responses.js",
  "responses.d.ts",
  "terminal.js",
  "terminal.d.ts",
];

#[derive(Deserialize)]
struct FilePath {
  path: String,
}

#[axum::debug_handler]
async fn serve_client_file(
  Path(FilePath { path }): Path<FilePath>,
) -> serror::Result<(HeaderMap, String)> {
  if !ALLOWED_FILES.contains(&path.as_str()) {
    return Err(
      anyhow!("File {path} not found.")
        .status_code(StatusCode::NOT_FOUND),
    );
  }

  let contents = fs::read_to_string(format!(
    "{}/client/{path}",
    core_config().frontend_path
  ))
  .await
  .with_context(|| format!("Failed to read file: {path}"))?;

  let mut headers = HeaderMap::new();

  if path.ends_with(".js") {
    headers.insert(
      "X-TypeScript-Types",
      HeaderValue::from_str(&format!(
        "/client/{}",
        path.replace(".js", ".d.ts")
      ))
      .context("?? Invalid Header Value")?,
    );
  }

  Ok((headers, contents))
}
