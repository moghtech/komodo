use std::task::Poll;

use anyhow::Context;
use axum::{Extension, Router, middleware, routing::post};
use futures::Stream;
use komodo_client::{
  api::terminal::ExecuteTerminal,
  entities::{
    permission::PermissionLevel, server::Server, user::User,
  },
};
use pin_project_lite::pin_project;
use serror::Json;
use tokio_util::codec::LinesCodecError;
use uuid::Uuid;

use crate::{
  auth::auth_request, helpers::periphery_client,
  monitor::update_cache_for_server, resource,
};

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .layer(middleware::from_fn(auth_request))
}

async fn handler(
  Extension(user): Extension<User>,
  Json(request): Json<ExecuteTerminal>,
) -> serror::Result<axum::body::Body> {
  inner(Uuid::new_v4(), request, user).await
}

#[instrument(
  name = "ExecuteTerminal",
  skip(user),
  fields(
    user_id = user.id,
  )
)]
async fn inner(
  req_id: Uuid,
  ExecuteTerminal {
    server,
    terminal,
    command,
  }: ExecuteTerminal,
  user: User,
) -> serror::Result<axum::body::Body> {
  info!("/terminal request | user: {}", user.username);

  let res = async {
    let server = resource::get_check_permissions::<Server>(
      &server,
      &user,
      PermissionLevel::Write,
    )
    .await?;

    let periphery = periphery_client(&server)?;

    let stream = periphery
      .execute_terminal(terminal, command)
      .await
      .context("Failed to execute command on periphery")?;

    anyhow::Ok(stream)
  }
  .await;

  let stream = match res {
    Ok(stream) => stream,
    Err(e) => {
      warn!("/terminal request {req_id} error: {e:#}");
      return Err(e.into());
    }
  };

  Ok(axum::body::Body::from_stream(stream.into_line_stream()))
}

pin_project! {
  pub struct RefreshOnFinish<S> { #[pin] inner: S, server: Server }
}

impl<S: Stream<Item = Result<String, LinesCodecError>>> Stream
  for RefreshOnFinish<S>
{
  type Item = Result<String, LinesCodecError>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let this = self.project();
    match this.inner.poll_next(cx) {
      Poll::Ready(None) => {
        // TODO: This isn't very efficient
        let server = this.server.clone();
        tokio::spawn(async move {
          update_cache_for_server(&server).await;
        });
        Poll::Ready(None)
      }
      other => other,
    }
  }
}
