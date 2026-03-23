use anyhow::Context;
use database::mungos::mongodb::Collection;
use komodo_client::entities::{
  Operation, ResourceTarget, ResourceTargetVariant,
  permission::PermissionLevel,
  resource::Resource,
  server::Server,
  swarm::{
    PartialSwarmConfig, Swarm, SwarmConfig, SwarmConfigDiff,
    SwarmInfo, SwarmListItem, SwarmListItemInfo, SwarmQuerySpecifics,
  },
  update::Update,
  user::User,
};

use crate::{
  config::core_config,
  monitor::refresh_swarm_cache,
  state::{db_client, swarm_status_cache},
};

use super::get_check_permissions;

impl super::KomodoResource for Swarm {
  type Config = SwarmConfig;
  type PartialConfig = PartialSwarmConfig;
  type ConfigDiff = SwarmConfigDiff;
  type Info = SwarmInfo;
  type ListItem = SwarmListItem;
  type QuerySpecifics = SwarmQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Swarm
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Swarm(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().swarms
  }

  async fn to_list_item(
    swarm: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (state, err) = swarm_status_cache()
      .get(&swarm.id)
      .await
      .map(|status| (status.state, status.err.clone()))
      .unwrap_or_default();
    SwarmListItem {
      name: swarm.name,
      id: swarm.id,
      template: swarm.template,
      tags: swarm.tags,
      resource_type: ResourceTargetVariant::Swarm,
      info: SwarmListItemInfo {
        server_ids: swarm.config.server_ids,
        state,
        err,
      },
    }
  }

  async fn busy(_id: &String) -> anyhow::Result<bool> {
    Ok(false)
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateSwarm
  }

  fn user_can_create(user: &User) -> bool {
    user.admin || !core_config().disable_non_admin_create
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_create(
    created: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    refresh_swarm_cache(created, true).await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateSwarm
  }

  async fn validate_update_config(
    _id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_update(
    updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    refresh_swarm_cache(updated, true).await;
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameSwarm
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteSwarm
  }

  async fn pre_delete(
    _swarm: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_delete(
    swarm: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    swarm_status_cache().remove(&swarm.id).await;
    Ok(())
  }
}

#[instrument("ValidateSwarmConfig", skip_all)]
async fn validate_config(
  config: &mut PartialSwarmConfig,
  user: &User,
) -> anyhow::Result<()> {
  if let Some(server_ids) = &mut config.server_ids {
    let mut res = Vec::with_capacity(server_ids.capacity());
    for server_id in server_ids.iter_mut() {
      let server = get_check_permissions::<Server>(
        server_id,
        user,
        PermissionLevel::Read.attach(),
      )
      .await
      .with_context(|| {
        format!("Cannot attach Server {server_id} to this Swarm")
      })?;
      res.push(server.id);
    }
    *server_ids = res;
  }
  Ok(())
}

// pub fn spawn_swarm_state_refresh_loop() {
//   tokio::spawn(async move {
//     loop {
//       refresh_swarm_state_cache().await;
//       tokio::time::sleep(Duration::from_secs(60)).await;
//     }
//   });
// }

// pub async fn refresh_swarm_state_cache() {
//   let _ = async {
//     let swarms = find_collect(&db_client().swarms, None, None)
//       .await
//       .context("failed to get swarms from db")?;
//     let cache = swarm_state_cache();
//     for swarm in swarms {
//       let state = get_swarm_state_from_db(&swarm.id).await;
//       cache.insert(swarm.id, state).await;
//     }
//     anyhow::Ok(())
//   }
//   .await
//   .inspect_err(|e| {
//     warn!("failed to refresh swarm state cache | {e:#}")
//   });
// }
