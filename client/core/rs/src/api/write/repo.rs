use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  repo::{Repo, _PartialRepoConfig},
  update::Update,
  NoData,
};

use super::KomodoWriteRequest;

//

/// Create a repo. Response: [Repo].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(serror::Error)]
pub struct CreateRepo {
  /// The name given to newly created repo.
  pub name: String,
  /// Optional partial config to initialize the repo with.
  #[serde(default)]
  pub config: _PartialRepoConfig,
}

//

/// Creates a new repo with given `name` and the configuration
/// of the repo at the given `id`. Response: [Repo].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(serror::Error)]
pub struct CopyRepo {
  /// The name of the new repo.
  pub name: String,
  /// The id of the repo to copy.
  pub id: String,
}

//

/// Deletes the repo at the given id, and returns the deleted repo.
/// Response: [Repo]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(serror::Error)]
pub struct DeleteRepo {
  /// The id or name of the repo to delete.
  pub id: String,
}

//

/// Update the repo at the given id, and return the updated repo.
/// Response: [Repo].
///
/// Note. If the attached server for the repo changes,
/// the repo will be deleted / cleaned up on the old server.
///
/// Note. This method updates only the fields which are set in the [_PartialRepoConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(serror::Error)]
pub struct UpdateRepo {
  /// The id of the repo to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialRepoConfig,
}

//

/// Rename the Repo at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
#[error(serror::Error)]
pub struct RenameRepo {
  /// The id or name of the Repo to rename.
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
pub struct RefreshRepoCache {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

//

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepoWebhookAction {
  Clone,
  Pull,
  Build,
}

/// Create a webhook on the github repo attached to the (Komodo) Repo resource.
/// passed in request. Response: [CreateRepoWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateRepoWebhookResponse)]
#[error(serror::Error)]
pub struct CreateRepoWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
  /// "Clone" or "Pull" or "Build"
  pub action: RepoWebhookAction,
}

#[typeshare]
pub type CreateRepoWebhookResponse = NoData;

//

/// Delete the webhook on the github repo attached to the (Komodo) Repo resource.
/// passed in request. Response: [DeleteRepoWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteRepoWebhookResponse)]
#[error(serror::Error)]
pub struct DeleteRepoWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
  /// "Clone" or "Pull" or "Build"
  pub action: RepoWebhookAction,
}

#[typeshare]
pub type DeleteRepoWebhookResponse = NoData;
