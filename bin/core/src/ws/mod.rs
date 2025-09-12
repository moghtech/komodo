use crate::{
  auth::{auth_api_key_check_enabled, auth_jwt_check_enabled},
  helpers::query::get_user,
};
use anyhow::anyhow;
use axum::{
  Router,
  extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
  routing::get,
};
use futures::{SinkExt, StreamExt};
use komodo_client::{
  entities::{server::Server, user::User},
  ws::WsLoginMessage,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, tungstenite,
};
use tokio_util::sync::CancellationToken;

mod container;
mod deployment;
mod stack;
mod terminal;
mod update;

pub fn router() -> Router {
  Router::new()
    .route("/update", get(update::handler))
    .route("/terminal", get(terminal::handler))
    .route("/container/terminal", get(container::terminal))
    .route("/deployment/terminal", get(deployment::terminal))
    .route("/stack/terminal", get(stack::terminal))
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
              "failed to authenticate user using jwt | {e:?}"
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
              "failed to authenticate user using api keys | {e:?}"
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

async fn handle_container_terminal(
  mut client_socket: WebSocket,
  server: &Server,
  container: String,
  shell: String,
) {
  let periphery = match crate::helpers::periphery_client(server) {
    Ok(periphery) => periphery,
    Err(e) => {
      debug!("couldn't get periphery | {e:#}");
      let _ = client_socket
        .send(Message::text(format!("ERROR: {e:#}")))
        .await;
      let _ = client_socket.close().await;
      return;
    }
  };

  trace!("connecting to periphery container exec websocket");

  let periphery_socket = match periphery
    .connect_container_exec(container, shell)
    .await
  {
    Ok(ws) => ws,
    Err(e) => {
      debug!(
        "Failed connect to periphery container exec websocket | {e:#}"
      );
      let _ = client_socket
        .send(Message::text(format!("ERROR: {e:#}")))
        .await;
      let _ = client_socket.close().await;
      return;
    }
  };

  trace!("connected to periphery container exec websocket");

  core_periphery_forward_ws(client_socket, periphery_socket).await
}

async fn core_periphery_forward_ws(
  client_socket: axum::extract::ws::WebSocket,
  periphery_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
  let (mut periphery_send, mut periphery_receive) =
    periphery_socket.split();
  let (mut core_send, mut core_receive) = client_socket.split();
  let cancel = CancellationToken::new();

  trace!("starting ws exchange");

  let core_to_periphery = async {
    loop {
      let res = tokio::select! {
        res = core_receive.next() => res,
        _ = cancel.cancelled() => {
          trace!("core to periphery read: cancelled from inside");
          break;
        }
      };
      match res {
        Some(Ok(msg)) => {
          if let Err(e) =
            periphery_send.send(axum_to_tungstenite(msg)).await
          {
            debug!("Failed to send terminal message | {e:?}",);
            cancel.cancel();
            break;
          };
        }
        Some(Err(_e)) => {
          cancel.cancel();
          break;
        }
        None => {
          cancel.cancel();
          break;
        }
      }
    }
  };

  let periphery_to_core = async {
    loop {
      let res = tokio::select! {
        res = periphery_receive.next() => res,
        _ = cancel.cancelled() => {
          trace!("periphery to core read: cancelled from inside");
          break;
        }
      };
      match res {
        Some(Ok(msg)) => {
          if let Err(e) =
            core_send.send(tungstenite_to_axum(msg)).await
          {
            debug!("{e:?}");
            cancel.cancel();
            break;
          };
        }
        Some(Err(e)) => {
          let _ = core_send
              .send(Message::text(format!(
                "ERROR: Failed to receive message from periphery | {e:?}"
              )))
              .await;
          cancel.cancel();
          break;
        }
        None => {
          let _ = core_send.send(Message::text("STREAM EOF")).await;
          cancel.cancel();
          break;
        }
      }
    }
  };

  tokio::join!(core_to_periphery, periphery_to_core);
}

fn axum_to_tungstenite(msg: Message) -> tungstenite::Message {
  match msg {
    Message::Text(text) => tungstenite::Message::Text(
      // TODO: improve this conversion cost from axum ws library
      tungstenite::Utf8Bytes::from(text.to_string()),
    ),
    Message::Binary(bytes) => tungstenite::Message::Binary(bytes),
    Message::Ping(bytes) => tungstenite::Message::Ping(bytes),
    Message::Pong(bytes) => tungstenite::Message::Pong(bytes),
    Message::Close(close_frame) => {
      tungstenite::Message::Close(close_frame.map(|cf| {
        tungstenite::protocol::CloseFrame {
          code: cf.code.into(),
          reason: tungstenite::Utf8Bytes::from(cf.reason.to_string()),
        }
      }))
    }
  }
}

fn tungstenite_to_axum(msg: tungstenite::Message) -> Message {
  match msg {
    tungstenite::Message::Text(text) => {
      Message::Text(Utf8Bytes::from(text.to_string()))
    }
    tungstenite::Message::Binary(bytes) => Message::Binary(bytes),
    tungstenite::Message::Ping(bytes) => Message::Ping(bytes),
    tungstenite::Message::Pong(bytes) => Message::Pong(bytes),
    tungstenite::Message::Close(close_frame) => {
      Message::Close(close_frame.map(|cf| CloseFrame {
        code: cf.code.into(),
        reason: Utf8Bytes::from(cf.reason.to_string()),
      }))
    }
    tungstenite::Message::Frame(_) => {
      unreachable!()
    }
  }
}
