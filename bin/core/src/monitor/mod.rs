use std::sync::{Arc, OnceLock};

use async_timing_util::wait_until_timelength;
use cache::CloneCache;
use database::mungos::{find::find_collect, mongodb::bson::doc};
use futures_util::future::join_all;
use helpers::insert_stacks_status_unknown;
use komodo_client::entities::{
  build::Build,
  deployment::Deployment,
  komodo_timestamp, optional_string,
  repo::Repo,
  server::{Server, ServerState},
  stack::Stack,
  stats::SystemStats,
  swarm::Swarm,
};
use periphery_client::api::{
  self, git::GetLatestCommit, poll::PollStatusResponse,
};
use serror::Serror;
use tokio::sync::Mutex;

use crate::{
  config::monitoring_interval,
  helpers::periphery_client,
  monitor::{alert::check_alerts, record::record_server_stats},
  state::{
    CachedRepoStatus, db_client, deployment_status_cache,
    periphery_connections, repo_status_cache,
  },
};

use self::helpers::{
  insert_deployments_status_unknown, insert_repos_status_unknown,
  insert_server_status,
};

mod alert;
mod helpers;
mod record;
mod resources;
mod swarm;

pub use swarm::update_cache_for_swarm;

const ADDITIONAL_MS: u128 = 500;

pub fn spawn_monitoring_loops() {
  spawn_server_monitoring_loop();
  swarm::spawn_swarm_monitoring_loop();
}

fn spawn_server_monitoring_loop() {
  tokio::spawn(async move {
    refresh_server_cache(komodo_timestamp()).await;
    let interval = monitoring_interval();
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
          "Failed to get server list (refresh server cache) | {e:#}"
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
-> &'static CloneCache<String, Arc<Mutex<i64>>> {
  static CACHE: OnceLock<CloneCache<String, Arc<Mutex<i64>>>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// The background loop will call this with force: false,
/// which exits early if the lock is busy or it was completed too recently.
/// If force is true, it will wait on simultaneous calls, and will
/// ignore the restriction on being completed too recently.
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

  let resources = UpdateCacheResources::load_server(server).await;

  // Handle server disabled
  if !server.config.enabled {
    resources.insert_status_unknown().await;
    insert_server_status(
      server,
      ServerState::Disabled,
      None,
      None,
      None,
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
        None,
        Serror::from(&e),
      )
      .await;
      return;
    }
  };

  if let Some(docker) = &mut docker {
    docker.containers.iter_mut().for_each(|container| {
      container.server_id = Some(server.id.clone())
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
    resources::update_deployment_cache(
      server.name.clone(),
      resources.deployments,
      containers,
      images,
      &resources.builds,
    ),
    resources::update_stack_cache(
      server.name.clone(),
      resources.stacks,
      containers,
      images
    ),
  );

  insert_server_status(
    server,
    ServerState::Ok,
    Some(periphery_info),
    Some(system_info),
    system_stats.map(|stats| filter_volumes(server, stats)),
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

struct UpdateCacheResources {
  stacks: Vec<Stack>,
  deployments: Vec<Deployment>,
  builds: Vec<Build>,
  repos: Vec<Repo>,
}

impl UpdateCacheResources {
  pub async fn load_swarm(swarm: &Swarm) -> Self {
    let (stacks, deployments, builds) = tokio::join!(
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
      find_collect(&db_client().builds, doc! {}, None,),
    );

    let stacks = stacks.inspect_err(|e|  error!("Failed to get stacks list from db (update swarm status cache) | swarm: {} | {e:#}", swarm.name)).unwrap_or_default();
    let deployments =  deployments.inspect_err(|e| error!("Failed to get deployments list from db (update swarm status cache) | swarm : {} | {e:#}", swarm.name)).unwrap_or_default();
    let builds =  builds.inspect_err(|e| error!("Failed to get builds list from db (update swarm status cache) | swarm : {} | {e:#}", swarm.name)).unwrap_or_default();

    Self {
      stacks,
      deployments,
      builds,
      repos: Default::default(),
    }
  }

  pub async fn load_server(server: &Server) -> Self {
    let (stacks, deployments, builds, repos) = tokio::join!(
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
      find_collect(&db_client().builds, doc! {}, None,),
      find_collect(
        &db_client().repos,
        doc! { "config.server_id": &server.id },
        None,
      ),
    );

    let stacks = stacks.inspect_err(|e|  error!("Failed to get stacks list from db (update server status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();
    let deployments =  deployments.inspect_err(|e| error!("Failed to get deployments list from db (update server status cache) | server : {} | {e:#}", server.name)).unwrap_or_default();
    let builds =  builds.inspect_err(|e| error!("Failed to get builds list from db (update server status cache) | server : {} | {e:#}", server.name)).unwrap_or_default();
    let repos = repos.inspect_err(|e|  error!("Failed to get repos list from db (update server status cache) | server: {} | {e:#}", server.name)).unwrap_or_default();

    Self {
      stacks,
      deployments,
      builds,
      repos,
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
