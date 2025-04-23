use std::{collections::HashMap, sync::OnceLock};

use anyhow::{Context, anyhow};
use axum::{
  extract::{
    Query, WebSocketUpgrade,
    ws::{Message, Utf8Bytes},
  },
  http::StatusCode,
  response::Response,
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use komodo_client::entities::{NoData, komodo_timestamp};
use periphery_client::api::pty::{
  ConnectPtyQuery, CreatePtyAuthToken, CreatePtyAuthTokenResponse,
  DeletePty, ListPtys,
};
use rand::Rng;
use resolver_api::Resolve;
use serror::AddStatusCodeError;
use tokio_util::sync::CancellationToken;

use crate::pty::{
  ResizeDimensions, StdinMsg, clean_up_ptys, delete_pty,
  get_or_insert_pty, list_ptys,
};

impl Resolve<super::Args> for ListPtys {
  #[instrument(name = "ListPtys", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<String>> {
    clean_up_ptys();
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

impl Resolve<super::Args> for CreatePtyAuthToken {
  #[instrument(name = "CreatePtyAuthToken", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<CreatePtyAuthTokenResponse> {
    Ok(CreatePtyAuthTokenResponse {
      token: auth_tokens().create_auth_token(),
    })
  }
}

/// Tokens valid for 3 seconds
const TOKEN_VALID_FOR_MS: i64 = 3_000;

fn auth_tokens() -> &'static AuthTokens {
  static AUTH_TOKENS: OnceLock<AuthTokens> = OnceLock::new();
  AUTH_TOKENS.get_or_init(Default::default)
}

#[derive(Default)]
struct AuthTokens {
  map: std::sync::Mutex<HashMap<String, i64>>,
}

impl AuthTokens {
  pub fn create_auth_token(&self) -> String {
    let token: String = rand::rng()
      .sample_iter(&rand::distr::Alphanumeric)
      .take(30)
      .map(char::from)
      .collect();
    self
      .map
      .lock()
      .unwrap()
      .insert(token.clone(), komodo_timestamp() + TOKEN_VALID_FOR_MS);
    token
  }

  pub fn check_token(&self, token: String) -> serror::Result<()> {
    let Some(valid_until) = self.map.lock().unwrap().remove(&token)
    else {
      return Err(
        anyhow!("Pty auth token not found")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    };
    if komodo_timestamp() <= valid_until {
      Ok(())
    } else {
      Err(
        anyhow!("Pty token is expired")
          .status_code(StatusCode::UNAUTHORIZED),
      )
    }
  }
}

pub async fn connect_pty(
  Query(ConnectPtyQuery {
    token,
    pty,
    command,
    shell,
  }): Query<ConnectPtyQuery>,
  ws: WebSocketUpgrade,
) -> serror::Result<Response> {
  clean_up_ptys();
  auth_tokens().check_token(token)?;
  let pty = get_or_insert_pty(pty, shell)
    .context("Failed to get pty handle")?;
  if let Some(command) = command {
    pty
      .stdin
      .send(StdinMsg::Bytes(Bytes::from(command + "\n")))
      .context("Failed to run init command")?
  }
  Ok(ws.on_upgrade(|socket| async move {
    let (mut ws_write, mut ws_read) = socket.split();
    

    let cancel = CancellationToken::new();

    let ws_read = async {
      loop {
        let res = tokio::select! {
          res = ws_read.next() => res,
          _ = pty.cancel.cancelled() => {
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
          _ = pty.cancel.cancelled() => {
            info!("ws write: cancelled from outside");
            let _ = ws_write.send(Message::Text(Utf8Bytes::from_static("PTY KILLED"))).await;
            if let Err(e) = ws_write.close().await {
              warn!("Failed to close ws: {e:?}");
            };
            break
          },
          _ = cancel.cancelled() => {
            let _ = ws_write.send(Message::Text(Utf8Bytes::from_static("WS KILLED"))).await;
            if let Err(e) = ws_write.close().await {
              warn!("Failed to close ws: {e:?}");
            };
            break
          }
        };
        match res {
          Ok(bytes) => {
            if let Err(e) =
              ws_write.send(Message::Binary(bytes)).await
            {
              warn!("Failed to send to WS: {e:?}");
              cancel.cancel();
              break;
            }
          }
          Err(e) => {
            warn!("PTY -> WS channel read error: {e:?}");
            pty.cancel();
            break;
          }
        }
      }
    };

    tokio::join!(ws_read, ws_write);
  }))
}
