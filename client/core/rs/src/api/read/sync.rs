use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::sync::{
  ResourceSync, ResourceSyncActionState, ResourceSyncListItem,
  ResourceSyncQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetResourceSync",
  description = "Get a specific sync.",
  request_body(content = GetResourceSync),
  responses(
    (status = 200, description = "The resource sync", body = crate::entities::sync::ResourceSyncSchema),
  ),
)]
pub fn get_resource_sync() {}

/// Get a specific sync. Response: [ResourceSync].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ResourceSync)]
#[error(mogh_error::Error)]
pub struct GetResourceSync {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

#[typeshare]
pub type GetResourceSyncResponse = ResourceSync;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListResourceSyncs",
  description = "List syncs matching optional query.",
  request_body(content = ListResourceSyncs),
  responses(
    (status = 200, description = "The list of resource syncs", body = ListResourceSyncsResponse),
  ),
)]
pub fn list_resource_syncs() {}

/// List syncs matching optional query. Response: [ListResourceSyncsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListResourceSyncsResponse)]
#[error(mogh_error::Error)]
pub struct ListResourceSyncs {
  /// optional structured query to filter syncs.
  #[serde(default)]
  pub query: ResourceSyncQuery,
}

#[typeshare]
pub type ListResourceSyncsResponse = Vec<ResourceSyncListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullResourceSyncs",
  description = "List syncs matching optional query.",
  request_body(content = ListFullResourceSyncs),
  responses(
    (status = 200, description = "The list of resource syncs", body = ListFullResourceSyncsResponse),
  ),
)]
pub fn list_full_resource_syncs() {}

/// List syncs matching optional query. Response: [ListFullResourceSyncsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullResourceSyncsResponse)]
#[error(mogh_error::Error)]
pub struct ListFullResourceSyncs {
  /// optional structured query to filter syncs.
  #[serde(default)]
  pub query: ResourceSyncQuery,
}

#[typeshare]
pub type ListFullResourceSyncsResponse = Vec<ResourceSync>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetResourceSyncActionState",
  description = "Get current action state for the sync.",
  request_body(content = GetResourceSyncActionState),
  responses(
    (status = 200, description = "The resource sync action state", body = GetResourceSyncActionStateResponse),
  ),
)]
pub fn get_resource_sync_action_state() {}

/// Get current action state for the sync. Response: [ResourceSyncActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetResourceSyncActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetResourceSyncActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub sync: String,
}

#[typeshare]
pub type GetResourceSyncActionStateResponse = ResourceSyncActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetResourceSyncsSummary",
  description = "Gets a summary of data relating to all syncs.",
  request_body(content = GetResourceSyncsSummary),
  responses(
    (status = 200, description = "The resource syncs summary", body = GetResourceSyncsSummaryResponse),
  ),
)]
pub fn get_resource_syncs_summary() {}

/// Gets a summary of data relating to all syncs.
/// Response: [GetResourceSyncsSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetResourceSyncsSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetResourceSyncsSummary {}

/// Response for [GetResourceSyncsSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
