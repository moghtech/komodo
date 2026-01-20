use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  alerter::{_PartialAlerterConfig, Alerter},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateAlerter",
  description = "Create an alerter.",
  request_body(content = CreateAlerter),
  responses(
    (status = 200, description = "The new alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn create_alerter() {}

/// Create an alerter. Response: [Alerter].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(mogh_error::Error)]
pub struct CreateAlerter {
  /// The name given to newly created alerter.
  pub name: String,
  /// Optional partial config to initialize the alerter with.
  #[serde(default)]
  pub config: _PartialAlerterConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyAlerter",
  description = "Copy an alerter.",
  request_body(content = CopyAlerter),
  responses(
    (status = 200, description = "The new alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn copy_alerter() {}

/// Creates a new alerter with given `name` and the configuration
/// of the alerter at the given `id`. Response: [Alerter].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(mogh_error::Error)]
pub struct CopyAlerter {
  /// The name of the new alerter.
  pub name: String,
  /// The id of the alerter to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteAlerter",
  description = "Delete an alerter.",
  request_body(content = DeleteAlerter),
  responses(
    (status = 200, description = "The deleted alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn delete_alerter() {}

/// Deletes the alerter at the given id, and returns the deleted alerter.
/// Response: [Alerter]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(mogh_error::Error)]
pub struct DeleteAlerter {
  /// The id or name of the alerter to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateAlerter",
  description = "Update an alerter.",
  request_body(content = UpdateAlerter),
  responses(
    (status = 200, description = "The updated alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn update_alerter() {}

/// Update the alerter at the given id, and return the updated alerter. Response: [Alerter].
///
/// Note. This method updates only the fields which are set in the [PartialAlerterConfig][crate::entities::alerter::PartialAlerterConfig],
/// effectively merging diffs into the final document. This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Alerter)]
#[error(mogh_error::Error)]
pub struct UpdateAlerter {
  /// The id of the alerter to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialAlerterConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameAlerter",
  description = "Rename an alerter.",
  request_body(content = RenameAlerter),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_alerter() {}

/// Rename the Alerter at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameAlerter {
  /// The id or name of the Alerter to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
