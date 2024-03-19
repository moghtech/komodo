use async_timing_util::{wait_until_timelength, Timelength};
use futures::future::join_all;
use monitor_client::entities::{
  deployment::{ContainerSummary, DockerContainerState},
  server::{
    stats::{AllSystemStats, ServerHealth},
    Server, ServerStatus,
  },
};
use mungos::{find::find_collect, mongodb::bson::doc};
use periphery_client::requests;

use crate::{
  db_client, helpers::{cache::deployment_status_cache, periphery_client}, state::State,
};

mod alert;
mod helpers;
mod record;

#[derive(Default)]
pub struct History<Curr: Default, Prev> {
  pub curr: Curr,
  pub prev: Option<Prev>,
}

#[derive(Default, Clone)]
pub struct CachedServerStatus {
  pub id: String,
  pub status: ServerStatus,
  pub version: String,
  pub stats: Option<AllSystemStats>,
  pub health: Option<ServerHealth>,
}

#[derive(Default, Clone)]
pub struct CachedDeploymentStatus {
  pub id: String,
  pub state: DockerContainerState,
  pub container: Option<ContainerSummary>,
}

pub fn spawn_monitor_loop() {
  tokio::spawn(async move {
    loop {
      let ts = (wait_until_timelength(Timelength::FiveSeconds, 500)
        .await
        - 500) as i64;
      let servers =
        match find_collect(&db_client().await.servers, None, None)
          .await
        {
          Ok(servers) => servers,
          Err(e) => {
            error!(
          "failed to get server list (manage status cache) | {e:#?}"
        );
            continue;
          }
        };
      let futures = servers.into_iter().map(|server| async move {
        update_cache_for_server(&server).await;
      });
      join_all(futures).await;
      tokio::join!(
        State.check_alerts(ts),
        State.record_server_stats(ts)
      );
    }
  });
}

pub async fn update_cache_for_server(server: &Server) {
  let deployments = find_collect(
    &db_client().await.deployments,
    doc! { "config.server_id": &server.id },
    None,
  )
  .await;
  if let Err(e) = &deployments {
    error!("failed to get deployments list from mongo (update status cache) | server id: {} | {e:#?}", server.id);
    return;
  }
  let deployments = deployments.unwrap();
  if !server.config.enabled {
    State.insert_deployments_status_unknown(deployments).await;
    State
      .insert_server_status(
        server,
        ServerStatus::Disabled,
        String::from("unknown"),
        None,
      )
      .await;
    return;
  }
  // already handle server disabled case above, so using unwrap here
  let periphery = periphery_client(server).unwrap();
  let version = periphery.request(requests::GetVersion {}).await;
  if version.is_err() {
    State.insert_deployments_status_unknown(deployments).await;
    State
      .insert_server_status(
        server,
        ServerStatus::NotOk,
        String::from("unknown"),
        None,
      )
      .await;
    return;
  }
  let stats = periphery.request(requests::GetAllSystemStats {}).await;
  if stats.is_err() {
    State.insert_deployments_status_unknown(deployments).await;
    State
      .insert_server_status(
        server,
        ServerStatus::NotOk,
        String::from("unknown"),
        None,
      )
      .await;
    return;
  }
  let stats = stats.unwrap();
  State
    .insert_server_status(
      server,
      ServerStatus::Ok,
      version.unwrap().version,
      stats.into(),
    )
    .await;
  let containers =
    periphery.request(requests::GetContainerList {}).await;
  if containers.is_err() {
    State.insert_deployments_status_unknown(deployments).await;
    return;
  }
  let containers = containers.unwrap();
  let status_cache = deployment_status_cache();
  for deployment in deployments {
    let container = containers
      .iter()
      .find(|c| c.name == deployment.name)
      .cloned();
    let prev =
      status_cache.get(&deployment.id).await.map(|s| s.curr.state);
    let state = container
      .as_ref()
      .map(|c| c.state)
      .unwrap_or(DockerContainerState::NotDeployed);
    status_cache
      .insert(
        deployment.id.clone(),
        History {
          curr: CachedDeploymentStatus {
            id: deployment.id,
            state,
            container,
          },
          prev,
        }
        .into(),
      )
      .await;
  }
}
