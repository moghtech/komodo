use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::provider::*;

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateGitProviderAccount",
  description = "**Admin only.** Create a git provider account.",
  request_body(content = CreateGitProviderAccount),
  responses(
    (status = 200, description = "The created account", body = CreateGitProviderAccountResponse),
  ),
)]
pub fn create_git_provider_account() {}

/// **Admin only.** Create a git provider account.
/// Response: [GitProviderAccount].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateGitProviderAccountResponse)]
#[error(mogh_error::Error)]
pub struct CreateGitProviderAccount {
  /// The initial account config. Anything in the _id field will be ignored,
  /// as this is generated on creation.
  pub account: _PartialGitProviderAccount,
}

#[typeshare]
pub type CreateGitProviderAccountResponse = GitProviderAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateGitProviderAccount",
  description = "**Admin only.** Update a git provider account.",
  request_body(content = UpdateGitProviderAccount),
  responses(
    (status = 200, description = "The updated account", body = UpdateGitProviderAccountResponse),
  ),
)]
pub fn update_git_provider_account() {}

/// **Admin only.** Update a git provider account.
/// Response: [GitProviderAccount].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateGitProviderAccountResponse)]
#[error(mogh_error::Error)]
pub struct UpdateGitProviderAccount {
  /// The id of the git provider account to update.
  pub id: String,
  /// The partial git provider account.
  pub account: _PartialGitProviderAccount,
}

#[typeshare]
pub type UpdateGitProviderAccountResponse = GitProviderAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteGitProviderAccount",
  description = "**Admin only.** Delete a git provider account.",
  request_body(content = DeleteGitProviderAccount),
  responses(
    (status = 200, description = "The deleted account", body = DeleteGitProviderAccountResponse),
  ),
)]
pub fn delete_git_provider_account() {}

/// **Admin only.** Delete a git provider account.
/// Response: [DeleteGitProviderAccountResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteGitProviderAccountResponse)]
#[error(mogh_error::Error)]
pub struct DeleteGitProviderAccount {
  /// The id of the git provider to delete
  pub id: String,
}

#[typeshare]
pub type DeleteGitProviderAccountResponse = GitProviderAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateDockerRegistryAccount",
  description = "**Admin only.** Create a docker registry account.",
  request_body(content = CreateDockerRegistryAccount),
  responses(
    (status = 200, description = "The created account", body = CreateDockerRegistryAccountResponse),
  ),
)]
pub fn create_docker_registry_account() {}

/// **Admin only.** Create a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateDockerRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct CreateDockerRegistryAccount {
  pub account: _PartialDockerRegistryAccount,
}

#[typeshare]
pub type CreateDockerRegistryAccountResponse = DockerRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateDockerRegistryAccount",
  description = "**Admin only.** Update a docker registry account.",
  request_body(content = UpdateDockerRegistryAccount),
  responses(
    (status = 200, description = "The updated account", body = UpdateDockerRegistryAccountResponse),
  ),
)]
pub fn update_docker_registry_account() {}

/// **Admin only.** Update a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateDockerRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct UpdateDockerRegistryAccount {
  /// The id of the docker registry to update
  pub id: String,
  /// The partial docker registry account.
  pub account: _PartialDockerRegistryAccount,
}

#[typeshare]
pub type UpdateDockerRegistryAccountResponse = DockerRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteDockerRegistryAccount",
  description = "**Admin only.** Delete a docker registry account.",
  request_body(content = DeleteDockerRegistryAccount),
  responses(
    (status = 200, description = "The deleted account", body = DeleteDockerRegistryAccountResponse),
  ),
)]
pub fn delete_docker_registry_account() {}

/// **Admin only.** Delete a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteDockerRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct DeleteDockerRegistryAccount {
  /// The id of the docker registry account to delete
  pub id: String,
}

#[typeshare]
pub type DeleteDockerRegistryAccountResponse = DockerRegistryAccount;
