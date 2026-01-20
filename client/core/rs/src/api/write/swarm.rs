use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  swarm::{_PartialSwarmConfig, Swarm},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateSwarm",
  description = "Create a Swarm.",
  request_body(content = CreateSwarm),
  responses(
    (status = 200, description = "The new swarm", body = crate::entities::swarm::SwarmSchema),
  ),
)]
pub fn create_swarm() {}

/// Create a Swarm. Response: [Swarm].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(mogh_error::Error)]
pub struct CreateSwarm {
  /// The name given to newly created swarm.
  pub name: String,
  /// Optional partial config to initialize the swarm with.
  #[serde(default)]
  pub config: _PartialSwarmConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopySwarm",
  description = "Copy a Swarm.",
  request_body(content = CopySwarm),
  responses(
    (status = 200, description = "The new swarm", body = crate::entities::swarm::SwarmSchema),
  ),
)]
pub fn copy_swarm() {}

/// Creates a new Swarm with given `name` and the configuration
/// of the Swarm at the given `id`. Response: [Swarm].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(mogh_error::Error)]
pub struct CopySwarm {
  /// The name of the new swarm.
  pub name: String,
  /// The id of the swarm to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteSwarm",
  description = "Delete a Swarm.",
  request_body(content = DeleteSwarm),
  responses(
    (status = 200, description = "The deleted swarm", body = crate::entities::swarm::SwarmSchema),
  ),
)]
pub fn delete_swarm() {}

/// Deletes the Swarm at the given id, and returns the deleted Swarm.
/// Response: [Swarm]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(mogh_error::Error)]
pub struct DeleteSwarm {
  /// The id or name of the swarm to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateSwarm",
  description = "Update a Swarm.",
  request_body(content = UpdateSwarm),
  responses(
    (status = 200, description = "The updated swarm", body = crate::entities::swarm::SwarmSchema),
  ),
)]
pub fn update_swarm() {}

/// Update the Swarm at the given id, and return the updated Swarm.
/// Response: [Swarm].
///
/// Note. If the attached server for the Swarm changes,
/// the Swarm will be deleted / cleaned up on the old server.
///
/// Note. This method updates only the fields which are set in the [_PartialSwarmConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(mogh_error::Error)]
pub struct UpdateSwarm {
  /// The id of the swarm to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialSwarmConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameSwarm",
  description = "Rename a Swarm.",
  request_body(content = RenameSwarm),
  responses(
    (status = 200, description = "The update", body = crate::entities::update::Update),
  ),
)]
pub fn rename_swarm() {}

/// Rename the Swarm at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameSwarm {
  /// The id or name of the Swarm to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
