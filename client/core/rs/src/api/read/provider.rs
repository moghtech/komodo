use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::provider::{
  DockerRegistryAccount, GitProviderAccount,
};

use super::KomodoReadRequest;

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
