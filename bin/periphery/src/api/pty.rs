use axum::{
  extract::{
    Query, WebSocketUpgrade,
    ws::{Message, Utf8Bytes},
  },
  response::Response,
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use komodo_client::entities::NoData;
use periphery_client::{
  PTY_LOGGED_IN_ACK,
  api::pty::{ConnectPtyQuery, DeletePty, ListPtys},
};
use resolver_api::Resolve;
use tokio_util::sync::CancellationToken;

use crate::{
  config::periphery_config,
  pty::{
    ResizeDimensions, StdinMsg, clean_up_ptys, delete_pty,
    get_or_insert_pty, list_ptys,
  },
};

impl Resolve<super::Args> for ListPtys {
  #[instrument(name = "ListPtys", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<String>> {
    Ok(list_ptys())
  }
}

impl Resolve<super::Args> for DeletePty {
  #[instrument(name = "DeletePty", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<NoData> {
    delete_pty(&self.pty);
    Ok(NoData {})
  }
}

pub async fn connect_pty(
  Query(ConnectPtyQuery {
    pty,
    command,
    shell,
  }): Query<ConnectPtyQuery>,
  ws: WebSocketUpgrade,
) -> Response {
  clean_up_ptys();
  ws.on_upgrade(|mut socket| async move {
    // Token auth
    let mut tries = 0;
    'auth: loop {
      if tries > 5 {
        let _ = socket
          .send(Message::Text(Utf8Bytes::from_static(
            "Failed to auth",
          )))
          .await;
        let _ = socket.close().await;
        return;
      }
      match socket.recv().await {
        Some(Ok(Message::Text(incoming))) => {
          for passkey in &periphery_config().passkeys {
            if passkey == incoming.as_str() {
              // Auth successful
              break 'auth;
            }
          }
          tries += 1;
        }
        Some(Ok(Message::Binary(bytes))) => {
          for passkey in &periphery_config().passkeys {
            if passkey.as_bytes() == &bytes {
              // Auth successful
              break 'auth;
            }
          }
          tries += 1;
        }
        Some(Ok(Message::Close(_))) => {
          return;
        }
        Some(Ok(_)) => {}
        Some(Err(e)) => {
          let _ = socket
            .send(Message::Text(Utf8Bytes::from(format!(
              "Failed to auth | {e:?}"
            ))))
            .await;
          let _ = socket.close().await;
          return;
        }
        None => {
          tries += 1;
        }
      }
    }

    if let Err(e) = socket
      .send(Message::Text(Utf8Bytes::from_static(PTY_LOGGED_IN_ACK)))
      .await
    {
      debug!("Failed to send login ack | {e:?}");
      let _ = socket.close().await;
      return;
    };

    let pty = match get_or_insert_pty(pty, shell) {
      Ok(pty) => pty,
      Err(e) => {
        let _ = socket
          .send(Message::Text(Utf8Bytes::from(format!(
            "ERROR: {e:#}"
          ))))
          .await;
        let _ = socket.close().await;
        return;
      }
    };

    if let Some(command) = command {
      if let Err(e) =
        pty.stdin.send(StdinMsg::Bytes(Bytes::from(command)))
      {
        let _ = socket
          .send(Message::Text(Utf8Bytes::from(format!(
            "ERROR: Failed to send init command to stdin: {e:?}"
          ))))
          .await;
        let _ = socket.close().await;
        clean_up_ptys();
        return;
      };
    }

    let (mut ws_write, mut ws_read) = socket.split();
    let cancel = CancellationToken::new();

    let ws_read = async {
      loop {
        let res = tokio::select! {
          res = ws_read.next() => res,
          _ = pty.cancelled() => {
            trace!("ws read: cancelled from outside");
            break
          },
          _ = cancel.cancelled() => {
            trace!("ws read: cancelled from inside");
            break;
          }
        };
        match res {
          Some(Ok(Message::Binary(bytes)))
            if bytes.first() == Some(&0x00) =>
          {
            // println!("Got ws read bytes - for stdin");
            if let Err(e) = pty.stdin.send(StdinMsg::Bytes(
              Bytes::copy_from_slice(&bytes[1..]),
            )) {
              debug!("WS -> PTY channel send error: {e:}");
              pty.cancel();
              break;
            };
          }
          Some(Ok(Message::Binary(bytes)))
            if bytes.first() == Some(&0xFF) =>
          {
            // println!("Got ws read bytes - for resize");
            if let Ok(dimensions) =
              serde_json::from_slice::<ResizeDimensions>(&bytes[1..])
            {
              if let Err(e) =
                pty.stdin.send(StdinMsg::Resize(dimensions))
              {
                debug!("WS -> PTY channel send error: {e:}");
                pty.cancel();
                break;
              };
            }
          }
          Some(Ok(Message::Text(text))) => {
            trace!("Got ws read text");
            if let Err(e) =
              pty.stdin.send(StdinMsg::Bytes(Bytes::from(text)))
            {
              debug!("WS -> PTY channel send error: {e:?}");
              pty.cancel();
              break;
            };
          }
          Some(Ok(Message::Close(_))) => {
            debug!("got ws read close");
            cancel.cancel();
            break;
          }
          Some(Ok(_)) => {
            // Do nothing (ping, non-prefixed bytes, etc.)
          }
          Some(Err(e)) => {
            debug!("Got ws read error: {e:?}");
            cancel.cancel();
            break;
          }
          None => {
            debug!("Got ws read none");
            cancel.cancel();
            break;
          }
        }
      }
    };

    let ws_write = async {
      loop {
        let res = tokio::select! {
          res = pty.stdout.recv_async() => res,
          _ = pty.cancelled() => {
            trace!("ws write: cancelled from outside");
            let _ = ws_write.send(Message::Text(Utf8Bytes::from_static("PTY KILLED"))).await;
            if let Err(e) = ws_write.close().await {
              debug!("Failed to close ws: {e:?}");
            };
            break
          },
          _ = cancel.cancelled() => {
            let _ = ws_write.send(Message::Text(Utf8Bytes::from_static("WS KILLED"))).await;
            if let Err(e) = ws_write.close().await {
              debug!("Failed to close ws: {e:?}");
            };
            break
          }
        };
        match res {
          Ok(bytes) => {
            if let Err(e) =
              ws_write.send(Message::Binary(bytes)).await
            {
              debug!("Failed to send to WS: {e:?}");
              cancel.cancel();
              break;
            }
          }
          Err(e) => {
            debug!("PTY -> WS channel read error: {e:?}");
            pty.cancel();
            break;
          }
        }
      }
    };

    tokio::join!(ws_read, ws_write);
  })
}
