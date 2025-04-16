use anyhow::Context;
use formatting::format_serror;
use komodo_client::{
  api::write::RefreshResourceSyncPending,
  entities::{
    komodo_timestamp, resource::Resource, sync::{
      PartialResourceSyncConfig, ResourceSync, ResourceSyncConfig,
      ResourceSyncConfigDiff, ResourceSyncInfo, ResourceSyncListItem,
      ResourceSyncListItemInfo, ResourceSyncQuerySpecifics,
      ResourceSyncState,
    }, update::Update, user::{sync_user, User}, Operation, ResourceTarget, ResourceTargetVariant
  },
};
use mongo_indexed::doc;
use mungos::mongodb::Collection;
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  state::{action_states, db_client},
};

impl super::KomodoResource for ResourceSync {
  type Config = ResourceSyncConfig;
  type PartialConfig = PartialResourceSyncConfig;
  type ConfigDiff = ResourceSyncConfigDiff;
  type Info = ResourceSyncInfo;
  type ListItem = ResourceSyncListItem;
  type QuerySpecifics = ResourceSyncQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::ResourceSync
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::ResourceSync(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().resource_syncs
  }

  async fn to_list_item(
    resource_sync: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state =
      get_resource_sync_state(&resource_sync.id, &resource_sync.info)
        .await;
    ResourceSyncListItem {
      id: resource_sync.id,
      name: resource_sync.name,
      tags: resource_sync.tags,
      resource_type: ResourceTargetVariant::ResourceSync,
      info: ResourceSyncListItemInfo {
        file_contents: !resource_sync.config.file_contents.is_empty(),
        files_on_host: resource_sync.config.files_on_host,
        managed: resource_sync.config.managed,
        git_provider: resource_sync.config.git_provider,
        repo: resource_sync.config.repo,
        branch: resource_sync.config.branch,
        last_sync_ts: resource_sync.info.last_sync_ts,
        last_sync_hash: resource_sync.info.last_sync_hash,
        last_sync_message: resource_sync.info.last_sync_message,
        resource_path: resource_sync.config.resource_path,
        state,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .resource_sync
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateResourceSync
  }

  fn user_can_create(user: &User) -> bool {
    user.admin
  }

  async fn validate_create_config(
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    if let Err(e) = (RefreshResourceSyncPending {
      sync: created.id.clone(),
    })
    .resolve(&WriteArgs {
      user: sync_user().to_owned(),
    })
    .await
    {
      update.push_error_log(
        "Refresh sync pending",
        format_serror(&e.error.context("The sync pending cache has failed to refresh. This is likely due to a misconfiguration of the sync").into())
      );
    };
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateResourceSync
  }

  async fn validate_update_config(
    _id: &str,
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_update(
    updated: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    Self::post_create(updated, update).await
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameResourceSync
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteResourceSync
  }

  async fn pre_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    db_client().alerts
      .update_many(
        doc! { "target.type": "ResourceSync", "target.id": &resource.id },
        doc! { "$set": {
          "resolved": true,
          "resolved_ts": komodo_timestamp()
        } },
      )
      .await
      .context("failed to close deleted sync alerts")?;

    Ok(())
  }

  async fn post_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }
}

async fn get_resource_sync_state(
  id: &String,
  data: &ResourceSyncInfo,
) -> ResourceSyncState {
  if let Some(state) = action_states()
    .resource_sync
    .get(id)
    .await
    .and_then(|s| {
      s.get()
        .map(|s| {
          if s.syncing {
            Some(ResourceSyncState::Syncing)
          } else {
            None
          }
        })
        .ok()
    })
    .flatten()
  {
    return state;
  }
  if data.pending_error.is_some() || !data.remote_errors.is_empty() {
    ResourceSyncState::Failed
  } else if !data.resource_updates.is_empty()
    || !data.variable_updates.is_empty()
    || !data.user_group_updates.is_empty()
    || data.pending_deploy.to_deploy > 0
  {
    ResourceSyncState::Pending
  } else {
    ResourceSyncState::Ok
  }
}
