use anyhow::Context;
use database::mungos::mongodb::{Collection, bson::doc};
use indexmap::IndexSet;
use komodo_client::entities::{
  Operation, ResourceTarget, ResourceTargetVariant, komodo_timestamp,
  permission::SpecificPermission,
  resource::Resource,
  server::{
    PartialServerConfig, Server, ServerConfig, ServerConfigDiff,
    ServerListItem, ServerListItemInfo, ServerQuerySpecifics,
  },
  update::Update,
  user::User,
};

use crate::{
  config::core_config,
  helpers::query::get_system_info,
  monitor::update_cache_for_server,
  state::{action_states, db_client, server_status_cache},
};

impl super::KomodoResource for Server {
  type Config = ServerConfig;
  type PartialConfig = PartialServerConfig;
  type ConfigDiff = ServerConfigDiff;
  type Info = ();
  type ListItem = ServerListItem;
  type QuerySpecifics = ServerQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Server
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Server(id.into())
  }

  fn creator_specific_permissions() -> IndexSet<SpecificPermission> {
    [
      SpecificPermission::Terminal,
      SpecificPermission::Inspect,
      SpecificPermission::Attach,
      SpecificPermission::Logs,
      SpecificPermission::Processes,
    ]
    .into_iter()
    .collect()
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().servers
  }

  async fn to_list_item(
    server: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = server_status_cache().get(&server.id).await;
    let (terminals_disabled, container_exec_disabled) =
      get_system_info(&server)
        .await
        .map(|i| (i.terminals_disabled, i.container_exec_disabled))
        .unwrap_or((true, true));
    ServerListItem {
      name: server.name,
      id: server.id,
      template: server.template,
      tags: server.tags,
      resource_type: ResourceTargetVariant::Server,
      info: ServerListItemInfo {
        state: status.as_ref().map(|s| s.state).unwrap_or_default(),
        version: status
          .map(|s| s.version.clone())
          .unwrap_or(String::from("Unknown")),
        region: server.config.region,
        address: server.config.address,
        external_address: server.config.external_address,
        send_unreachable_alerts: server
          .config
          .send_unreachable_alerts,
        send_cpu_alerts: server.config.send_cpu_alerts,
        send_mem_alerts: server.config.send_mem_alerts,
        send_disk_alerts: server.config.send_disk_alerts,
        send_version_mismatch_alerts: server
          .config
          .send_version_mismatch_alerts,
        terminals_disabled,
        container_exec_disabled,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .server
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateServer
  }

  fn user_can_create(user: &User) -> bool {
    user.admin
      || (!core_config().disable_non_admin_create
        && user.create_server_permissions)
  }

  async fn validate_create_config(
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    update_cache_for_server(created, true).await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateServer
  }

  async fn validate_update_config(
    _id: &str,
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_update(
    updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    update_cache_for_server(updated, true).await;
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameServer
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteServer
  }

  async fn pre_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    let db = db_client();

    let id = &resource.id;

    db.builders
      .update_many(
        doc! { "config.params.server_id": &id },
        doc! { "$set": { "config.params.server_id": "" } },
      )
      .await
      .context("failed to detach server from builders")?;

    db.deployments
      .update_many(
        doc! { "config.server_id": &id },
        doc! { "$set": { "config.server_id": "" } },
      )
      .await
      .context("failed to detach server from deployments")?;

    db.stacks
      .update_many(
        doc! { "config.server_id": &id },
        doc! { "$set": { "config.server_id": "" } },
      )
      .await
      .context("failed to detach server from stacks")?;

    db.repos
      .update_many(
        doc! { "config.server_id": &id },
        doc! { "$set": { "config.server_id": "" } },
      )
      .await
      .context("failed to detach server from repos")?;

    db.alerts
      .update_many(
        doc! { "target.type": "Server", "target.id": &id },
        doc! { "$set": {
          "resolved": true,
          "resolved_ts": komodo_timestamp()
        } },
      )
      .await
      .context("failed to close deleted server alerts")?;

    Ok(())
  }

  async fn post_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    server_status_cache().remove(&resource.id).await;
    Ok(())
  }
}
