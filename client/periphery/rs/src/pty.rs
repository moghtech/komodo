use anyhow::{Context, anyhow};
use futures_util::{SinkExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream,
  tungstenite::{Message, Utf8Bytes},
};

use crate::{
  PTY_LOGGED_IN_ACK, PeripheryClient, api::pty::ConnectPtyQuery,
};

impl PeripheryClient {
  /// Handles ws connect and login
  pub async fn connect_pty(
    &self,
    query: &ConnectPtyQuery,
  ) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    tracing::trace!(
      "sending request | type: ConnectPty | pty name: {} | shell: {} | command: {:?}",
      query.pty,
      query.shell,
      query.command
    );

    let query_str = serde_qs::to_string(&query)
      .context("Failed to serialize query string")?;

    let url = format!(
      "{}/pty?{query_str}",
      self.address.replacen("http", "ws", 1)
    );

    let (mut stream, _) = tokio_tungstenite::connect_async(url)
      .await
      .context("failed to connect to websocket")?;

    

    stream
      .send(Message::Text(Utf8Bytes::from(&self.passkey)))
      .await?;

    let mut tries = 0;
    loop {
      if tries > 2 {
        return Err(anyhow!("Failed to login after 3 tries"));
      }
      match stream
        .try_next()
        .await
        .context("Failed to read ws stream")?
      {
        Some(Message::Text(text)) => {
          if text.as_str() == PTY_LOGGED_IN_ACK {
            break;
          } else {
            tries += 1;
          }
        }
        Some(Message::Binary(bytes)) => {
          if &bytes == PTY_LOGGED_IN_ACK.as_bytes() {
            break;
          } else {
            tries += 1;
          }
        }
        Some(Message::Close(_)) => {
          return Err(anyhow!("Websocket closed before login ack"));
        }
        Some(_) => {}
        None => {
          return Err(anyhow!("Websocket EOF before login ack"));
        }
      }
    }

    Ok(stream)
  }
}
