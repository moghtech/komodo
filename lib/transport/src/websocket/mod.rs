//! Wrappers to normalize behavior of websockets between Tungstenite and Axum

use std::time::Duration;

use anyhow::{Context, anyhow};
use bytes::Bytes;
use encoding::{
  CastBytes as _, Decode as _, Encode, EncodedJsonMessage,
  EncodedResponse, JsonMessage,
};
use periphery_client::transport::{
  EncodedTransportMessage, RequestMessage, ResponseMessage,
  TerminalMessage, TransportMessage,
};
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::timeout::MaybeWithTimeout;

pub mod axum;
pub mod login;
pub mod tungstenite;

/// Flattened websocket message possibilites
/// for easier handling.
pub enum WebsocketMessage {
  /// Standard message
  Message(EncodedTransportMessage),
  /// Core / Periphery must receive every 10s
  /// or reconnect triggered.
  Ping,
  /// Graceful close message
  Close,
  /// Stream closed
  Closed,
}

/// Standard traits for websocket
pub trait Websocket: Send {
  /// Abstraction over websocket splitting
  fn split(self) -> (impl WebsocketSender, impl WebsocketReceiver);

  fn send(
    &mut self,
    bytes: Bytes,
  ) -> impl Future<Output = anyhow::Result<()>> + Send;

  /// Send close message
  fn close(
    &mut self,
  ) -> impl Future<Output = anyhow::Result<()>> + Send;

  /// Looping receiver for websocket messages which only returns
  /// on significant messages.
  fn recv_inner(
    &mut self,
  ) -> MaybeWithTimeout<
    impl Future<Output = anyhow::Result<WebsocketMessage>> + Send,
  >;
}

pub trait WebsocketExt: Websocket {
  fn send_message(
    &mut self,
    message: impl Encode<EncodedTransportMessage>,
  ) -> impl Future<Output = anyhow::Result<()>> + Send {
    self.send(message.encode().into_bytes())
  }

  /// Looping receiver for websocket messages which only returns on TransportMessage.
  /// Also ensures either Messages or Pings are received at least every 10s.
  fn recv_message(
    &mut self,
  ) -> MaybeWithTimeout<
    impl Future<Output = anyhow::Result<TransportMessage>> + Send,
  > {
    MaybeWithTimeout::new(async {
      loop {
        match tokio::time::timeout(
          Duration::from_secs(10),
          self.recv_inner(),
        )
        .await
        .context("Timed out waiting for Ping")??
        {
          WebsocketMessage::Message(message) => {
            return message.decode();
          }
          WebsocketMessage::Ping => continue,
          WebsocketMessage::Close => {
            return Err(anyhow!("Connection closed"));
          }
          WebsocketMessage::Closed => {
            return Err(anyhow!("Connection already closed"));
          }
        }
      }
    })
  }
}

impl<W: Websocket> WebsocketExt for W {}

/// Traits for split websocket receiver
pub trait WebsocketSender {
  /// Streamlined pinging
  fn ping(
    &mut self,
  ) -> impl Future<Output = anyhow::Result<()>> + Send;

  /// Streamlined sending on bytes
  fn send(
    &mut self,
    bytes: Bytes,
  ) -> impl Future<Output = anyhow::Result<()>> + Send;

  /// Send close message
  fn close(
    &mut self,
  ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait WebsocketSenderExt: WebsocketSender + Send {
  fn send_message(
    &mut self,
    message: impl Encode<EncodedTransportMessage>,
  ) -> impl Future<Output = anyhow::Result<()>> + Send {
    self.send(message.encode().into_vec().into())
  }

  fn send_request<'a, T: Serialize + Send>(
    &mut self,
    channel: Uuid,
    request: &'a T,
  ) -> impl Future<Output = anyhow::Result<()>> + Send
  where
    &'a T: Send,
  {
    async move {
      let json = JsonMessage(request).encode()?;
      self.send_message(RequestMessage::new(channel, json)).await
    }
  }

  fn send_in_progress(
    &mut self,
    channel: Uuid,
  ) -> impl Future<Output = anyhow::Result<()>> + Send {
    self.send_message(ResponseMessage::new(
      channel,
      encoding::Response::Pending.encode(),
    ))
  }

  fn send_response(
    &mut self,
    channel: Uuid,
    response: EncodedResponse<EncodedJsonMessage>,
  ) -> impl Future<Output = anyhow::Result<()>> + Send {
    self.send_message(ResponseMessage::new(channel, response))
  }

  fn send_terminal(
    &mut self,
    channel: Uuid,
    data: anyhow::Result<Vec<u8>>,
  ) -> impl Future<Output = anyhow::Result<()>> + Send {
    self.send_message(TerminalMessage::new(channel, data))
  }
}

impl<S: WebsocketSender + Send> WebsocketSenderExt for S {}

/// Traits for split websocket receiver
pub trait WebsocketReceiver: Send {
  type CloseFrame: std::fmt::Debug + Send + Sync + 'static;

  /// Cancellation sensitive receive.
  fn set_cancel(&mut self, _cancel: CancellationToken);

  /// Looping receiver for websocket messages which only returns
  /// on significant messages. Must implement cancel support.
  fn recv(
    &mut self,
  ) -> impl Future<Output = anyhow::Result<WebsocketMessage>> + Send;
}

pub trait WebsocketReceiverExt: WebsocketReceiver {
  /// Looping receiver for websocket messages which only returns on TransportMessage.
  /// Also ensures either Messages or Pings are received at least every 10s.
  fn recv_message(
    &mut self,
  ) -> MaybeWithTimeout<
    impl Future<Output = anyhow::Result<TransportMessage>> + Send,
  > {
    MaybeWithTimeout::new(async {
      loop {
        match tokio::time::timeout(
          Duration::from_secs(10),
          self.recv(),
        )
        .await
        .context("Timed out waiting for Ping")??
        {
          WebsocketMessage::Message(message) => {
            return message.decode();
          }
          WebsocketMessage::Ping => continue,
          WebsocketMessage::Close => {
            return Err(anyhow!("Connection closed"));
          }
          WebsocketMessage::Closed => {
            return Err(anyhow!("Connection already closed"));
          }
        }
      }
    })
  }
}

impl<R: WebsocketReceiver> WebsocketReceiverExt for R {}
