use std::{sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use encoding::Decode as _;
use mogh_resolver::HasResponse;
use periphery_client::api;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use transport::channel::channel;
use uuid::Uuid;

use crate::{
  connection::{
    PeripheryConnection, PeripheryConnectionArgs, ResponseChannels,
  },
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
    self
      .request_custom_timeout(request, Duration::from_secs(10))
      .await
  }

  pub async fn request_custom_timeout<T>(
    &self,
    request: T,
    timeout: Duration,
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
        let message =
          response_receiever.recv().with_timeout(timeout).await?;

        // Still in progress, sent to avoid timeout.
        let Some(message) = message.decode()? else {
          continue;
        };

        return message.decode();
      }
    }
    .await;

    self.responses.remove(&channel_id).await;

    res
  }
}
