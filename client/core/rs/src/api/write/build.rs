use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  build::{Build, _PartialBuildConfig},
  update::Update,
  NoData,
};

use super::KomodoWriteRequest;

//

/// Create a build. Response: [Build].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(serror::Error)]
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
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(serror::Error)]
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
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(serror::Error)]
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
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Build)]
#[error(serror::Error)]
pub struct UpdateBuild {
  /// The id of the build to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialBuildConfig,
}

//

/// Rename the Build at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
#[error(serror::Error)]
pub struct RenameBuild {
  /// The id or name of the Build to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

/// Trigger a refresh of the cached latest hash and message.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct RefreshBuildCache {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

//

/// Create a webhook on the github repo attached to the build
/// passed in request. Response: [CreateBuildWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateBuildWebhookResponse)]
#[error(serror::Error)]
pub struct CreateBuildWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type CreateBuildWebhookResponse = NoData;

//

/// Delete a webhook on the github repo attached to the build
/// passed in request. Response: [CreateBuildWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteBuildWebhookResponse)]
#[error(serror::Error)]
pub struct DeleteBuildWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type DeleteBuildWebhookResponse = NoData;
