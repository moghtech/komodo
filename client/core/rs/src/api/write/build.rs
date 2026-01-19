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
