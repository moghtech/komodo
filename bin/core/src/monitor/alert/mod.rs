use std::{collections::HashMap, sync::Mutex};

use anyhow::Context;
use komodo_client::entities::{
  alert::AlertDataVariant, permission::PermissionLevel, resource::ResourceQuery, server::Server, swarm::Swarm, user::system_user
};

use crate::resource;

mod deployment;
mod server;
mod stack;
mod swarm;

// called after cache update
pub async fn check_alerts(ts: i64) {
  let (swarm, server) =
    tokio::join!(get_all_swarms_map(), get_all_servers_map(),);

  let (swarms, swarm_names) =
    swarm.inspect_err(|e| error!("{e:#}")).unwrap_or_default();
  let (servers, server_names) =
    server.inspect_err(|e| error!("{e:#}")).unwrap_or_default();

  tokio::join!(
    swarm::alert_swarms(ts, swarms),
    server::alert_servers(ts, servers),
    deployment::alert_deployments(ts, &swarm_names, &server_names),
    stack::alert_stacks(ts, &swarm_names, &server_names)
  );
}

async fn get_all_swarms_map()
-> anyhow::Result<(HashMap<String, Swarm>, HashMap<String, String>)> {
  let swarms = resource::list_full_for_user::<Swarm>(
    ResourceQuery::default(),
    system_user(),
    PermissionLevel::Read.into(),
    &[],
  )
  .await
  .context("failed to get swarms from db (in alert_swarms)")?;

  let swarms = swarms
    .into_iter()
    .map(|swarm| (swarm.id.clone(), swarm))
    .collect::<HashMap<_, _>>();

  let swarm_names = swarms
    .iter()
    .map(|(id, swarm)| (id.clone(), swarm.name.clone()))
    .collect::<HashMap<_, _>>();

  Ok((swarms, swarm_names))
}

async fn get_all_servers_map()
-> anyhow::Result<(HashMap<String, Server>, HashMap<String, String>)>
{
  let servers = resource::list_full_for_user::<Server>(
    ResourceQuery::default(),
    system_user(),
    PermissionLevel::Read.into(),
    &[],
  )
  .await
  .context("failed to get servers from db (in alert_servers)")?;

  let servers = servers
    .into_iter()
    .map(|server| (server.id.clone(), server))
    .collect::<HashMap<_, _>>();

  let server_names = servers
    .iter()
    .map(|(id, server)| (id.clone(), server.name.clone()))
    .collect::<HashMap<_, _>>();

  Ok((servers, server_names))
}

/// Alert buffer to prevent immediate alerts on transient issues
struct AlertBuffer {
  buffer: Mutex<HashMap<(String, AlertDataVariant), bool>>,
}

impl AlertBuffer {
  fn new() -> Self {
    Self {
      buffer: Mutex::new(HashMap::new()),
    }
  }

  /// Check if alert should be opened. Requires two consecutive calls to return true.
  fn ready_to_open(
    &self,
    server_id: String,
    variant: AlertDataVariant,
  ) -> bool {
    let mut lock = self.buffer.lock().unwrap();
    let ready = lock.entry((server_id, variant)).or_default();
    if *ready {
      *ready = false;
      true
    } else {
      *ready = true;
      false
    }
  }

  /// Reset buffer state for a specific server/alert combination
  fn reset(&self, server_id: String, variant: AlertDataVariant) {
    let mut lock = self.buffer.lock().unwrap();
    lock.remove(&(server_id, variant));
  }
}
