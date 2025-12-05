use std::{
  sync::{Arc, OnceLock},
  time::Duration,
};

use anyhow::anyhow;
use async_timing_util::wait_until_timelength;
use cache::CloneCache;
use database::mungos::find::find_collect;
use formatting::format_serror;
use futures_util::future::join_all;
use komodo_client::entities::{
  docker::node::NodeState,
  komodo_timestamp,
  swarm::{Swarm, SwarmState},
};
use periphery_client::api::swarm::{
  PollSwarmStatus, PollSwarmStatusResponse,
};
use tokio::sync::Mutex;

use crate::{
  config::monitoring_interval,
  helpers::swarm::swarm_request_custom_timeout,
  monitor::UpdateCacheResources,
  state::{CachedSwarmStatus, db_client, swarm_status_cache},
};

const ADDITIONAL_MS: u128 = 1000;

pub fn spawn_swarm_monitoring_loop() {
  tokio::spawn(async move {
    refresh_swarm_cache(komodo_timestamp()).await;
    let interval = monitoring_interval();
    loop {
      let ts = (wait_until_timelength(interval, ADDITIONAL_MS).await
        - ADDITIONAL_MS) as i64;
      refresh_swarm_cache(ts).await;
    }
  });
}

async fn refresh_swarm_cache(_ts: i64) {
  let swarms =
    match find_collect(&db_client().swarms, None, None).await {
      Ok(swarms) => swarms,
      Err(e) => {
        error!(
          "Failed to get swarm list (refresh swarm cache) | {e:#}"
        );
        return;
      }
    };
  let futures = swarms.into_iter().map(|swarm| async move {
    update_cache_for_swarm(&swarm, false).await;
  });
  join_all(futures).await;
  // tokio::join!(check_alerts(ts), record_swarm_stats(ts));
}

/// Makes sure cache for swarm doesn't update too frequently / simultaneously.
/// If forced, will still block against simultaneous update.
fn update_cache_for_swarm_controller()
-> &'static CloneCache<String, Arc<Mutex<i64>>> {
  static CACHE: OnceLock<CloneCache<String, Arc<Mutex<i64>>>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// The background loop will call this with force: false,
/// which exits early if the lock is busy or it was completed too recently.
/// If force is true, it will wait on simultaneous calls, and will
/// ignore the restriction on being completed too recently.
pub async fn update_cache_for_swarm(swarm: &Swarm, force: bool) {
  // Concurrency controller to ensure it isn't done too often
  // when it happens in other contexts.
  let controller = update_cache_for_swarm_controller()
    .get_or_insert_default(&swarm.id)
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

  let resources = UpdateCacheResources::load_swarm(swarm).await;

  if swarm.config.server_ids.is_empty() {
    resources.insert_status_unknown().await;
    swarm_status_cache()
      .insert(
        swarm.id.clone(),
        CachedSwarmStatus {
          state: SwarmState::Unknown,
          inspect: None,
          lists: None,
          err: Some(format_serror(
            &anyhow!("No Servers configured as manager nodes").into(),
          )),
        }
        .into(),
      )
      .await;
    return;
  }

  let PollSwarmStatusResponse { inspect, lists } =
    match swarm_request_custom_timeout(
      &swarm.config.server_ids,
      PollSwarmStatus {},
      Duration::from_secs(1),
    )
    .await
    {
      Ok(info) => info,
      Err(e) => {
        resources.insert_status_unknown().await;
        swarm_status_cache()
          .insert(
            swarm.id.clone(),
            CachedSwarmStatus {
              state: SwarmState::Unknown,
              inspect: None,
              lists: None,
              err: Some(format_serror(&e.into())),
            }
            .into(),
          )
          .await;
        return;
      }
    };

  let mut state = SwarmState::Healthy;

  for node in &lists.nodes {
    if !matches!(node.state, Some(NodeState::READY)) {
      state = SwarmState::Unhealthy;
    }
  }

  // TODO: UPDATE STACKS / DEPLOYMENT CACHES

  swarm_status_cache()
    .insert(
      swarm.id.clone(),
      CachedSwarmStatus {
        state,
        inspect,
        lists: Some(lists),
        err: None,
      }
      .into(),
    )
    .await;
}
