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
    komodo_timestamp,
    permission::PermissionLevel,
    server::{
      Server, ServerActionState, ServerListItem, ServerState,
      TerminalInfo,
    },
    stack::{Stack, StackServiceNames},
    stats::{SystemInformation, SystemProcess},
    update::Log,
  },
};
use periphery_client::api::{
  self as periphery,
  container::InspectContainer,
  image::{ImageHistory, InspectImage},
  network::InspectNetwork,
  volume::InspectVolume,
};
use resolver_api::Resolve;
use tokio::sync::Mutex;

use crate::{
  helpers::{
    periphery_client,
    query::{get_all_tags, get_system_info},
  },
  permission::get_check_permissions,
  resource,
  stack::compose_container_match_regex,
  state::{action_states, db_client, server_status_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetServersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetServersSummaryResponse> {
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
          let has_version_mismatch = !server.info.version.is_empty()
            && server.info.version != "Unknown"
            && server.info.version != core_version;

          if has_version_mismatch {
            res.warning += 1;
          } else {
            res.healthy += 1;
          }
        }
        ServerState::NotOk => {
          res.unhealthy += 1;
        }
        ServerState::Disabled => {
          res.disabled += 1;
        }
      }
    }
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetPeripheryVersion {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetPeripheryVersionResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let version = server_status_cache()
      .get(&server.id)
      .await
      .map(|s| s.version.clone())
      .unwrap_or(String::from("unknown"));
    Ok(GetPeripheryVersionResponse { version })
  }
}

impl Resolve<ReadArgs> for GetServer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Server> {
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
  ) -> serror::Result<Vec<ServerListItem>> {
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
  ) -> serror::Result<ListFullServersResponse> {
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
  ) -> serror::Result<GetServerStateResponse> {
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
  ) -> serror::Result<ServerActionState> {
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

impl Resolve<ReadArgs> for GetSystemInformation {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<SystemInformation> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    get_system_info(&server).await.map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for GetSystemStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetSystemStatsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let status =
      server_status_cache().get(&server.id).await.with_context(
        || format!("did not find status for server at {}", server.id),
      )?;
    let stats = status
      .stats
      .as_ref()
      .context("server stats not available")?;
    Ok(stats.clone())
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
  ) -> serror::Result<ListSystemProcessesResponse> {
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
        let stats = periphery_client(&server)?
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
  ) -> serror::Result<GetHistoricalServerStatsResponse> {
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
  ) -> serror::Result<ListDockerContainersResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(containers) = &cache.containers {
      Ok(containers.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for ListAllDockerContainers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListAllDockerContainersResponse> {
    let servers = resource::list_for_user::<Server>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?
    .into_iter()
    .filter(|server| {
      self.servers.is_empty()
        || self.servers.contains(&server.id)
        || self.servers.contains(&server.name)
    });

    let mut containers = Vec::<ContainerListItem>::new();

    for server in servers {
      let cache = server_status_cache()
        .get_or_insert_default(&server.id)
        .await;
      if let Some(more_containers) = &cache.containers {
        containers.extend(more_containers.clone());
      }
    }

    Ok(containers)
  }
}

impl Resolve<ReadArgs> for GetDockerContainersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetDockerContainersSummaryResponse> {
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

      if let Some(containers) = &cache.containers {
        for container in containers {
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
  ) -> serror::Result<Container> {
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
    let res = periphery_client(&server)?
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
  ) -> serror::Result<Log> {
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
    let res = periphery_client(&server)?
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
  ) -> serror::Result<Log> {
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
    let res = periphery_client(&server)?
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
  ) -> serror::Result<GetResourceMatchingContainerResponse> {
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
    let stacks =
      resource::list_full_for_user_using_document::<Stack>(
        doc! { "config.server_id": &server.id },
        user,
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
  ) -> serror::Result<ListDockerNetworksResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(networks) = &cache.networks {
      Ok(networks.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerNetwork {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Network> {
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
    let res = periphery_client(&server)?
      .request(InspectNetwork { name: self.network })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImages {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListDockerImagesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(images) = &cache.images {
      Ok(images.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerImage {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Image> {
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
    let res = periphery_client(&server)?
      .request(InspectImage { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImageHistory {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<ImageHistoryResponseItem>> {
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
    let res = periphery_client(&server)?
      .request(ImageHistory { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerVolumes {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListDockerVolumesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(volumes) = &cache.volumes {
      Ok(volumes.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerVolume {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Volume> {
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
    let res = periphery_client(&server)?
      .request(InspectVolume { name: self.volume })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListComposeProjects {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListComposeProjectsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(projects) = &cache.projects {
      Ok(projects.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

#[derive(Default)]
struct TerminalCacheItem {
  list: Vec<TerminalInfo>,
  ttl: i64,
}

const TERMINAL_CACHE_TIMEOUT: i64 = 30_000;

#[derive(Default)]
struct TerminalCache(
  std::sync::Mutex<
    HashMap<String, Arc<tokio::sync::Mutex<TerminalCacheItem>>>,
  >,
);

impl TerminalCache {
  fn get_or_insert(
    &self,
    server_id: String,
  ) -> Arc<tokio::sync::Mutex<TerminalCacheItem>> {
    if let Some(cached) =
      self.0.lock().unwrap().get(&server_id).cloned()
    {
      return cached;
    }
    let to_cache =
      Arc::new(tokio::sync::Mutex::new(TerminalCacheItem::default()));
    self.0.lock().unwrap().insert(server_id, to_cache.clone());
    to_cache
  }
}

fn terminals_cache() -> &'static TerminalCache {
  static TERMINALS: OnceLock<TerminalCache> = OnceLock::new();
  TERMINALS.get_or_init(Default::default)
}

impl Resolve<ReadArgs> for ListTerminals {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListTerminalsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.terminal(),
    )
    .await?;
    let cache = terminals_cache().get_or_insert(server.id.clone());
    let mut cache = cache.lock().await;
    if self.fresh || komodo_timestamp() > cache.ttl {
      cache.list = periphery_client(&server)?
        .request(periphery_client::api::terminal::ListTerminals {})
        .await
        .context("Failed to get fresh terminal list")?;
      cache.ttl = komodo_timestamp() + TERMINAL_CACHE_TIMEOUT;
      Ok(cache.list.clone())
    } else {
      Ok(cache.list.clone())
    }
  }
}
