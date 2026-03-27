use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  repo::{_PartialRepoConfig, Repo},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateRepo",
  description = "Create a repo.",
  request_body(content = CreateRepo),
  responses(
    (status = 200, description = "The new repo", body = crate::entities::repo::RepoSchema),
  ),
)]
pub fn create_repo() {}

/// Create a repo. Response: [Repo].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(mogh_error::Error)]
pub struct CreateRepo {
  /// The name given to newly created repo.
  pub name: String,
  /// Optional partial config to initialize the repo with.
  #[serde(default)]
  pub config: _PartialRepoConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyRepo",
  description = "Copy a repo.",
  request_body(content = CopyRepo),
  responses(
    (status = 200, description = "The new repo", body = crate::entities::repo::RepoSchema),
  ),
)]
pub fn copy_repo() {}

/// Creates a new repo with given `name` and the configuration
/// of the repo at the given `id`. Response: [Repo].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(mogh_error::Error)]
pub struct CopyRepo {
  /// The name of the new repo.
  pub name: String,
  /// The id of the repo to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteRepo",
  description = "Delete a repo.",
  request_body(content = DeleteRepo),
  responses(
    (status = 200, description = "The deleted repo", body = crate::entities::repo::RepoSchema),
  ),
)]
pub fn delete_repo() {}

/// Deletes the repo at the given id, and returns the deleted repo.
/// Response: [Repo]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(mogh_error::Error)]
pub struct DeleteRepo {
  /// The id or name of the repo to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateRepo",
  description = "Update a repo.",
  request_body(content = UpdateRepo),
  responses(
    (status = 200, description = "The updated repo", body = crate::entities::repo::RepoSchema),
  ),
)]
pub fn update_repo() {}

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
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Repo)]
#[error(mogh_error::Error)]
pub struct UpdateRepo {
  /// The id of the repo to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialRepoConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameRepo",
  description = "Rename a repo.",
  request_body(content = RenameRepo),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_repo() {}

/// Rename the Repo at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameRepo {
  /// The id or name of the Repo to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RefreshRepoCache",
  description = "Trigger a refresh of the cached latest hash and message.",
  request_body(content = RefreshRepoCache),
  responses(
    (status = 200, description = "Repo cache refreshed.", body = NoData),
  ),
)]
pub fn refresh_repo_cache() {}

/// Trigger a refresh of the cached latest hash and message.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct RefreshRepoCache {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

//

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum RepoWebhookAction {
  Clone,
  Pull,
  Build,
}
