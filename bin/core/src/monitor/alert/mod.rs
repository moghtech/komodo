use std::collections::HashMap;

use anyhow::Context;
use komodo_client::entities::{
  permission::PermissionLevel, resource::ResourceQuery,
  server::Server, user::User,
};

use crate::resource;

mod deployment;
mod server;
mod stack;

// called after cache update
#[instrument(level = "debug")]
pub async fn check_alerts(ts: i64) {
  let (servers, server_names) = match get_all_servers_map().await {
    Ok(res) => res,
    Err(e) => {
      error!("{e:#?}");
      return;
    }
  };

  tokio::join!(
    server::alert_servers(ts, servers),
    deployment::alert_deployments(ts, &server_names),
    stack::alert_stacks(ts, &server_names)
  );
}

#[instrument(level = "debug")]
async fn get_all_servers_map()
-> anyhow::Result<(HashMap<String, Server>, HashMap<String, String>)>
{
  let servers = resource::list_full_for_user::<Server>(
    ResourceQuery::default(),
    &User {
      admin: true,
      ..Default::default()
    },
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
