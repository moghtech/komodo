use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  build::{_PartialBuildConfig, Build},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateBuild",
  description = "Create a build.",
  request_body(content = CreateBuild),
  responses(
    (status = 200, description = "The new build", body = crate::entities::build::BuildSchema),
  ),
)]
pub fn create_build() {}

/// Create a build. Response: [Build].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(mogh_error::Error)]
pub struct CreateBuild {
  /// The name given to newly created build.
  pub name: String,
  /// Optional partial config to initialize the build with.
  #[serde(default)]
  pub config: _PartialBuildConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyBuild",
  description = "Copy a build.",
  request_body(content = CopyBuild),
  responses(
    (status = 200, description = "The new build", body = crate::entities::build::BuildSchema),
  ),
)]
pub fn copy_build() {}

/// Creates a new build with given `name` and the configuration
/// of the build at the given `id`. Response: [Build].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(mogh_error::Error)]
pub struct CopyBuild {
  /// The name of the new build.
  pub name: String,
  /// The id of the build to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteBuild",
  description = "Delete a build.",
  request_body(content = DeleteBuild),
  responses(
    (status = 200, description = "The deleted build", body = crate::entities::build::BuildSchema),
  ),
)]
pub fn delete_build() {}

/// Deletes the build at the given id, and returns the deleted build.
/// Response: [Build]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(mogh_error::Error)]
pub struct DeleteBuild {
  /// The id or name of the build to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateBuild",
  description = "Update a build.",
  request_body(content = UpdateBuild),
  responses(
    (status = 200, description = "The updated build", body = crate::entities::build::BuildSchema),
  ),
)]
pub fn update_build() {}

/// Update the build at the given id, and return the updated build.
/// Response: [Build].
///
/// Note. This method updates only the fields which are set in the [_PartialBuildConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(mogh_error::Error)]
pub struct UpdateBuild {
  /// The id or name of the build to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialBuildConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameBuild",
  description = "Rename a build.",
  request_body(content = RenameBuild),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_build() {}

/// Rename the Build at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameBuild {
  /// The id or name of the Build to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/WriteBuildFileContents",
  description = "Update dockerfile contents in Files on Server or Git Repo mode.",
  request_body(content = WriteBuildFileContents),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn write_build_file_contents() {}

/// Update dockerfile contents in Files on Server or Git Repo mode. Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct WriteBuildFileContents {
  /// The name or id of the target Build.
  #[serde(alias = "id", alias = "name")]
  pub build: String,
  /// The dockerfile contents to write.
  pub contents: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RefreshBuildCache",
  description = "Trigger a refresh of the cached latest hash and message.",
  request_body(content = RefreshBuildCache),
  responses(
    (status = 200, description = "Build cache refreshed.", body = NoData),
  ),
)]
pub fn refresh_build_cache() {}

/// Trigger a refresh of the cached latest hash and message.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct RefreshBuildCache {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}
