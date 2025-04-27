use axum::{
  extract::{
    Query, WebSocketUpgrade,
    ws::{CloseFrame, Message, Utf8Bytes},
  },
  response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use komodo_client::{
  api::terminal::ConnectTerminalQuery,
  entities::{permission::PermissionLevel, server::Server},
};
use tokio_tungstenite::tungstenite;
use tokio_util::sync::CancellationToken;

use crate::{helpers::periphery_client, resource};

#[instrument(name = "ConnectTerminal", skip(ws))]
pub async fn handler(
  Query(ConnectTerminalQuery {
    server,
    terminal,
    init,
  }): Query<ConnectTerminalQuery>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| async move {
    let Some((mut socket, user)) = super::ws_login(socket).await
    else {
      return;
    };

    let server = match resource::get_check_permissions::<Server>(
      &server,
      &user,
      PermissionLevel::Write,
    )
    .await
    {
      Ok(server) => server,
      Err(e) => {
        debug!("could not get server | {e:#}");
        let _ =
          socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = socket.close().await;
        return;
      }
    };

    let periphery = match periphery_client(&server) {
      Ok(periphery) => periphery,
      Err(e) => {
        debug!("couldn't get periphery | {e:#}");
        let _ =
          socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = socket.close().await;
        return;
      }
    };

    trace!("connecting to periphery terminal");

    let periphery_socket = match periphery
      .connect_terminal(
        terminal,
        init,
      )
      .await
    {
      Ok(ws) => ws,
      Err(e) => {
        debug!("Failed connect to periphery terminal | {e:#}");
        let _ =
          socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = socket.close().await;
        return;
      }
    };

    trace!("connected to periphery terminal socket");

    let (mut periphery_send, mut periphery_receive) =
      periphery_socket.split();
    let (mut core_send, mut core_receive) = socket.split();
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
              debug!("Failed to send terminal message to {} | {e:?}", server.name);
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
            let _ = core_send
              .send(Message::text("STREAM EOF"))
              .await;
            cancel.cancel();
            break;
          }
        }
      }
    };

    tokio::join!(core_to_periphery, periphery_to_core);
  })
}

fn axum_to_tungstenite(msg: Message) -> tungstenite::Message {
  match msg {
    Message::Text(text) => tungstenite::Message::Text(
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
