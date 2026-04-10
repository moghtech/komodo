use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  sync::{_PartialResourceSyncConfig, ResourceSync},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateResourceSync",
  description = "Create a resource sync.",
  request_body(content = CreateResourceSync),
  responses(
    (status = 200, description = "The created resource sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn create_resource_sync() {}

/// Create a sync. Response: [ResourceSync].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct CreateResourceSync {
  /// The name given to newly created sync.
  pub name: String,
  /// Optional partial config to initialize the sync with.
  #[serde(default)]
  pub config: _PartialResourceSyncConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyResourceSync",
  description = "Copy a resource sync.",
  request_body(content = CopyResourceSync),
  responses(
    (status = 200, description = "The copied resource sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn copy_resource_sync() {}

/// Creates a new sync with given `name` and the configuration
/// of the sync at the given `id`. Response: [ResourceSync].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct CopyResourceSync {
  /// The name of the new sync.
  pub name: String,
  /// The id of the sync to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteResourceSync",
  description = "Delete a resource sync.",
  request_body(content = DeleteResourceSync),
  responses(
    (status = 200, description = "The deleted resource sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn delete_resource_sync() {}

/// Deletes the sync at the given id, and returns the deleted sync.
/// Response: [ResourceSync]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct DeleteResourceSync {
  /// The id or name of the sync to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateResourceSync",
  description = "Update a resource sync.",
  request_body(content = UpdateResourceSync),
  responses(
    (status = 200, description = "The updated resource sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn update_resource_sync() {}

/// Update the sync at the given id, and return the updated sync.
/// Response: [ResourceSync].
///
/// Note. This method updates only the fields which are set in the [_PartialResourceSyncConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct UpdateResourceSync {
  /// The id of the sync to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialResourceSyncConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameResourceSync",
  description = "Rename a resource sync.",
  request_body(content = RenameResourceSync),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_resource_sync() {}

/// Rename the ResourceSync at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameResourceSync {
  /// The id or name of the ResourceSync to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RefreshResourceSyncPending",
  description = "Refresh resource sync pending state.",
  request_body(content = RefreshResourceSyncPending),
  responses(
    (status = 200, description = "The refreshed sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn refresh_resource_sync_pending() {}

/// Trigger a refresh of the computed diff logs for view. Response: [ResourceSync]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct RefreshResourceSyncPending {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/WriteSyncFileContents",
  description = "Write to the sync toml file contents.",
  request_body(content = WriteSyncFileContents),
  responses(
    (status = 200, description = "The update result", body = Update),
  ),
)]
pub fn write_sync_file_contents() {}

/// Write to the sync toml file contents. Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct WriteSyncFileContents {
  /// The name or id of the target Sync.
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
  /// If this file was under a resource folder, this will be the folder.
  /// Otherwise, it should be empty string.
  pub resource_path: String,
  /// The file path relative to the resource path.
  pub file_path: String,
  /// The contents to write.
  pub contents: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CommitSync",
  description = "Exports matching resources, and writes to the target sync's resource file.",
  request_body(content = CommitSync),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn commit_sync() {}

/// Exports matching resources, and writes to the target sync's resource file. Response: [Update]
///
/// Note. Will fail if the Sync is not `managed`.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CommitSync {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

//

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SyncWebhookAction {
  Refresh,
  Sync,
}
