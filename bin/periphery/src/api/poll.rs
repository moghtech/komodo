use futures_util::FutureExt;
use komodo_client::entities::{
  docker::DockerLists, server::PeripheryInformation,
};
use periphery_client::api::poll::{PollStatus, PollStatusResponse};
use resolver_api::Resolve;

use crate::{
  config::periphery_config,
  docker::{DockerClient, compose::list_compose_projects},
  state::{
    docker_client, host_public_ip, periphery_keys, stats_client,
  },
};

impl Resolve<crate::api::Args> for PollStatus {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<PollStatusResponse> {
    let stats_client = stats_client().read().await;

    let system_stats = if self.include_stats {
      Some(stats_client.stats.clone())
    } else {
      None
    };

    let docker = if self.include_docker {
      let client = docker_client().load();
      if let Some(client) = client.iter().next() {
        Some(docker_lists(client).await)
      } else {
        None
      }
    } else {
      None
    };

    Ok(PollStatusResponse {
      periphery_info: periphery_information().await,
      system_info: stats_client.info.clone(),
      system_stats,
      docker,
    })
  }
}

async fn periphery_information() -> PeripheryInformation {
  let config = periphery_config();
  PeripheryInformation {
    version: env!("CARGO_PKG_VERSION").to_string(),
    public_key: periphery_keys().load().public.to_string(),
    terminals_disabled: config.disable_terminals,
    container_terminals_disabled: config.disable_container_terminals,
    stats_polling_rate: config.stats_polling_rate,
    docker_connected: docker_client().load().is_some(),
    public_ip: host_public_ip().await.cloned(),
  }
}

async fn docker_lists(client: &DockerClient) -> DockerLists {
  let containers = client.list_containers().await.unwrap_or_default();
  let (networks, images, volumes, projects) = tokio::join!(
    client
      .list_networks(&containers)
      .map(Result::unwrap_or_default),
    client
      .list_images(&containers)
      .map(Result::unwrap_or_default),
    client
      .list_volumes(&containers)
      .map(Result::unwrap_or_default),
    list_compose_projects().map(Result::unwrap_or_default),
  );
  DockerLists {
    containers,
    networks,
    images,
    volumes,
    projects,
  }
}
