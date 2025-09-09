use std::sync::{Arc, OnceLock};

use async_timing_util::wait_until_timelength;
use database::mungos::{find::find_collect, mongodb::bson::doc};
use futures::future::join_all;
use helpers::insert_stacks_status_unknown;
use komodo_client::entities::{
  deployment::DeploymentState,
  docker::{
    container::ContainerListItem, image::ImageListItem,
    network::NetworkListItem, volume::VolumeListItem,
  },
  komodo_timestamp, optional_string,
  server::{Server, ServerHealth, ServerState},
  stack::{ComposeProject, StackService, StackState},
  stats::SystemStats,
};
use periphery_client::api::{self, git::GetLatestCommit};
use serror::Serror;
use tokio::sync::Mutex;

use crate::{
  config::core_config,
  helpers::{cache::Cache, periphery_client},
  monitor::{alert::check_alerts, record::record_server_stats},
  state::{db_client, deployment_status_cache, repo_status_cache},
};

use self::helpers::{
  insert_deployments_status_unknown, insert_repos_status_unknown,
  insert_server_status,
};

mod alert;
mod helpers;
mod lists;
mod record;
mod resources;

#[derive(Default, Debug)]
pub struct History<Curr: Default, Prev> {
  pub curr: Curr,
  pub prev: Option<Prev>,
}

#[derive(Default, Clone, Debug)]
pub struct CachedServerStatus {
  pub id: String,
  pub state: ServerState,
  pub version: String,
  pub stats: Option<SystemStats>,
  pub health: Option<ServerHealth>,
  pub containers: Option<Vec<ContainerListItem>>,
  pub networks: Option<Vec<NetworkListItem>>,
  pub images: Option<Vec<ImageListItem>>,
  pub volumes: Option<Vec<VolumeListItem>>,
  pub projects: Option<Vec<ComposeProject>>,
  /// Store the error in reaching periphery
  pub err: Option<serror::Serror>,
}

#[derive(Default, Clone, Debug)]
pub struct CachedDeploymentStatus {
  /// The deployment id
  pub id: String,
  pub state: DeploymentState,
  pub container: Option<ContainerListItem>,
  pub update_available: bool,
}

#[derive(Default, Clone, Debug)]
pub struct CachedRepoStatus {
  pub latest_hash: Option<String>,
  pub latest_message: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct CachedStackStatus {
  /// The stack id
  pub id: String,
  /// The stack state
  pub state: StackState,
  /// The services connected to the stack
  pub services: Vec<StackService>,
}

const ADDITIONAL_MS: u128 = 500;

pub fn spawn_monitor_loop() {
  let interval: async_timing_util::Timelength = core_config()
    .monitoring_interval
    .try_into()
    .expect("Invalid monitoring interval");
  tokio::spawn(async move {
    refresh_server_cache(komodo_timestamp()).await;
    loop {
      let ts = (wait_until_timelength(interval, ADDITIONAL_MS).await
        - ADDITIONAL_MS) as i64;
      refresh_server_cache(ts).await;
    }
  });
}

async fn refresh_server_cache(ts: i64) {
  let servers =
    match find_collect(&db_client().servers, None, None).await {
      Ok(servers) => servers,
      Err(e) => {
        error!(
          "failed to get server list (manage status cache) | {e:#}"
        );
        return;
      }
    };
  let futures = servers.into_iter().map(|server| async move {
    update_cache_for_server(&server, false).await;
  });
  join_all(futures).await;
  tokio::join!(check_alerts(ts), record_server_stats(ts));
}

/// Makes sure cache for server doesn't update too frequently / simultaneously.
/// If forced, will still block against simultaneous update.
fn update_cache_for_server_controller()
-> &'static Cache<String, Arc<Mutex<i64>>> {
  static CACHE: OnceLock<Cache<String, Arc<Mutex<i64>>>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// The background loop will call this with force: false,
/// which exits early if the lock is busy or it was completed too recently.
/// If force is true, it will wait on simultaneous calls, and will
/// ignore the restriction on being completed too recently.
#[instrument(level = "debug")]
pub async fn update_cache_for_server(server: &Server, force: bool) {
  // Concurrency controller to ensure it isn't done too often
  // when it happens in other contexts.
  let controller = update_cache_for_server_controller()
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

  let (deployments, builds, repos, stacks) = tokio::join!(
    find_collect(
      &db_client().deployments,
      doc! { "config.server_id": &server.id },
      None,
    ),
    find_collect(&db_client().builds, doc! {}, None,),
    find_collect(
      &db_client().repos,
      doc! { "config.server_id": &server.id },
      None,
    ),
    find_collect(
      &db_client().stacks,
      doc! { "config.server_id": &server.id },
      None,
    )
  );

  let deployments =  deployments.inspect_err(|e| error!("failed to get deployments list from db (update status cache) | server : {} | {e:#}", server.name)).unwrap_or_default();
  let builds =  builds.inspect_err(|e| error!("failed to get builds list from db (update status cache) | server : {} | {e:#}", server.name)).unwrap_or_default();
  let repos = repos.inspect_err(|e|  error!("failed to get repos list from db (update status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();
  let stacks = stacks.inspect_err(|e|  error!("failed to get stacks list from db (update status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();

  // Handle server disabled
  if !server.config.enabled {
    insert_deployments_status_unknown(deployments).await;
    insert_stacks_status_unknown(stacks).await;
    insert_repos_status_unknown(repos).await;
    insert_server_status(
      server,
      ServerState::Disabled,
      String::from("unknown"),
      None,
      (None, None, None, None, None),
      None,
    )
    .await;
    return;
  }

  let Ok(periphery) = periphery_client(server) else {
    error!(
      "somehow periphery not ok to create. should not be reached."
    );
    return;
  };

  let version = match periphery.request(api::GetVersion {}).await {
    Ok(version) => version.version,
    Err(e) => {
      insert_deployments_status_unknown(deployments).await;
      insert_stacks_status_unknown(stacks).await;
      insert_repos_status_unknown(repos).await;
      insert_server_status(
        server,
        ServerState::NotOk,
        String::from("Unknown"),
        None,
        (None, None, None, None, None),
        Serror::from(&e),
      )
      .await;
      return;
    }
  };

  let stats = if server.config.stats_monitoring {
    match periphery.request(api::stats::GetSystemStats {}).await {
      Ok(stats) => Some(filter_volumes(server, stats)),
      Err(e) => {
        insert_deployments_status_unknown(deployments).await;
        insert_stacks_status_unknown(stacks).await;
        insert_repos_status_unknown(repos).await;
        insert_server_status(
          server,
          ServerState::NotOk,
          String::from("unknown"),
          None,
          (None, None, None, None, None),
          Serror::from(&e),
        )
        .await;
        return;
      }
    }
  } else {
    None
  };

  match lists::get_docker_lists(&periphery).await {
    Ok((mut containers, networks, images, volumes, projects)) => {
      containers.iter_mut().for_each(|container| {
        container.server_id = Some(server.id.clone())
      });
      tokio::join!(
        resources::update_deployment_cache(
          server.name.clone(),
          deployments,
          &containers,
          &images,
          &builds,
        ),
        resources::update_stack_cache(
          server.name.clone(),
          stacks,
          &containers,
          &images
        ),
      );
      insert_server_status(
        server,
        ServerState::Ok,
        version,
        stats,
        (
          Some(containers.clone()),
          Some(networks),
          Some(images),
          Some(volumes),
          Some(projects),
        ),
        None,
      )
      .await;
    }
    Err(e) => {
      insert_deployments_status_unknown(deployments).await;
      insert_stacks_status_unknown(stacks).await;
      insert_server_status(
        server,
        ServerState::Ok,
        version,
        stats,
        (None, None, None, None, None),
        Some(e.into()),
      )
      .await;
    }
  }

  let status_cache = repo_status_cache();
  for repo in repos {
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
