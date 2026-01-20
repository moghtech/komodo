use anyhow::Context;
use axum::{Extension, Router, middleware, routing::post};
use komodo_client::{api::terminal::*, entities::user::User};
use mogh_auth_server::middleware::authenticate_request;
use mogh_error::Json;

use crate::{
  auth::KomodoAuthImpl, helpers::terminal::setup_target_for_user,
};

pub fn router() -> Router {
  Router::new()
    .route("/execute", post(execute_terminal))
    .layer(middleware::from_fn(
      authenticate_request::<KomodoAuthImpl>,
    ))
}

// =================
//  ExecuteTerminal
// =================

#[instrument(
  name = "ExecuteTerminal",
  skip_all,
  fields(
    operator = user.id,
    target,
    terminal,
    init = format!("{init:?}")
  )
)]
async fn execute_terminal(
  Extension(user): Extension<User>,
  Json(ExecuteTerminalBody {
    target,
    terminal,
    command,
    init,
  }): Json<ExecuteTerminalBody>,
) -> mogh_error::Result<axum::body::Body> {
  info!("/terminal/execute request | user: {}", user.username);

  let (target, terminal, periphery) =
    setup_target_for_user(target, terminal, init, &user).await?;

  let stream = periphery
    .execute_terminal(target, terminal, command)
    .await
    .context("Failed to execute command on Terminal")?;

  Ok(axum::body::Body::from_stream(stream))
}
