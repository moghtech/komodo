use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::provider::{
  DockerRegistryAccount, GitProviderAccount,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetGitProviderAccount",
  description = "Get a specific git provider account.",
  request_body(content = GetGitProviderAccount),
  responses(
    (status = 200, description = "The git provider account", body = GetGitProviderAccountResponse),
  ),
)]
pub fn get_git_provider_account() {}

/// Get a specific git provider account.
/// Response: [GetGitProviderAccountResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetGitProviderAccountResponse)]
#[error(mogh_error::Error)]
pub struct GetGitProviderAccount {
  pub id: String,
}

#[typeshare]
pub type GetGitProviderAccountResponse = GitProviderAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListGitProviderAccounts",
  description = "List git provider accounts matching optional query.",
  request_body(content = ListGitProviderAccounts),
  responses(
    (status = 200, description = "The list of git provider accounts", body = ListGitProviderAccountsResponse),
  ),
)]
pub fn list_git_provider_accounts() {}

/// List git provider accounts matching optional query.
/// Response: [ListGitProviderAccountsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListGitProviderAccountsResponse)]
#[error(mogh_error::Error)]
pub struct ListGitProviderAccounts {
  /// Optionally filter by accounts with a specific domain.
  pub domain: Option<String>,
  /// Optionally filter by accounts with a specific username.
  pub username: Option<String>,
}

#[typeshare]
pub type ListGitProviderAccountsResponse = Vec<GitProviderAccount>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetDockerRegistryAccount",
  description = "Get a specific docker registry account.",
  request_body(content = GetDockerRegistryAccount),
  responses(
    (status = 200, description = "The docker registry account", body = GetDockerRegistryAccountResponse),
  ),
)]
pub fn get_docker_registry_account() {}

/// Get a specific docker registry account.
/// Response: [GetDockerRegistryAccountResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetDockerRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct GetDockerRegistryAccount {
  pub id: String,
}

#[typeshare]
pub type GetDockerRegistryAccountResponse = DockerRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListDockerRegistryAccounts",
  description = "List docker registry accounts matching optional query.",
  request_body(content = ListDockerRegistryAccounts),
  responses(
    (status = 200, description = "The list of docker registry accounts", body = ListDockerRegistryAccountsResponse),
  ),
)]
pub fn list_docker_registry_accounts() {}

/// List docker registry accounts matching optional query.
/// Response: [ListDockerRegistryAccountsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerRegistryAccountsResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerRegistryAccounts {
  /// Optionally filter by accounts with a specific domain.
  pub domain: Option<String>,
  /// Optionally filter by accounts with a specific username.
  pub username: Option<String>,
}

#[typeshare]
pub type ListDockerRegistryAccountsResponse =
  Vec<DockerRegistryAccount>;
