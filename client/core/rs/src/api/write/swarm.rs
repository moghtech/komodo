use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  swarm::{_PartialSwarmConfig, Swarm},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a Swarm. Response: [Swarm].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(serror::Error)]
pub struct CreateSwarm {
  /// The name given to newly created swarm.
  pub name: String,
  /// Optional partial config to initialize the swarm with.
  #[serde(default)]
  pub config: _PartialSwarmConfig,
}

//

/// Creates a new Swarm with given `name` and the configuration
/// of the Swarm at the given `id`. Response: [Swarm].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(serror::Error)]
pub struct CopySwarm {
  /// The name of the new swarm.
  pub name: String,
  /// The id of the swarm to copy.
  pub id: String,
}

//

/// Deletes the Swarm at the given id, and returns the deleted Swarm.
/// Response: [Swarm]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(serror::Error)]
pub struct DeleteSwarm {
  /// The id or name of the swarm to delete.
  pub id: String,
}

//

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
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Swarm)]
#[error(serror::Error)]
pub struct UpdateSwarm {
  /// The id of the swarm to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialSwarmConfig,
}

//

/// Rename the Swarm at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameSwarm {
  /// The id or name of the Swarm to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
