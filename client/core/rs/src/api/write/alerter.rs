use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  alerter::{_PartialAlerterConfig, Alerter},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create an alerter. Response: [Alerter].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(serror::Error)]
pub struct CreateAlerter {
  /// The name given to newly created alerter.
  pub name: String,
  /// Optional partial config to initialize the alerter with.
  #[serde(default)]
  pub config: _PartialAlerterConfig,
}

//

/// Creates a new alerter with given `name` and the configuration
/// of the alerter at the given `id`. Response: [Alerter].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(serror::Error)]
pub struct CopyAlerter {
  /// The name of the new alerter.
  pub name: String,
  /// The id of the alerter to copy.
  pub id: String,
}

//

/// Deletes the alerter at the given id, and returns the deleted alerter.
/// Response: [Alerter]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(serror::Error)]
pub struct DeleteAlerter {
  /// The id or name of the alerter to delete.
  pub id: String,
}

//

/// Update the alerter at the given id, and return the updated alerter. Response: [Alerter].
///
/// Note. This method updates only the fields which are set in the [PartialAlerterConfig][crate::entities::alerter::PartialAlerterConfig],
/// effectively merging diffs into the final document. This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(serror::Error)]
pub struct UpdateAlerter {
  /// The id of the alerter to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialAlerterConfig,
}

//

/// Rename the Alerter at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameAlerter {
  /// The id or name of the Alerter to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
