use std::sync::{Arc, OnceLock};

use async_timing_util::wait_until_timelength;
use database::mungos::{find::find_collect, mongodb::bson::doc};
use futures_util::future::join_all;
use helpers::insert_stacks_status_unknown;
use komodo_client::entities::{
  deployment::Deployment,
  komodo_timestamp, optional_string,
  repo::Repo,
  server::{Server, ServerState},
  stack::Stack,
  stats::{SystemProcess, SystemStats},
  swarm::Swarm,
};
use mogh_cache::CloneCache;
use mogh_error::Serror;
use periphery_client::api::{
  self, git::GetLatestCommit, poll::PollStatusResponse,
  stats::GetSystemProcesses,
};
use tokio::sync::Mutex;

use crate::{
  config::monitoring_interval,
  helpers::{matcher::Matcher, periphery_client},
  monitor::{alert::check_alerts, record::record_server_stats},
  state::{
    CachedRepoStatus, db_client, deployment_status_cache,
    periphery_connections, repo_status_cache,
  },
};

use self::helpers::{
  ALERT_PERCENTAGE_THRESHOLD, insert_deployments_status_unknown,
  insert_repos_status_unknown, insert_server_status,
};

mod alert;
mod helpers;
mod record;
mod resources;
mod swarm;

pub use swarm::refresh_swarm_cache;

const ADDITIONAL_MS: u128 = 500;

pub fn spawn_monitoring_loops() {
  spawn_server_monitoring_loop();
  swarm::spawn_swarm_monitoring_loop();
}

fn spawn_server_monitoring_loop() {
  tokio::spawn(async move {
    refresh_all_server_cache().await;
    let interval = monitoring_interval();
    loop {
      let ts = (wait_until_timelength(interval, ADDITIONAL_MS).await
        - ADDITIONAL_MS) as i64;
      refresh_all_server_cache().await;
      tokio::join!(check_alerts(ts), record_server_stats(ts));
    }
  });
}

async fn refresh_all_server_cache() {
  let servers =
    match find_collect(&db_client().servers, None, None).await {
      Ok(servers) => servers,
      Err(e) => {
        error!(
          "Failed to get server list (refresh server cache) | {e:#}"
        );
        return;
      }
    };
  let futures = servers.into_iter().map(|server| async move {
    refresh_server_cache(&server, false).await;
  });
  join_all(futures).await;
}

/// Makes sure cache for server doesn't update too frequently / simultaneously.
/// If forced, will still block against simultaneous update.
fn refresh_server_cache_controller()
-> &'static CloneCache<String, Arc<Mutex<i64>>> {
  static CACHE: OnceLock<CloneCache<String, Arc<Mutex<i64>>>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// The background loop will call this with force: false,
/// which exits early if the lock is busy or it was completed too recently.
/// If force is true, it will wait on simultaneous calls, and will
/// ignore the restriction on being completed too recently.
///
/// Returns impl Future for explicit Send bound, as this function
/// calls `periphery_client` which in some paths calls back this function `refresh_server_cache`.
/// While infinite recursion is prevented, the compiler needs some help to know the Send bound.
///
/// Clippy is not aware of this requirement, so the `manual_async_fn` lint is allowed here.
#[allow(clippy::manual_async_fn)]
pub fn refresh_server_cache(
  server: &Server,
  force: bool,
) -> impl Future<Output = ()> + Send + '_ {
  async move {
    // Concurrency controller to ensure it isn't done too often
    // when it happens in other contexts.
    let controller = refresh_server_cache_controller()
      .get_or_insert_default(&server.id)
      .await;
    let mut lock = match controller.try_lock() {
      Ok(lock) => lock,
      Err(_) if force => controller.lock().await,
      Err(_) => return,
    };

    let now = komodo_timestamp();

    // early return if called again sooner than 1s.
    if !force && *lock > now - 1_000 {
      return;
    }

    *lock = now;

    let resources = RefreshCacheResources::load_server(server).await;

    // Handle server disabled
    if !server.config.enabled {
      resources.insert_status_unknown().await;
      insert_server_status(
        server,
        ServerState::Disabled,
        None,
        None,
        None,
        0.0,
        None,
        None,
      )
      .await;
      periphery_connections().remove(&server.id).await;
      return;
    }

    let periphery = match periphery_client(server).await {
      Ok(periphery) => periphery,
      Err(e) => {
        resources.insert_status_unknown().await;
        insert_server_status(
          server,
          ServerState::NotOk,
          None,
          None,
          None,
          0.0,
          None,
          Serror::from(&e),
        )
        .await;
        return;
      }
    };

    let PollStatusResponse {
      periphery_info,
      system_info,
      system_stats,
      mut docker,
    } = match periphery
      .request(api::poll::PollStatus {
        include_stats: server.config.stats_monitoring,
        include_docker: true,
      })
      .await
    {
      Ok(info) => info,
      Err(e) => {
        resources.insert_status_unknown().await;
        insert_server_status(
          server,
          ServerState::NotOk,
          None,
          None,
          None,
          0.0,
          None,
          Serror::from(&e),
        )
        .await;
        return;
      }
    };

    if let Some(docker) = &mut docker {
      docker.containers.iter_mut().for_each(|container| {
        container.server_id = Some(server.id.clone());
        container.server_name = Some(server.name.clone());
      });
    }

    let containers = docker
      .as_ref()
      .map(|docker| docker.containers.as_slice())
      .unwrap_or(&[]);
    let images = docker
      .as_ref()
      .map(|docker| docker.images.as_slice())
      .unwrap_or(&[]);

    tokio::join!(
      resources::update_server_stack_cache(
        resources.stacks,
        containers,
        images
      ),
      resources::update_server_deployment_cache(
        resources.deployments,
        containers,
        images,
      ),
    );

    let ignored_cpu_perc = match &system_stats {
      Some(stats) if should_check_ignored_cpu(server, stats) => {
        match periphery.request(GetSystemProcesses {}).await {
          Ok(processes) => ignored_cpu_perc(
            &processes,
            &server.config.ignore_cpu_processes,
            system_info.logical_core_count,
          )
          .min(stats.cpu_perc),
          Err(e) => {
            warn!(
              "Failed to get processes for ignored cpu | server: {} | {e:#}",
              server.name
            );
            0.0
          }
        }
      }
      _ => 0.0,
    };

    insert_server_status(
      server,
      ServerState::Ok,
      Some(periphery_info),
      Some(system_info),
      system_stats.map(|stats| filter_volumes(server, stats)),
      ignored_cpu_perc,
      docker,
      None,
    )
    .await;

    let status_cache = repo_status_cache();
    for repo in resources.repos {
      let (latest_hash, latest_message) = periphery
        .request(GetLatestCommit {
          name: repo.name.clone(),
          path: optional_string(&repo.config.path),
        })
        .await
        .ok()
        .flatten()
        .map(|c| (c.hash, c.message))
        .unzip();
      status_cache
        .insert(
          repo.id,
          CachedRepoStatus {
            latest_hash,
            latest_message,
          }
          .into(),
        )
        .await;
    }
  }
}

struct RefreshCacheResources {
  stacks: Vec<Stack>,
  deployments: Vec<Deployment>,
  repos: Vec<Repo>,
}

impl RefreshCacheResources {
  pub async fn load_server(server: &Server) -> Self {
    let (stacks, deployments, repos) = tokio::join!(
      find_collect(
        &db_client().stacks,
        doc! { "config.server_id": &server.id },
        None,
      ),
      find_collect(
        &db_client().deployments,
        doc! { "config.server_id": &server.id },
        None,
      ),
      find_collect(
        &db_client().repos,
        doc! { "config.server_id": &server.id },
        None,
      ),
    );

    let stacks = stacks.inspect_err(|e|  error!("Failed to get stacks list from db (update server status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();
    let deployments =  deployments.inspect_err(|e| error!("Failed to get deployments list from db (update server status cache) | server : {} | {e:#}", server.name)).unwrap_or_default();
    let repos = repos.inspect_err(|e|  error!("Failed to get repos list from db (update server status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();

    Self {
      stacks,
      deployments,
      repos,
    }
  }

  pub async fn load_swarm(swarm: &Swarm) -> Self {
    let (stacks, deployments) = tokio::join!(
      find_collect(
        &db_client().stacks,
        doc! { "config.swarm_id": &swarm.id },
        None,
      ),
      find_collect(
        &db_client().deployments,
        doc! { "config.swarm_id": &swarm.id },
        None,
      ),
    );

    let stacks = stacks.inspect_err(|e|  error!("Failed to get stacks list from db (update swarm status cache) | swarm: {} | {e:#}", swarm.name)).unwrap_or_default();
    let deployments =  deployments.inspect_err(|e| error!("Failed to get deployments list from db (update swarm status cache) | swarm : {} | {e:#}", swarm.name)).unwrap_or_default();

    Self {
      stacks,
      deployments,
      repos: Default::default(),
    }
  }

  pub async fn insert_status_unknown(self) {
    insert_stacks_status_unknown(self.stacks).await;
    insert_deployments_status_unknown(self.deployments).await;
    insert_repos_status_unknown(self.repos).await;
  }
}

fn filter_volumes(
  server: &Server,
  mut stats: SystemStats,
) -> SystemStats {
  stats.disks.retain(|disk| {
    // Always filter out volume mounts
    !disk.mount.starts_with("/var/lib/docker/volumes")
    // Filter out any that were declared to ignore in server config
      && !server
        .config
        .ignore_mounts
        .iter()
        .any(|mount| disk.mount.starts_with(mount))
  });
  stats
}

/// Below this, subtracting ignored processes can't change alert state.
fn should_check_ignored_cpu(
  server: &Server,
  stats: &SystemStats,
) -> bool {
  server.config.send_cpu_alerts
    && !server.config.ignore_cpu_processes.is_empty()
    && stats.cpu_perc
      >= server.config.cpu_warning - ALERT_PERCENTAGE_THRESHOLD
}

/// Process cpu is per core (100% = 1 core), host cpu is 0-100%.
fn ignored_cpu_perc(
  processes: &[SystemProcess],
  patterns: &[String],
  logical_core_count: Option<u32>,
) -> f32 {
  let matchers = patterns
    .iter()
    .filter(|p| !p.is_empty())
    .filter_map(|p| {
      Matcher::new(p)
        .inspect_err(|e| {
          warn!("Invalid ignore_cpu_processes | {e:#}")
        })
        .ok()
    })
    .collect::<Vec<_>>();
  let total = processes
    .iter()
    .filter(|p| {
      matchers
        .iter()
        .any(|m| m.is_match(&p.name) || m.is_match(&p.exe))
    })
    .map(|p| p.cpu_perc)
    .sum::<f32>();
  total / logical_core_count.unwrap_or(1).max(1) as f32
}

#[cfg(test)]
mod tests {
  use super::*;

  fn process(name: &str, exe: &str, cpu_perc: f32) -> SystemProcess {
    SystemProcess {
      pid: 0,
      name: name.to_string(),
      exe: exe.to_string(),
      cmd: Vec::new(),
      start_time: 0.0,
      cpu_perc,
      mem_mb: 0.0,
      disk_read_kb: 0.0,
      disk_write_kb: 0.0,
    }
  }

  fn patterns(p: &[&str]) -> Vec<String> {
    p.iter().map(|s| s.to_string()).collect()
  }

  fn processes() -> Vec<SystemProcess> {
    vec![
      process("ffmpeg", "/usr/bin/ffmpeg", 300.0),
      process("ffprobe", "/usr/bin/ffprobe", 100.0),
      process("dockerd", "/usr/bin/dockerd", 50.0),
      process(
        "very-long-proc",
        "/opt/app/very-long-process-name",
        20.0,
      ),
    ]
  }

  #[test]
  fn no_patterns_ignores_nothing() {
    assert_eq!(ignored_cpu_perc(&processes(), &[], Some(4)), 0.0);
    assert_eq!(
      ignored_cpu_perc(&processes(), &patterns(&[""]), Some(4)),
      0.0
    );
  }

  #[test]
  fn exact_name_scaled_by_cores() {
    assert_eq!(
      ignored_cpu_perc(&processes(), &patterns(&["ffmpeg"]), Some(4)),
      75.0
    );
  }

  #[test]
  fn wildcard_and_regex() {
    assert_eq!(
      ignored_cpu_perc(&processes(), &patterns(&["ff*"]), Some(4)),
      100.0
    );
    assert_eq!(
      ignored_cpu_perc(
        &processes(),
        &patterns(&["\\^ffprobe$\\"]),
        Some(4)
      ),
      25.0
    );
  }

  #[test]
  fn matches_exe_path() {
    assert_eq!(
      ignored_cpu_perc(
        &processes(),
        &patterns(&["*very-long-process-name"]),
        Some(1)
      ),
      20.0
    );
  }

  #[test]
  fn invalid_pattern_skipped() {
    assert_eq!(
      ignored_cpu_perc(
        &processes(),
        &patterns(&["\\(\\", "dockerd"]),
        Some(2)
      ),
      25.0
    );
  }

  #[test]
  fn missing_core_count_no_panic() {
    assert_eq!(
      ignored_cpu_perc(&processes(), &patterns(&["dockerd"]), None),
      50.0
    );
    assert_eq!(
      ignored_cpu_perc(
        &processes(),
        &patterns(&["dockerd"]),
        Some(0)
      ),
      50.0
    );
  }
}
