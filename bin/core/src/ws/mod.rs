use std::{collections::HashMap, sync::OnceLock};

use crate::{
  auth::{auth_api_key_check_enabled, auth_jwt_check_enabled},
  helpers::query::get_user,
};
use anyhow::anyhow;
use axum::{
  Router,
  extract::ws::{Message, WebSocket},
  routing::get,
};
use futures::SinkExt;
use komodo_client::{
  entities::{komodo_timestamp, user::User},
  ws::WsLoginMessage,
};
use rand::Rng;
use reqwest::StatusCode;
use serror::AddStatusCodeError;

mod pty;
mod update;

pub fn router() -> Router {
  Router::new()
    .route("/update", get(update::handler))
    .route("/pty", get(pty::handler))
}

#[instrument(level = "debug")]
async fn ws_login(
  mut socket: WebSocket,
) -> Option<(WebSocket, User)> {
  let login_msg = match socket.recv().await {
    Some(Ok(Message::Text(login_msg))) => {
      LoginMessage::Ok(login_msg.to_string())
    }
    Some(Ok(msg)) => {
      LoginMessage::Err(format!("invalid login message: {msg:?}"))
    }
    Some(Err(e)) => {
      LoginMessage::Err(format!("failed to get login message: {e:?}"))
    }
    None => {
      LoginMessage::Err("failed to get login message".to_string())
    }
  };
  let login_msg = match login_msg {
    LoginMessage::Ok(login_msg) => login_msg,
    LoginMessage::Err(msg) => {
      let _ = socket.send(Message::text(msg)).await;
      let _ = socket.close().await;
      return None;
    }
  };
  match WsLoginMessage::from_json_str(&login_msg) {
    // Login using a jwt
    Ok(WsLoginMessage::Jwt { jwt }) => {
      match auth_jwt_check_enabled(&jwt).await {
        Ok(user) => {
          let _ = socket.send(Message::text("LOGGED_IN")).await;
          Some((socket, user))
        }
        Err(e) => {
          let _ = socket
            .send(Message::text(format!(
              "failed to authenticate user using jwt | {e:#}"
            )))
            .await;
          let _ = socket.close().await;
          None
        }
      }
    }
    // login using api keys
    Ok(WsLoginMessage::ApiKeys { key, secret }) => {
      match auth_api_key_check_enabled(&key, &secret).await {
        Ok(user) => {
          let _ = socket.send(Message::text("LOGGED_IN")).await;
          Some((socket, user))
        }
        Err(e) => {
          let _ = socket
            .send(Message::text(format!(
              "failed to authenticate user using api keys | {e:#}"
            )))
            .await;
          let _ = socket.close().await;
          None
        }
      }
    }
    Err(e) => {
      let _ = socket
        .send(Message::text(format!(
          "failed to parse login message: {e:#}"
        )))
        .await;
      let _ = socket.close().await;
      None
    }
  }
}

enum LoginMessage {
  /// The text message
  Ok(String),
  /// The err message
  Err(String),
}

#[instrument(level = "debug")]
async fn check_user_valid(user_id: &str) -> anyhow::Result<User> {
  let user = get_user(user_id).await?;
  if !user.enabled {
    return Err(anyhow!("user not enabled"));
  }
  Ok(user)
}

/// Tokens valid for 3 seconds
const TOKEN_VALID_FOR_MS: i64 = 3_000;

fn auth_tokens() -> &'static AuthTokens {
  static AUTH_TOKENS: OnceLock<AuthTokens> = OnceLock::new();
  AUTH_TOKENS.get_or_init(Default::default)
}

#[derive(Default)]
struct AuthTokens {
  map: std::sync::Mutex<HashMap<String, (i64, User)>>,
}

impl AuthTokens {
  pub fn create_auth_token(&self, user: User) -> String {
    let token: String = rand::rng()
      .sample_iter(&rand::distr::Alphanumeric)
      .take(30)
      .map(char::from)
      .collect();
    self.map.lock().unwrap().insert(
      token.clone(),
      (komodo_timestamp() + TOKEN_VALID_FOR_MS, user),
    );
    token
  }

  pub fn check_token_get_user(
    &self,
    token: String,
  ) -> serror::Result<User> {
    let Some((valid_until, user)) =
      self.map.lock().unwrap().remove(&token)
    else {
      return Err(
        anyhow!("Ws auth token not found")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    };
    if komodo_timestamp() <= valid_until {
      Ok(user)
    } else {
      Err(
        anyhow!("Ws auth token is expired")
          .status_code(StatusCode::UNAUTHORIZED),
      )
    }
  }
}
