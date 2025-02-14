use anyhow::anyhow;
use axum::{
  extract::{
    ws::{Message, WebSocket},
    WebSocketUpgrade,
  },
  response::IntoResponse,
  routing::get,
  Router,
};
use futures::{SinkExt, StreamExt};
use komodo_client::{
  entities::{
    permission::PermissionLevel, user::User, ResourceTarget,
  },
  ws::WsLoginMessage,
};
use serde_json::json;
use serror::serialize_error;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{
  auth::{auth_api_key_check_enabled, auth_jwt_check_enabled},
  helpers::{
    channel::update_channel,
    query::{get_user, get_user_permission_on_target},
  },
};

pub fn router() -> Router {
  Router::new().route("/update", get(ws_handler))
}

#[instrument(level = "debug")]
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
  // get a reveiver for internal update messages.
  let mut receiver = update_channel().receiver.resubscribe();

  // handle http -> ws updgrade
  ws.on_upgrade(|socket| async move {
    let Some((socket, user)) = ws_login(socket).await else {
      return
    };

    let (mut ws_sender, mut ws_reciever) = socket.split();

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    tokio::spawn(async move {
      loop {
        // poll for updates off the receiver / await cancel.
        let update = select! {
          _ = cancel_clone.cancelled() => break,
          update = receiver.recv() => {update.expect("failed to recv update msg")}
        };

        // before sending every update, verify user is still valid.
        // kill the connection is user if found to be invalid.
        let user = check_user_valid(&user.id).await;
        let user = match user {
          Err(e) => {
            let _ = ws_sender
              .send(Message::text(json!({ "type": "INVALID_USER", "msg": serialize_error(&e) }).to_string()))
              .await;
            let _ = ws_sender.close().await;
            return;
          },
          Ok(user) => user,
        };

        // Only send if user has permission on the target resource.
        if user_can_see_update(&user, &update.target).await.is_ok() {
          let _ = ws_sender
            .send(Message::text(serde_json::to_string(&update).unwrap()))
            .await;
        }
      }
    });

    // Handle messages from the client.
    // After login, only handles close message.
    while let Some(msg) = ws_reciever.next().await {
      match msg {
        Ok(msg) => {
          if let Message::Close(_) = msg {
            cancel.cancel();
            return;
          }
        }
        Err(_) => {
          cancel.cancel();
          return;
        }
      }
    }
    })
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

#[instrument(level = "debug")]
async fn user_can_see_update(
  user: &User,
  update_target: &ResourceTarget,
) -> anyhow::Result<()> {
  if user.admin {
    return Ok(());
  }
  let permissions =
    get_user_permission_on_target(user, update_target).await?;
  if permissions > PermissionLevel::None {
    Ok(())
  } else {
    Err(anyhow!(
      "user does not have permissions on {update_target:?}"
    ))
  }
}
