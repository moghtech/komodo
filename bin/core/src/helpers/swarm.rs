use std::time::Duration;

use anyhow::anyhow;
use komodo_client::entities::server::{Server, ServerState};
use mogh_resolver::HasResponse;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
  helpers::periphery_client, resource, state::server_status_cache,
};

pub async fn swarm_request<T>(
  server_ids: &[String],
  request: T,
) -> anyhow::Result<T::Response>
where
  T: std::fmt::Debug + Clone + Serialize + HasResponse,
  T::Response: DeserializeOwned,
{
  swarm_request_custom_timeout(
    server_ids,
    request,
    Duration::from_secs(10),
  )
  .await
}

pub async fn swarm_request_custom_timeout<T>(
  server_ids: &[String],
  request: T,
  timeout: Duration,
) -> anyhow::Result<T::Response>
where
  T: std::fmt::Debug + Clone + Serialize + HasResponse,
  T::Response: DeserializeOwned,
{
  let status_cache = server_status_cache();
  let mut err = Option::<anyhow::Error>::None;
  for server_id in server_ids {
    let Some(ServerState::Ok) =
      status_cache.get(server_id).await.map(|status| status.state)
    else {
      err = Some(anyhow!("No managers connected"));
      continue;
    };
    let Ok(periphery) =
      (match resource::get::<Server>(server_id).await {
        Ok(server) => periphery_client(&server).await,
        Err(e) => {
          err = Some(e);
          continue;
        }
      })
    else {
      continue;
    };
    match periphery
      .request_custom_timeout(request.clone(), timeout)
      .await
    {
      Ok(res) => return Ok(res),
      Err(e) => err = Some(e),
    }
  }
  Err(
    err
      .unwrap_or_else(|| anyhow!("Unknown error"))
      .context("Failed to request from Swarm"),
  )
}
