use komodo_client::entities::{
  alert::SeverityLevel,
  deployment::{Deployment, DeploymentState},
  docker::{
    container::ContainerListItem, image::ImageListItem,
    network::NetworkListItem, volume::VolumeListItem,
  },
  repo::Repo,
  server::{
    Server, ServerConfig, ServerHealth, ServerHealthState,
    ServerState,
  },
  stack::{ComposeProject, Stack, StackState},
  stats::{SingleDiskUsage, SystemStats},
};
use serror::Serror;

use crate::state::{
  deployment_status_cache, repo_status_cache, server_status_cache,
  stack_status_cache,
};

use super::{
  CachedDeploymentStatus, CachedRepoStatus, CachedServerStatus,
  CachedStackStatus, History,
};

#[instrument(level = "debug", skip_all)]
pub async fn insert_deployments_status_unknown(
  deployments: Vec<Deployment>,
) {
  let status_cache = deployment_status_cache();
  for deployment in deployments {
    let prev =
      status_cache.get(&deployment.id).await.map(|s| s.curr.state);
    status_cache
      .insert(
        deployment.id.clone(),
        History {
          curr: CachedDeploymentStatus {
            id: deployment.id,
            state: DeploymentState::Unknown,
            container: None,
            update_available: false,
          },
          prev,
        }
        .into(),
      )
      .await;
  }
}

#[instrument(level = "debug", skip_all)]
pub async fn insert_repos_status_unknown(repos: Vec<Repo>) {
  let status_cache = repo_status_cache();
  for repo in repos {
    status_cache
      .insert(
        repo.id.clone(),
        CachedRepoStatus {
          latest_hash: None,
          latest_message: None,
        }
        .into(),
      )
      .await;
  }
}

#[instrument(level = "debug", skip_all)]
pub async fn insert_stacks_status_unknown(stacks: Vec<Stack>) {
  let status_cache = stack_status_cache();
  for stack in stacks {
    let prev =
      status_cache.get(&stack.id).await.map(|s| s.curr.state);
    status_cache
      .insert(
        stack.id.clone(),
        History {
          curr: CachedStackStatus {
            id: stack.id,
            state: StackState::Unknown,
            services: Vec::new(),
          },
          prev,
        }
        .into(),
      )
      .await;
  }
}

type DockerLists = (
  Option<Vec<ContainerListItem>>,
  Option<Vec<NetworkListItem>>,
  Option<Vec<ImageListItem>>,
  Option<Vec<VolumeListItem>>,
  Option<Vec<ComposeProject>>,
);

#[instrument(level = "debug", skip_all)]
pub async fn insert_server_status(
  server: &Server,
  state: ServerState,
  version: String,
  stats: Option<SystemStats>,
  (containers, networks, images, volumes, projects): DockerLists,
  err: impl Into<Option<Serror>>,
) {
  let health = stats.as_ref().map(|s| get_server_health(server, s));
  server_status_cache()
    .insert(
      server.id.clone(),
      CachedServerStatus {
        id: server.id.clone(),
        state,
        version,
        stats,
        health,
        containers,
        networks,
        images,
        volumes,
        projects,
        err: err.into(),
      }
      .into(),
    )
    .await;
}

const ALERT_PERCENTAGE_THRESHOLD: f32 = 5.0;

fn get_server_health(
  server: &Server,
  SystemStats {
    cpu_perc,
    mem_used_gb,
    mem_total_gb,
    disks,
    ..
  }: &SystemStats,
) -> ServerHealth {
  let ServerConfig {
    cpu_warning,
    cpu_critical,
    mem_warning,
    mem_critical,
    disk_warning,
    disk_critical,
    ..
  } = &server.config;
  let mut health = ServerHealth::default();

  if cpu_perc >= cpu_critical {
    health.cpu.level = SeverityLevel::Critical;
  } else if cpu_perc >= cpu_warning {
    health.cpu.level = SeverityLevel::Warning
  } else if *cpu_perc < cpu_warning - ALERT_PERCENTAGE_THRESHOLD {
    health.cpu.should_close_alert = true
  }

  let mem_perc = 100.0 * mem_used_gb / mem_total_gb;
  if mem_perc >= *mem_critical {
    health.mem.level = SeverityLevel::Critical
  } else if mem_perc >= *mem_warning {
    health.mem.level = SeverityLevel::Warning
  } else if mem_perc
    < mem_warning - (ALERT_PERCENTAGE_THRESHOLD as f64)
  {
    health.mem.should_close_alert = true
  }

  for SingleDiskUsage {
    mount,
    used_gb,
    total_gb,
    ..
  } in disks
  {
    let perc = 100.0 * used_gb / total_gb;
    let mut state = ServerHealthState::default();
    if perc >= *disk_critical {
      state.level = SeverityLevel::Critical;
    } else if perc >= *disk_warning {
      state.level = SeverityLevel::Warning;
    } else if perc
      < disk_warning - (ALERT_PERCENTAGE_THRESHOLD as f64)
    {
      state.should_close_alert = true;
    };
    health.disks.insert(mount.clone(), state);
  }

  health
}
