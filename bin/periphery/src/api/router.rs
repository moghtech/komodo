use std::net::SocketAddr;

use anyhow::{Context, anyhow};
use axum::{
  Router,
  body::Body,
  extract::ConnectInfo,
  http::{Request, StatusCode},
  middleware::{self, Next},
  response::Response,
  routing::post,
};
use derive_variants::ExtractVariant;
use resolver_api::Resolve;
use serror::{AddStatusCode, AddStatusCodeError, Json};
use uuid::Uuid;

use crate::config::periphery_config;

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .route("/terminal/exec", post(super::terminal::exec))
    .layer(middleware::from_fn(guard_request_by_ip))
    .layer(middleware::from_fn(guard_request_by_passkey))
}

async fn handler(
  Json(request): Json<crate::api::PeripheryRequest>,
) -> serror::Result<axum::response::Response> {
  let req_id = Uuid::new_v4();

  let res = tokio::spawn(task(req_id, request))
    .await
    .context("task handler spawn error");

  if let Err(e) = &res {
    warn!("request {req_id} spawn error: {e:#}");
  }

  res?
}

async fn task(
  req_id: Uuid,
  request: crate::api::PeripheryRequest,
) -> serror::Result<axum::response::Response> {
  let variant = request.extract_variant();

  let res = request.resolve(&crate::api::Args).await.map(|res| res.0);

  if let Err(e) = &res {
    warn!(
      "request {req_id} | type: {variant:?} | error: {:#}",
      e.error
    );
  }

  res
}

async fn guard_request_by_passkey(
  req: Request<Body>,
  next: Next,
) -> serror::Result<Response> {
  if periphery_config().passkeys.is_empty() {
    return Ok(next.run(req).await);
  }
  let Some(req_passkey) = req.headers().get("authorization") else {
    return Err(
      anyhow!("request was not sent with passkey")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  };
  let req_passkey = req_passkey
    .to_str()
    .context("failed to convert passkey to str")
    .status_code(StatusCode::UNAUTHORIZED)?;
  if periphery_config()
    .passkeys
    .iter()
    .any(|passkey| passkey == req_passkey)
  {
    Ok(next.run(req).await)
  } else {
    Err(
      anyhow!("request passkey invalid")
        .status_code(StatusCode::UNAUTHORIZED),
    )
  }
}

async fn guard_request_by_ip(
  req: Request<Body>,
  next: Next,
) -> serror::Result<Response> {
  if periphery_config().allowed_ips.is_empty() {
    return Ok(next.run(req).await);
  }
  let ConnectInfo(socket_addr) = req
    .extensions()
    .get::<ConnectInfo<SocketAddr>>()
    .context("could not get ConnectionInfo of request")
    .status_code(StatusCode::UNAUTHORIZED)?;
  let ip = socket_addr.ip();
  if periphery_config().allowed_ips.contains(&ip) {
    Ok(next.run(req).await)
  } else {
    Err(
      anyhow!("requesting ip {ip} not allowed")
        .status_code(StatusCode::UNAUTHORIZED),
    )
  }
}
