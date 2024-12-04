use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::sync::{
  ResourceSync, ResourceSyncActionState, ResourceSyncListItem,
  ResourceSyncQuery,
};

use super::KomodoReadRequest;

//

/// Get a specific sync. Response: [ResourceSync].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ResourceSync)]
#[error(serror::Error)]
pub struct GetResourceSync {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

#[typeshare]
pub type GetResourceSyncResponse = ResourceSync;

//

/// List syncs matching optional query. Response: [ListResourceSyncsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListResourceSyncsResponse)]
#[error(serror::Error)]
pub struct ListResourceSyncs {
  /// optional structured query to filter syncs.
  #[serde(default)]
  pub query: ResourceSyncQuery,
}

#[typeshare]
pub type ListResourceSyncsResponse = Vec<ResourceSyncListItem>;

//

/// List syncs matching optional query. Response: [ListFullResourceSyncsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullResourceSyncsResponse)]
#[error(serror::Error)]
pub struct ListFullResourceSyncs {
  /// optional structured query to filter syncs.
  #[serde(default)]
  pub query: ResourceSyncQuery,
}

#[typeshare]
pub type ListFullResourceSyncsResponse = Vec<ResourceSync>;

//

/// Get current action state for the sync. Response: [ResourceSyncActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetResourceSyncActionStateResponse)]
#[error(serror::Error)]
pub struct GetResourceSyncActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

#[typeshare]
pub type GetResourceSyncActionStateResponse = ResourceSyncActionState;

//

/// Gets a summary of data relating to all syncs.
/// Response: [GetResourceSyncsSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetResourceSyncsSummaryResponse)]
#[error(serror::Error)]
pub struct GetResourceSyncsSummary {}

/// Response for [GetResourceSyncsSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetResourceSyncsSummaryResponse {
  /// The total number of syncs
  pub total: u32,
  /// The number of syncs with Ok state.
  pub ok: u32,
  /// The number of syncs currently syncing.
  pub syncing: u32,
  /// The number of syncs with pending updates
  pub pending: u32,
  /// The number of syncs with failed state.
  pub failed: u32,
  /// The number of syncs with unknown state.
  pub unknown: u32,
}

//

/// Get a target Sync's configured webhooks. Response: [GetSyncWebhooksEnabledResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetSyncWebhooksEnabledResponse)]
#[error(serror::Error)]
pub struct GetSyncWebhooksEnabled {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

/// Response for [GetSyncWebhooksEnabled]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetSyncWebhooksEnabledResponse {
  /// Whether the repo webhooks can even be managed.
  /// The repo owner must be in `github_webhook_app.owners` list to be managed.
  pub managed: bool,
  /// Whether pushes to branch trigger refresh. Will always be false if managed is false.
  pub refresh_enabled: bool,
  /// Whether pushes to branch trigger sync execution. Will always be false if managed is false.
  pub sync_enabled: bool,
}
