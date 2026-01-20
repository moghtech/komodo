use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  builder::{Builder, PartialBuilderConfig},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateBuilder",
  description = "Create a builder.",
  request_body(content = CreateBuilder),
  responses(
    (status = 200, description = "The created builder", body = crate::entities::builder::BuilderSchema),
  ),
)]
pub fn create_builder() {}

/// Create a builder. Response: [Builder].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Builder)]
#[error(mogh_error::Error)]
pub struct CreateBuilder {
  /// The name given to newly created builder.
  pub name: String,
  /// Optional partial config to initialize the builder with.
  #[serde(default)]
  pub config: PartialBuilderConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyBuilder",
  description = "Copy a builder.",
  request_body(content = CopyBuilder),
  responses(
    (status = 200, description = "The new builder", body = crate::entities::builder::BuilderSchema),
  ),
)]
pub fn copy_builder() {}

/// Creates a new builder with given `name` and the configuration
/// of the builder at the given `id`. Response: [Builder]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Builder)]
#[error(mogh_error::Error)]
pub struct CopyBuilder {
  /// The name of the new builder.
  pub name: String,
  /// The id of the builder to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteBuilder",
  description = "Delete a builder.",
  request_body(content = DeleteBuilder),
  responses(
    (status = 200, description = "The deleted builder", body = crate::entities::builder::BuilderSchema),
  ),
)]
pub fn delete_builder() {}

/// Deletes the builder at the given id, and returns the deleted builder.
/// Response: [Builder]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Builder)]
#[error(mogh_error::Error)]
pub struct DeleteBuilder {
  /// The id or name of the builder to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateBuilder",
  description = "Update a builder.",
  request_body(content = UpdateBuilder),
  responses(
    (status = 200, description = "The updated builder", body = crate::entities::builder::BuilderSchema),
  ),
)]
pub fn update_builder() {}

/// Update the builder at the given id, and return the updated builder.
/// Response: [Builder].
///
/// Note. This method updates only the fields which are set in the [PartialBuilderConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Builder)]
#[error(mogh_error::Error)]
pub struct UpdateBuilder {
  /// The id of the builder to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: PartialBuilderConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameBuilder",
  description = "Rename a builder.",
  request_body(content = RenameBuilder),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_builder() {}

/// Rename the Builder at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameBuilder {
  /// The id or name of the Builder to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
