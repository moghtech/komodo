use std::net::IpAddr;

use crate::{
  auth::GENERAL_RATE_LIMITER,
  auth::{auth_api_key_check_enabled, auth_jwt_check_enabled},
  helpers::query::get_user,
};
use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::ws::{self, WebSocket},
  routing::get,
};
use komodo_client::{entities::user::User, ws::WsLoginMessage};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serror::{AddStatusCode, AddStatusCodeError};

mod terminal;
mod update;

pub fn router() -> Router {
  Router::new()
    // Periphery facing
    .route("/periphery", get(crate::connection::server::handler))
    // User facing
    .route("/update", get(update::handler))
    .route("/terminal", get(terminal::handler))
}

async fn user_ws_login(
  mut socket: WebSocket,
  ip: IpAddr,
) -> Option<(WebSocket, User)> {
  let res = async {
    let message = match socket
      .recv()
      .await
      .context("Failed to receive message over socket: Closed")
      .status_code(StatusCode::BAD_REQUEST)?
      .context("Failed to recieve message over socket: Error")
      .status_code(StatusCode::BAD_REQUEST)?
    {
      ws::Message::Text(utf8_bytes) => utf8_bytes.to_string(),
      ws::Message::Binary(bytes) => String::from_utf8(bytes.into())
        .context("Received invalid message bytes: Not UTF-8")
        .status_code(StatusCode::BAD_REQUEST)?,
      message => {
        return Err(
          anyhow!("Received invalid message: {message:?}")
            .status_code(StatusCode::BAD_REQUEST),
        );
      }
    };

    match WsLoginMessage::from_json_str(&message)
      .context("Invalid login message")
      .status_code(StatusCode::BAD_REQUEST)?
    {
      WsLoginMessage::Jwt { jwt } => auth_jwt_check_enabled(&jwt)
        .await
        .status_code(StatusCode::UNAUTHORIZED),
      WsLoginMessage::ApiKeys { key, secret } => {
        auth_api_key_check_enabled(&key, &secret)
          .await
          .status_code(StatusCode::UNAUTHORIZED)
      }
    }
  }
  .with_failure_rate_limit_using_ip(&GENERAL_RATE_LIMITER, &ip)
  .await;
  match res {
    Ok(user) => {
      let _ = socket.send(ws::Message::text("LOGGED_IN")).await;
      Some((socket, user))
    }
    Err(e) => {
      let _ = socket
        .send(ws::Message::text(format!(
          "[{}]: {:#}",
          e.status, e.error
        )))
        .await;
      None
    }
  }
}

async fn check_user_valid(user_id: &str) -> anyhow::Result<User> {
  let user = get_user(user_id).await?;
  if !user.enabled {
    return Err(anyhow!("User not enabled"));
  }
  Ok(user)
}
