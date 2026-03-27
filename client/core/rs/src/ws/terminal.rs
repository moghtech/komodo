use anyhow::Context;
use bytes::Bytes;
use futures_util::{
  SinkExt, StreamExt, TryStreamExt,
  stream::{SplitSink, SplitStream},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, tungstenite,
};

use crate::{
  KomodoClient,
  api::terminal::{ConnectTerminalQuery, InitTerminal},
  entities::terminal::{
    TerminalResizeMessage, TerminalStdinMessage, TerminalTarget,
  },
};

impl KomodoClient {
  pub async fn connect_terminal(
    &self,
    query: &ConnectTerminalQuery,
  ) -> anyhow::Result<TerminalWebsocket> {
    self
      .connect_login_user_websocket(
        "/terminal",
        Some(&serde_qs::to_string(query)?),
      )
      .await
      .map(TerminalWebsocket)
  }

  pub async fn connect_server_terminal(
    &self,
    server: String,
    terminal: Option<String>,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalWebsocket> {
    self
      .connect_terminal(&ConnectTerminalQuery {
        target: TerminalTarget::Server {
          server: Some(server),
        },
        terminal,
        init,
      })
      .await
  }

  pub async fn connect_container_terminal(
    &self,
    server: String,
    container: String,
    terminal: Option<String>,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalWebsocket> {
    self
      .connect_terminal(&ConnectTerminalQuery {
        target: TerminalTarget::Container { server, container },
        terminal,
        init,
      })
      .await
  }

  pub async fn connect_stack_service_terminal(
    &self,
    stack: String,
    service: String,
    terminal: Option<String>,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalWebsocket> {
    self
      .connect_terminal(&ConnectTerminalQuery {
        target: TerminalTarget::Stack {
          stack,
          service: Some(service),
        },
        terminal,
        init,
      })
      .await
  }

  pub async fn connect_deployment_terminal(
    &self,
    deployment: String,
    terminal: Option<String>,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalWebsocket> {
    self
      .connect_terminal(&ConnectTerminalQuery {
        target: TerminalTarget::Deployment { deployment },
        terminal,
        init,
      })
      .await
  }
}

pub type TerminalWebsocketInner =
  WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct TerminalWebsocket(TerminalWebsocketInner);

impl TerminalWebsocket {
  pub fn into_inner(self) -> TerminalWebsocketInner {
    self.0
  }

  pub fn split(
    self,
  ) -> (TerminalWebsocketSink, TerminalWebsocketStream) {
    let (write, read) = self.0.split();
    (TerminalWebsocketSink(write), TerminalWebsocketStream(read))
  }

  pub async fn send_stdin_message(
    &mut self,
    message: TerminalStdinMessage,
  ) -> anyhow::Result<()> {
    let message = message.into_terminal_message()?.into_ws_message();
    self
      .0
      .send(message)
      .await
      .context("Failed to forward stdin message")
  }

  pub async fn send_stdin_bytes(
    &mut self,
    bytes: Vec<u8>,
  ) -> anyhow::Result<()> {
    self
      .send_stdin_message(TerminalStdinMessage::Forward(bytes))
      .await
  }

  pub async fn send_resize_bytes(
    &mut self,
    resize: TerminalResizeMessage,
  ) -> anyhow::Result<()> {
    self
      .send_stdin_message(TerminalStdinMessage::Resize(resize))
      .await
  }

  pub async fn receive_stdout(
    &mut self,
  ) -> anyhow::Result<Option<Bytes>> {
    loop {
      match self.0.try_next().await.context("Websocket read error")? {
        Some(tungstenite::Message::Binary(bytes)) => {
          return Ok(Some(bytes));
        }
        Some(tungstenite::Message::Text(text)) => {
          return Ok(Some(text.into()));
        }
        Some(tungstenite::Message::Close(_)) | None => {
          return Ok(None);
        }
        // Can ignore these message types
        Some(tungstenite::Message::Ping(_))
        | Some(tungstenite::Message::Pong(_))
        | Some(tungstenite::Message::Frame(_)) => continue,
      }
    }
  }
}

pub type TerminalWebsocketSinkInner = SplitSink<
  WebSocketStream<MaybeTlsStream<TcpStream>>,
  tungstenite::Message,
>;

pub struct TerminalWebsocketSink(TerminalWebsocketSinkInner);

impl TerminalWebsocketSink {
  pub async fn send_stdin_message(
    &mut self,
    message: TerminalStdinMessage,
  ) -> anyhow::Result<()> {
    let message = message.into_terminal_message()?.into_ws_message();
    self
      .0
      .send(message)
      .await
      .context("Failed to forward stdin message")
  }

  pub async fn send_stdin_bytes(
    &mut self,
    bytes: Vec<u8>,
  ) -> anyhow::Result<()> {
    self
      .send_stdin_message(TerminalStdinMessage::Forward(bytes))
      .await
  }

  pub async fn send_resize_bytes(
    &mut self,
    resize: TerminalResizeMessage,
  ) -> anyhow::Result<()> {
    self
      .send_stdin_message(TerminalStdinMessage::Resize(resize))
      .await
  }
}

pub type TerminalWebsocketStreamInner =
  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct TerminalWebsocketStream(TerminalWebsocketStreamInner);

impl TerminalWebsocketStream {
  pub async fn receive_stdout(
    &mut self,
  ) -> anyhow::Result<Option<Bytes>> {
    loop {
      match self.0.try_next().await.context("Websocket read error")? {
        Some(tungstenite::Message::Binary(bytes)) => {
          return Ok(Some(bytes));
        }
        Some(tungstenite::Message::Text(text)) => {
          return Ok(Some(text.into()));
        }
        Some(tungstenite::Message::Close(_)) | None => {
          return Ok(None);
        }
        // Can ignore these message types
        Some(tungstenite::Message::Ping(_))
        | Some(tungstenite::Message::Pong(_))
        | Some(tungstenite::Message::Frame(_)) => continue,
      }
    }
  }
}
