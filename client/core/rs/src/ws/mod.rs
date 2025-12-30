use std::fmt::Write;

use anyhow::{Context, anyhow};
use futures_util::{SinkExt as _, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, tungstenite,
};
use typeshare::typeshare;

use crate::KomodoClient;

pub mod terminal;
pub mod update;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum WsLoginMessage {
  Jwt { jwt: String },
  ApiKeys { key: String, secret: String },
}

impl WsLoginMessage {
  pub fn from_json_str(json: &str) -> anyhow::Result<WsLoginMessage> {
    serde_json::from_str(json)
      .context("failed to parse json as WsLoginMessage")
  }

  pub fn to_json_string(&self) -> anyhow::Result<String> {
    serde_json::to_string(self)
      .context("failed to serialize WsLoginMessage to json string")
  }
}

impl KomodoClient {
  async fn connect_login_user_websocket(
    &self,
    path: &str,
    query: Option<&str>,
  ) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let mut endpoint =
      format!("{}/ws{path}", self.address.replacen("http", "ws", 1));
    if let Some(query) = query {
      write!(&mut endpoint, "?{query}")?;
    }
    let login_msg = WsLoginMessage::ApiKeys {
      key: self.key.clone(),
      secret: self.secret.clone(),
    }
    .to_json_string()?;
    let (mut ws, _) = tokio_tungstenite::connect_async(&endpoint)
      .await
      .with_context(|| {
        format!("failed to connect to Komodo websocket at {endpoint}")
      })?;
    ws.send(tungstenite::Message::Text(login_msg.into()))
      .await
      .context("Failed to send websocket login message")?;
    loop {
      match ws
        .try_next()
        .await
        .context("Failed to receive websocket login response")?
      {
        Some(tungstenite::Message::Text(msg)) => {
          if msg == "LOGGED_IN" {
            return Ok(ws);
          } else {
            let _ = ws.close(None).await;
            return Err(anyhow!("Failed to log in | {msg}"));
          }
        }
        Some(tungstenite::Message::Close(_)) | None => {
          let _ = ws.close(None).await;
          return Err(anyhow!("Socket closed before login"));
        }
        // Keep looping on other message types
        Some(_) => {}
      };
    }
  }
}
