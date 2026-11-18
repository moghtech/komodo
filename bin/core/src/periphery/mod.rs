use std::{sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use encoding::{Decode as _, Response};
use komodo_client::entities::update::{Log, Update};
use mogh_resolver::HasResponse;
use periphery_client::api;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use tokio::sync::mpsc;
use transport::channel::channel;
use uuid::Uuid;

use crate::{
  connection::{
    PeripheryConnection, PeripheryConnectionArgs, ResponseChannels,
  },
  helpers::update::update_update_without_send,
  state::periphery_connections,
};

pub mod terminal;

#[derive(Debug)]
pub struct PeripheryClient {
  /// Usually the server id
  pub id: String,
  pub responses: Arc<ResponseChannels>,
}

impl PeripheryClient {
  pub async fn new(
    args: PeripheryConnectionArgs<'_>,
    insecure_tls: bool,
  ) -> anyhow::Result<PeripheryClient> {
    let connections = periphery_connections();

    let id = args.id.to_string();

    // Spawn client side connection if one doesn't exist.
    let Some(connection) = connections.get(&id).await else {
      if args.address.is_none() {
        return Err(anyhow!("Server {id} is not connected"));
      }
      return args
        .spawn_client_connection(id.clone(), insecure_tls)
        .await;
    };

    // Ensure the connection args are unchanged.
    if args.matches(&connection.args) {
      return Ok(PeripheryClient {
        id,
        responses: connection.responses.clone(),
      });
    }

    // The args have changed.
    if args.address.is_none() {
      // Periphery -> Core connection
      // Remove this connection, wait and see if client reconnects
      connections.remove(&id).await;
      tokio::time::sleep(Duration::from_millis(500)).await;
      let connection = connections
        .get(&id)
        .await
        .with_context(|| format!("Server {id} is not connected"))?;
      Ok(PeripheryClient {
        id,
        responses: connection.responses.clone(),
      })
    } else {
      // Core -> Periphery connection
      args.spawn_client_connection(id.clone(), insecure_tls).await
    }
  }

  pub async fn cleanup(self) -> Option<Arc<PeripheryConnection>> {
    periphery_connections().remove(&self.id).await
  }

  pub async fn health_check(&self) -> anyhow::Result<()> {
    self.request(api::GetHealth {}).await?;
    Ok(())
  }

  pub async fn request<T>(
    &self,
    request: T,
  ) -> anyhow::Result<T::Response>
  where
    T: std::fmt::Debug + Serialize + HasResponse,
    T::Response: DeserializeOwned,
  {
    self.request_inner(request, None).await
  }

  /// Like [Self::request], and adds the logs of commands
  /// Periphery sends while they run to the update.
  pub async fn request_with_live_logs<T>(
    &self,
    request: T,
    update: &Update,
  ) -> anyhow::Result<T::Response>
  where
    T: std::fmt::Debug + Serialize + HasResponse,
    T::Response: DeserializeOwned,
  {
    // Only an Update stored in the database can show them.
    if update.id.is_empty() {
      return self.request(request).await;
    }
    let (live_log, mut live_logs) = mpsc::channel(10);
    let res = self.request_inner(request, Some(live_log));
    tokio::pin!(res);
    let mut update = update.clone();
    let prev_len = update.logs.len();
    loop {
      tokio::select! {
        res = &mut res => {
          // The logs so far may be incomplete. Remove them, as on error
          // the Update may be finished from the one in the database.
          if res.is_err() && update.logs.len() > prev_len {
            update.logs.truncate(prev_len);
            let _ = update_update_without_send(&update).await;
          }
          return res;
        }
        Some(log) = live_logs.recv() => {
          // A command sends its log again as it grows, replace it.
          match update.logs[prev_len..].iter_mut().find(|prev| {
            prev.stage == log.stage && prev.start_ts == log.start_ts
          }) {
            Some(prev) => *prev = log,
            None => update.logs.push(log),
          }
          // Not sent to clients, the open Update polls while in progress.
          let _ = update_update_without_send(&update).await;
        }
      }
    }
  }

  async fn request_inner<T>(
    &self,
    request: T,
    live_log: Option<mpsc::Sender<Log>>,
  ) -> anyhow::Result<T::Response>
  where
    T: std::fmt::Debug + Serialize + HasResponse,
    T::Response: DeserializeOwned,
  {
    let connection =
      periphery_connections().get(&self.id).await.with_context(
        || format!("No connection found for server {}", self.id),
      )?;

    // Polls connected 3 times before bailing
    connection.bail_if_not_connected().await?;

    let channel_id = Uuid::new_v4();
    let (response_sender, mut response_receiever) = channel();
    self.responses.insert(channel_id, response_sender).await;

    if let Err(e) = connection
      .sender
      .send_request(
        channel_id,
        &json!({
          "type": T::req_type(),
          "params": request
        }),
      )
      .await
      .context("Failed to send request over channel")
    {
      self.responses.remove(&channel_id).await;
      return Err(e);
    }

    let res = async {
      // Poll for the associated response
      loop {
        let message = response_receiever
          .recv()
          // Periphery request handler sends pings every 4s
          // *on this channel specifically* so Core knows
          // request is being processed. Hardcoded 11s
          // allows for missed 5s ping due to network reconnect.
          .with_timeout(Duration::from_secs(10))
          .await?;

        match message.decode_response()? {
          Response::Ok(message) => return message.decode(),
          Response::Err(e) => return Err(e),
          // Just a ping from periphery request handler
          Response::Pending => continue,
          Response::Progress(log) => {
            if let Some(live_log) = &live_log
              && let Ok(log) = log.decode()
            {
              // Only for display, so dropped when the update is behind.
              let _ = live_log.try_send(log);
            }
          }
        }
      }
    }
    .await;

    self.responses.remove(&channel_id).await;

    res
  }
}
