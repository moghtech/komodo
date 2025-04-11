use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  sync::{_PartialResourceSyncConfig, ResourceSync},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a sync. Response: [ResourceSync].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct CreateResourceSync {
  /// The name given to newly created sync.
  pub name: String,
  /// Optional partial config to initialize the sync with.
  #[serde(default)]
  pub config: _PartialResourceSyncConfig,
}

//

/// Creates a new sync with given `name` and the configuration
/// of the sync at the given `id`. Response: [ResourceSync].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct CopyResourceSync {
  /// The name of the new sync.
  pub name: String,
  /// The id of the sync to copy.
  pub id: String,
}

//

/// Deletes the sync at the given id, and returns the deleted sync.
/// Response: [ResourceSync]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct DeleteResourceSync {
  /// The id or name of the sync to delete.
  pub id: String,
}

//

/// Update the sync at the given id, and return the updated sync.
/// Response: [ResourceSync].
///
/// Note. This method updates only the fields which are set in the [_PartialResourceSyncConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct UpdateResourceSync {
  /// The id of the sync to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialResourceSyncConfig,
}

//

/// Rename the ResourceSync at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameResourceSync {
  /// The id or name of the ResourceSync to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

/// Trigger a refresh of the computed diff logs for view. Response: [ResourceSync]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct RefreshResourceSyncPending {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

//

/// Rename the stack at id to the given name. Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
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

/// Exports matching resources, and writes to the target sync's resource file. Response: [Update]
///
/// Note. Will fail if the Sync is not `managed`.
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct CommitSync {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

//

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncWebhookAction {
  Refresh,
  Sync,
}

/// Create a webhook on the github repo attached to the sync
/// passed in request. Response: [CreateSyncWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateSyncWebhookResponse)]
#[error(serror::Error)]
pub struct CreateSyncWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
  /// "Refresh" or "Sync"
  pub action: SyncWebhookAction,
}

#[typeshare]
pub type CreateSyncWebhookResponse = NoData;

//

/// Delete the webhook on the github repo attached to the sync
/// passed in request. Response: [DeleteSyncWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteSyncWebhookResponse)]
#[error(serror::Error)]
pub struct DeleteSyncWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
  /// "Refresh" or "Sync"
  pub action: SyncWebhookAction,
}

#[typeshare]
pub type DeleteSyncWebhookResponse = NoData;
