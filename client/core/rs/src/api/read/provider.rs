use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::provider::{
  GitProviderAccount, ImageRegistryAccount,
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
  path = "/GetImageRegistryAccount",
  description = "Get a specific image registry account.",
  request_body(content = GetImageRegistryAccount),
  responses(
    (status = 200, description = "The image registry account", body = GetImageRegistryAccountResponse),
  ),
)]
pub fn get_image_registry_account() {}

/// Get a specific image registry account.
/// Response: [GetImageRegistryAccountResponse].
///
/// Pre v2.3.0, called `GetDockerRegistryAccount`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetImageRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct GetImageRegistryAccount {
  pub id: String,
}

#[typeshare]
pub type GetImageRegistryAccountResponse = ImageRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListImageRegistryAccounts",
  description = "List image registry accounts matching optional query.",
  request_body(content = ListImageRegistryAccounts),
  responses(
    (status = 200, description = "The list of image registry accounts", body = ListImageRegistryAccountsResponse),
  ),
)]
pub fn list_image_registry_accounts() {}

/// List image registry accounts matching optional query.
/// Response: [ListImageRegistryAccountsResponse].
///
/// Pre v2.3.0, called `ListDockerRegistryAccounts`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListImageRegistryAccountsResponse)]
#[error(mogh_error::Error)]
pub struct ListImageRegistryAccounts {
  /// Optionally filter by accounts with a specific domain.
  pub domain: Option<String>,
  /// Optionally filter by accounts with a specific username.
  pub username: Option<String>,
}

#[typeshare]
pub type ListImageRegistryAccountsResponse =
  Vec<ImageRegistryAccount>;
