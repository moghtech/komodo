use std::time::Duration;

use anyhow::Context;
use futures_util::TryStreamExt;
use mogh_error::serialize_error;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use crate::{KomodoClient, entities::update::UpdateListItem};

#[derive(Debug, Clone)]
pub enum UpdateWsMessage {
  Update(UpdateListItem),
  Error(UpdateWsError),
  Disconnected,
  Reconnected,
}

#[derive(Error, Debug, Clone)]
pub enum UpdateWsError {
  #[error("Failed to connect | {0}")]
  ConnectionError(String),
  #[error("Failed to recieve message | {0}")]
  MessageError(String),
  #[error("Did not recognize message | {0}")]
  MessageUnrecognized(String),
}

const MAX_SHORT_RETRY_COUNT: usize = 5;

impl KomodoClient {
  /// Subscribes to the Komodo Core update websocket,
  /// and forwards the updates over a channel.
  /// Handles reconnection internally.
  ///
  /// ```text
  /// let (mut rx, _) = komodo.subscribe_to_updates()?;
  /// loop {
  ///   let update = match rx.recv().await {
  ///     Ok(msg) => msg,
  ///     Err(e) => {
  ///       error!("🚨 recv error | {e:?}");
  ///       break;
  ///     }
  ///   };
  ///   // Handle the update
  ///   info!("Got update: {update:?}");
  /// }
  /// ```
  pub fn subscribe_to_updates(
    self,
    // retry_cooldown_secs: u64,
  ) -> anyhow::Result<(
    broadcast::Receiver<UpdateWsMessage>,
    CancellationToken,
  )> {
    let (tx, rx) = broadcast::channel(128);
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    tokio::spawn(async move {
      loop {
        // OUTER LOOP (LONG RECONNECT)
        if cancel.is_cancelled() {
          break;
        }

        let mut retry = 0;
        loop {
          // INNER LOOP (SHORT RECONNECT)
          if cancel.is_cancelled() {
            break;
          }
          if retry >= MAX_SHORT_RETRY_COUNT {
            break;
          }

          let mut ws = match self
            .connect_login_user_websocket("/update", None)
            .await
          {
            Ok(ws) => ws,
            Err(e) => {
              let _ = tx.send(UpdateWsMessage::Error(
                UpdateWsError::ConnectionError(serialize_error(&e)),
              ));
              retry += 1;
              continue;
            }
          };

          let _ = tx.send(UpdateWsMessage::Reconnected);

          // If we get to this point (connected / logged in) reset the short retry counter
          retry = 0;

          // ==================
          // HANLDE MSGS
          // ==================
          loop {
            match ws
              .try_next()
              .await
              .context("failed to recieve message")
            {
              Ok(Some(Message::Text(msg))) => {
                match serde_json::from_str::<UpdateListItem>(&msg) {
                  Ok(msg) => {
                    let _ = tx.send(UpdateWsMessage::Update(msg));
                  }
                  Err(_) => {
                    let _ = tx.send(UpdateWsMessage::Error(
                      UpdateWsError::MessageUnrecognized(
                        msg.to_string(),
                      ),
                    ));
                  }
                }
              }
              Ok(Some(Message::Close(_))) => {
                let _ = tx.send(UpdateWsMessage::Disconnected);
                let _ = ws.close(None).await;
                break;
              }
              Err(e) => {
                let _ = tx.send(UpdateWsMessage::Error(
                  UpdateWsError::MessageError(serialize_error(&e)),
                ));
                let _ = tx.send(UpdateWsMessage::Disconnected);
                let _ = ws.close(None).await;
                break;
              }
              Ok(_) => {
                // ignore
              }
            }
          }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
      }
    });

    Ok((rx, cancel_clone))
  }
}
