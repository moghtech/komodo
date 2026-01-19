use std::{
  cmp,
  collections::HashMap,
  sync::{Arc, OnceLock},
};

use anyhow::{Context, anyhow};
use async_timing_util::{
  FIFTEEN_SECONDS_MS, get_timelength_in_ms, unix_timestamp_ms,
};
use database::mungos::{
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use komodo_client::{
  api::read::*,
  entities::{
    ResourceTarget,
    deployment::Deployment,
    docker::{
      container::{
        Container, ContainerListItem, ContainerStateStatusEnum,
      },
      image::{Image, ImageHistoryResponseItem},
      network::Network,
      volume::Volume,
    },
    permission::PermissionLevel,
    server::{
      Server, ServerActionState, ServerListItem, ServerQuery,
      ServerState,
    },
    stack::{Stack, StackServiceNames},
    stats::{SystemInformation, SystemProcess},
    update::Log,
  },
};
use mogh_resolver::Resolve;
use periphery_client::api::{
  self as periphery,
  container::InspectContainer,
  docker::{
    ImageHistory, InspectImage, InspectNetwork, InspectVolume,
  },
};
use reqwest::StatusCode;
use mogh_error::AddStatusCode;
use tokio::sync::Mutex;

use crate::{
  helpers::{periphery_client, query::get_all_tags},
  permission::{get_check_permissions, list_resources_for_user},
  resource,
  stack::compose_container_match_regex,
  state::{action_states, db_client, server_status_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetServersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetServersSummaryResponse> {
    let servers = resource::list_for_user::<Server>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?;

    let core_version = env!("CARGO_PKG_VERSION");
    let mut res = GetServersSummaryResponse::default();

    for server in servers {
      res.total += 1;
      match server.info.state {
        ServerState::Ok => {
          // Check for version mismatch
          if matches!(&server.info.version, Some(version) if version != core_version)
          {
            res.warning += 1;
          } else {
            res.healthy += 1;
          }
        }
        ServerState::NotOk => {
          res.unhealthy += 1;
        }
        ServerState::Disabled => {
          if !server.template {
            res.disabled += 1;
          }
        }
      }
    }
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetServer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Server> {
    Ok(
      get_check_permissions::<Server>(
        &self.server,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListServers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<ServerListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Server>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullServers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullServersResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Server>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetServerState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetServerStateResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let status = server_status_cache()
      .get(&server.id)
      .await
      .ok_or(anyhow!("did not find cached status for server"))?;
    let response = GetServerStateResponse {
      status: status.state,
    };
    Ok(response)
  }
}

impl Resolve<ReadArgs> for GetServerActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ServerActionState> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .server
      .get(&server.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetPeripheryInformation {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetPeripheryInformationResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .periphery_info
      .as_ref()
      .cloned()
      .context("Server status missing Periphery Info. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

impl Resolve<ReadArgs> for GetSystemInformation {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<SystemInformation> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await
    .status_code(StatusCode::BAD_REQUEST)?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .system_info
      .as_ref()
      .cloned()
      .context("Server status missing system Info. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

impl Resolve<ReadArgs> for GetSystemStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetSystemStatsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .system_stats
      .as_ref()
      .cloned()
      .context("Server status missing system stats. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

// This protects the peripheries from spam requests
const PROCESSES_EXPIRY: u128 = FIFTEEN_SECONDS_MS;
type ProcessesCache =
  Mutex<HashMap<String, Arc<(Vec<SystemProcess>, u128)>>>;
fn processes_cache() -> &'static ProcessesCache {
  static PROCESSES_CACHE: OnceLock<ProcessesCache> = OnceLock::new();
  PROCESSES_CACHE.get_or_init(Default::default)
}

impl Resolve<ReadArgs> for ListSystemProcesses {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListSystemProcessesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.processes(),
    )
    .await?;
    let mut lock = processes_cache().lock().await;
    let res = match lock.get(&server.id) {
      Some(cached) if cached.1 > unix_timestamp_ms() => {
        cached.0.clone()
      }
      _ => {
        let stats = periphery_client(&server)
          .await?
          .request(periphery::stats::GetSystemProcesses {})
          .await?;
        lock.insert(
          server.id,
          (stats.clone(), unix_timestamp_ms() + PROCESSES_EXPIRY)
            .into(),
        );
        stats
      }
    };
    Ok(res)
  }
}

const STATS_PER_PAGE: i64 = 200;

impl Resolve<ReadArgs> for GetHistoricalServerStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetHistoricalServerStatsResponse> {
    let GetHistoricalServerStats {
      server,
      granularity,
      page,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let granularity =
      get_timelength_in_ms(granularity.to_string().parse().unwrap())
        as i64;
    let mut ts_vec = Vec::<i64>::new();
    let curr_ts = unix_timestamp_ms() as i64;
    let mut curr_ts = curr_ts
      - curr_ts % granularity
      - granularity * STATS_PER_PAGE * page as i64;
    for _ in 0..STATS_PER_PAGE {
      ts_vec.push(curr_ts);
      curr_ts -= granularity;
    }

    let stats = find_collect(
      &db_client().stats,
      doc! {
        "sid": server.id,
        "ts": { "$in": ts_vec },
      },
      FindOptions::builder()
        .sort(doc! { "ts": -1 })
        .skip(page as u64 * STATS_PER_PAGE as u64)
        .limit(STATS_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to pull stats from db")?;
    let next_page = if stats.len() == STATS_PER_PAGE as usize {
      Some(page + 1)
    } else {
      None
    };
    let res = GetHistoricalServerStatsResponse { stats, next_page };
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerContainers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerContainersResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.containers.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for ListAllDockerContainers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListAllDockerContainersResponse> {
    let servers = resource::list_for_user::<Server>(
      ServerQuery::builder().names(self.servers.clone()).build(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?;

    let mut containers = Vec::<ContainerListItem>::new();

    for server in servers {
      let cache = server_status_cache()
        .get_or_insert_default(&server.id)
        .await;
      let Some(docker) = &cache.docker else {
        continue;
      };
      let more = docker
        .containers
        .iter()
        .filter(|container| {
          self.containers.is_empty()
            || self.containers.contains(&container.name)
        })
        .cloned();
      containers.extend(more);
    }

    Ok(containers)
  }
}

impl Resolve<ReadArgs> for GetDockerContainersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetDockerContainersSummaryResponse> {
    let servers = resource::list_full_for_user::<Server>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get servers from db")?;

    let mut res = GetDockerContainersSummaryResponse::default();

    for server in servers {
      let cache = server_status_cache()
        .get_or_insert_default(&server.id)
        .await;

      if let Some(docker) = &cache.docker {
        for container in &docker.containers {
          res.total += 1;
          match container.state {
            ContainerStateStatusEnum::Created
            | ContainerStateStatusEnum::Paused
            | ContainerStateStatusEnum::Exited => res.stopped += 1,
            ContainerStateStatusEnum::Running => res.running += 1,
            ContainerStateStatusEnum::Empty => res.unknown += 1,
            _ => res.unhealthy += 1,
          }
        }
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for InspectDockerContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Container> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect container: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectContainer {
        name: self.container,
      })
      .await?;
    Ok(res)
  }
}

const MAX_LOG_LENGTH: u64 = 5000;

impl Resolve<ReadArgs> for GetContainerLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Log> {
    let GetContainerLog {
      server,
      container,
      tail,
      timestamps,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    let res = periphery_client(&server)
      .await?
      .request(periphery::container::GetContainerLog {
        name: container,
        tail: cmp::min(tail, MAX_LOG_LENGTH),
        timestamps,
      })
      .await
      .context("failed at call to periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for SearchContainerLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Log> {
    let SearchContainerLog {
      server,
      container,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    let res = periphery_client(&server)
      .await?
      .request(periphery::container::GetContainerLogSearch {
        name: container,
        terms,
        combinator,
        invert,
        timestamps,
      })
      .await
      .context("failed at call to periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetResourceMatchingContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetResourceMatchingContainerResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    // first check deployments
    if let Ok(deployment) =
      resource::get::<Deployment>(&self.container).await
    {
      return Ok(GetResourceMatchingContainerResponse {
        resource: ResourceTarget::Deployment(deployment.id).into(),
      });
    }

    // then check stacks
    let stacks = list_resources_for_user::<Stack>(
      doc! { "config.server_id": &server.id },
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    // check matching stack
    for stack in stacks {
      for StackServiceNames {
        service_name,
        container_name,
        ..
      } in stack
        .info
        .deployed_services
        .unwrap_or(stack.info.latest_services)
      {
        let is_match = match compose_container_match_regex(&container_name)
          .with_context(|| format!("failed to construct container name matching regex for service {service_name}")) 
        {
          Ok(regex) => regex,
          Err(e) => {
            warn!("{e:#}");
            continue;
          }
        }.is_match(&self.container);

        if is_match {
          return Ok(GetResourceMatchingContainerResponse {
            resource: ResourceTarget::Stack(stack.id).into(),
          });
        }
      }
    }

    Ok(GetResourceMatchingContainerResponse { resource: None })
  }
}

impl Resolve<ReadArgs> for ListDockerNetworks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerNetworksResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.networks.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerNetwork {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Network> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect network: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectNetwork { name: self.network })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImages {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerImagesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.images.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerImage {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Image> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!("Cannot inspect image: server is {:?}", cache.state)
          .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectImage { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImageHistory {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<ImageHistoryResponseItem>> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot get image history: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(ImageHistory { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerVolumes {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerVolumesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.volumes.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerVolume {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Volume> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!("Cannot inspect volume: server is {:?}", cache.state)
          .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectVolume { name: self.volume })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListComposeProjects {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListComposeProjectsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.projects.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

// impl Resolve<ReadArgs> for ListAllTerminals {
//   async fn resolve(
//     self,
//     args: &ReadArgs,
//   ) -> Result<Self::Response, Self::Error> {
//     // match self.tar
//     let mut terminals = resource::list_full_for_user::<Server>(
//       self.query, &args.user, &all_tags,
//     )
//     .await?
//     .into_iter()
//     .map(|server| async move {
//       (
//         list_terminals_inner(&server, self.fresh).await,
//         (server.id, server.name),
//       )
//     })
//     .collect::<FuturesUnordered<_>>()
//     .collect::<Vec<_>>()
//     .await
//     .into_iter()
//     .flat_map(|(terminals, server)| {
//       let terminals = terminals.ok()?;
//       Some((terminals, server))
//     })
//     .flat_map(|(terminals, (server_id, server_name))| {
//       terminals.into_iter().map(move |info| {
//         TerminalInfoWithServer::from_terminal_info(
//           &server_id,
//           &server_name,
//           info,
//         )
//       })
//     })
//     .collect::<Vec<_>>();

//     terminals.sort_by(|a, b| {
//       a.server_name.cmp(&b.server_name).then(a.name.cmp(&b.name))
//     });

//     Ok(terminals)
//   }
// }
