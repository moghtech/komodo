use std::{collections::HashMap, sync::OnceLock, task::Poll};

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
use futures::{SinkExt, Stream, StreamExt, TryStreamExt};
use komodo_client::entities::{
  KOMODO_EXIT_CODE, NoData, komodo_timestamp, server::TerminalInfo,
};
use periphery_client::api::terminal::*;
use pin_project_lite::pin_project;
use rand::Rng;
use resolver_api::Resolve;
use serror::{AddStatusCodeError, Json};
use tokio_util::sync::CancellationToken;

use crate::{config::periphery_config, terminal::*};

impl Resolve<super::Args> for ListTerminals {
  #[instrument(name = "ListTerminals", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<TerminalInfo>> {
    if periphery_config().disable_terminals {
      return Err(
        anyhow!("Terminals are disabled in the periphery config")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    clean_up_terminals().await;
    Ok(list_terminals().await)
  }
}

impl Resolve<super::Args> for CreateTerminal {
  #[instrument(name = "CreateTerminal", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<NoData> {
    if periphery_config().disable_terminals {
      return Err(
        anyhow!("Terminals are disabled in the periphery config")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    create_terminal(self.name, self.command, self.recreate)
      .await
      .map(|_| NoData {})
      .map_err(Into::into)
  }
}

impl Resolve<super::Args> for DeleteTerminal {
  #[instrument(name = "DeleteTerminal", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<NoData> {
    if periphery_config().disable_terminals {
      return Err(
        anyhow!("Terminals are disabled in the periphery config")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    delete_terminal(&self.terminal).await;
    Ok(NoData {})
  }
}

impl Resolve<super::Args> for DeleteAllTerminals {
  #[instrument(name = "DeleteAllTerminals", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<NoData> {
    if periphery_config().disable_terminals {
      return Err(
        anyhow!("Terminals are disabled in the periphery config")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    delete_all_terminals().await;
    Ok(NoData {})
  }
}

impl Resolve<super::Args> for CreateTerminalAuthToken {
  #[instrument(name = "CreateTerminalAuthToken", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<CreateTerminalAuthTokenResponse> {
    if periphery_config().disable_terminals {
      return Err(
        anyhow!("Terminals are disabled in the periphery config")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    Ok(CreateTerminalAuthTokenResponse {
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
        anyhow!("Terminal auth token not found")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    };
    if komodo_timestamp() <= valid_until {
      Ok(())
    } else {
      Err(
        anyhow!("Terminal token is expired")
          .status_code(StatusCode::UNAUTHORIZED),
      )
    }
  }
}

pub async fn connect_terminal(
  Query(ConnectTerminalQuery {
    token,
    terminal,
    init,
  }): Query<ConnectTerminalQuery>,
  ws: WebSocketUpgrade,
) -> serror::Result<Response> {
  if periphery_config().disable_terminals {
    return Err(
      anyhow!("Terminals are disabled in the periphery config")
        .status_code(StatusCode::FORBIDDEN),
    );
  }

  // Auth the connection with single use token
  auth_tokens().check_token(token)?;

  clean_up_terminals().await;
  let terminal = get_terminal(&terminal).await?;

  Ok(ws.on_upgrade(|mut socket| async move {
    let init_res = async {
      let (a, b) = terminal.history.bytes_parts();
      if !a.is_empty() {
        socket.send(Message::Binary(a)).await.context("Failed to send history part a")?;
      }
      if !b.is_empty() {
        socket.send(Message::Binary(b)).await.context("Failed to send history part b")?;
      }

      if let Some(init) = init {
        terminal
          .stdin
          .send(StdinMsg::Bytes(Bytes::from(init + "\n")))
          .await
          .context("Failed to run init command")?
      }
      anyhow::Ok(())
    }.await;

    if let Err(e) = init_res {
      let _ = socket.send(Message::Text(format!("ERROR: {e:#}").into())).await;
      let _ = socket.close().await;
      return;
    }

    let (mut ws_write, mut ws_read) = socket.split();

    let cancel = CancellationToken::new();

    let ws_read = async {
      loop {
        let res = tokio::select! {
          res = ws_read.next() => res,
          _ = terminal.cancel.cancelled() => {
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
            if let Err(e) = terminal.stdin.send(StdinMsg::Bytes(
              Bytes::copy_from_slice(&bytes[1..]),
            )).await {
              debug!("WS -> PTY channel send error: {e:}");
              terminal.cancel();
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
                terminal.stdin.send(StdinMsg::Resize(dimensions)).await
              {
                debug!("WS -> PTY channel send error: {e:}");
                terminal.cancel();
                break;
              };
            }
          }
          Some(Ok(Message::Text(text))) => {
            trace!("Got ws read text");
            if let Err(e) =
              terminal.stdin.send(StdinMsg::Bytes(Bytes::from(text))).await
            {
              debug!("WS -> PTY channel send error: {e:?}");
              terminal.cancel();
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
      let mut stdout = terminal.stdout.resubscribe();
      loop {
        let res = tokio::select! {
          res = stdout.recv() => res.context("Failed to get message over stdout receiver"),
          _ = terminal.cancel.cancelled() => {
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
            let _ = ws_write.send(Message::Text(Utf8Bytes::from(format!("ERROR: {e:#}")))).await;
            let _ = ws_write.close().await;
            terminal.cancel();
            break;
          }
        }
      }
    };

    tokio::join!(ws_read, ws_write);

    clean_up_terminals().await;
  }))
}

/// Sentinels
const START_OF_OUTPUT: &str = "__KOMODO_START_OF_OUTPUT__";
const END_OF_OUTPUT: &str = "__KOMODO_END_OF_OUTPUT__";

pub async fn execute_terminal(
  Json(ExecuteTerminalBody { terminal, command }): Json<
    ExecuteTerminalBody,
  >,
) -> serror::Result<axum::body::Body> {
  let terminal = get_terminal(&terminal).await?;

  // Read the bytes into lines
  // This is done to check the lines for the EOF sentinal
  let mut stdout = tokio_util::codec::FramedRead::new(
    tokio_util::io::StreamReader::new(
      tokio_stream::wrappers::BroadcastStream::new(
        terminal.stdout.resubscribe(),
      )
      .map(|res| res.map_err(std::io::Error::other)),
    ),
    tokio_util::codec::LinesCodec::new(),
  );

  let full_command = format!(
    "printf '\n{START_OF_OUTPUT}\n'; {command}; rc=$? printf '\n{KOMODO_EXIT_CODE}%d\n{END_OF_OUTPUT}\n' \"$rc\"\n"
  );

  terminal
    .stdin
    .send(StdinMsg::Bytes(Bytes::from(full_command)))
    .await
    .context("Failed to send command to terminal stdin")?;

  // Only start the response AFTER the start sentinel is printed
  loop {
    match stdout
      .try_next()
      .await
      .context("Failed to read stdout line")?
    {
      Some(line) if line == START_OF_OUTPUT => break,
      // Keep looping until the start sentinel received.
      Some(line) => {
        println!("{line} - {}", line_compare(&line, START_OF_OUTPUT));
      }
      None => {
        return Err(
          anyhow!(
            "Stdout stream terminated before start sentinel received"
          )
          .into(),
        );
      }
    }
  }

  Ok(axum::body::Body::from_stream(TerminalStream { stdout }))
}

fn line_compare(a: &str, b: &str) -> String {
  // let a_len = a.len();
  // let b_len = b.len();
  // if a_len != b_len {
  //   return format!("different length: {a_len} vs {b_len}");
  // }
  for (i, (ac, bc)) in a.chars().zip(b.chars()).enumerate() {
    if ac != bc {
      return format!("[{i}]: '{ac}' != '{bc}'");
    }
  }
  String::from("string are equal")
}

pin_project! {
  struct TerminalStream<S> { #[pin] stdout: S }
}

impl<S> Stream for TerminalStream<S>
where
  S:
    Stream<Item = Result<String, tokio_util::codec::LinesCodecError>>,
{
  // Axum expects a stream of results
  type Item = Result<String, String>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let this = self.project();
    match this.stdout.poll_next(cx) {
      Poll::Ready(None) => {
        // This is if a None comes in before END_OF_OUTPUT.
        // This probably means the terminal has exited early,
        // and needs to be cleaned up
        tokio::spawn(async move { clean_up_terminals().await });
        Poll::Ready(None)
      }
      Poll::Ready(Some(line)) => {
        match line {
          Ok(line) if line.as_str() == END_OF_OUTPUT => {
            // Stop the stream on end sentinel
            Poll::Ready(None)
          }
          Ok(line) => Poll::Ready(Some(Ok(line + "\n"))),
          Err(e) => Poll::Ready(Some(Err(format!("{e:?}")))),
        }
      }
      Poll::Pending => Poll::Pending,
    }
  }
}
