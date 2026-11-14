use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ResourceTarget,
  provider::{
    ExternalSecretMetadataItem, GitProviderAccount,
    ImageRegistryAccount, PreviewResourceSecretsResponse,
    SecretProvider, SecretProviderGrant, SecretProviderKind,
    SecretProviderResourceUsage, SecretProviderSummary,
    SecretSelector,
  },
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

// ====================================
// = External Secret Provider (Admin) =
// ====================================

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetSecretProvider",
  description = "**Admin only.** Get a specific external secret provider configuration (never returns credentials).",
  request_body(content = GetSecretProvider),
  responses(
    (status = 200, description = "The secret provider", body = GetSecretProviderResponse),
  ),
)]
pub fn get_secret_provider() {}

/// **Admin only.** Get a specific external secret provider configuration by ID or slug.
/// Never returns credentials or encrypted ciphertexts.
/// Response: [GetSecretProviderResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSecretProviderResponse)]
#[error(mogh_error::Error)]
pub struct GetSecretProvider {
  pub id_or_slug: String,
}

#[typeshare]
pub type GetSecretProviderResponse = SecretProvider;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSecretProviders",
  description = "**Admin only.** List external secret providers (never returns credentials).",
  request_body(content = ListSecretProviders),
  responses(
    (status = 200, description = "The list of secret providers", body = ListSecretProvidersResponse),
  ),
)]
pub fn list_secret_providers() {}

/// **Admin only.** List external secret providers.
/// Response: [ListSecretProvidersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSecretProvidersResponse)]
#[error(mogh_error::Error)]
pub struct ListSecretProviders {
  pub kind: Option<SecretProviderKind>,
}

#[typeshare]
pub type ListSecretProvidersResponse = Vec<SecretProvider>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListAvailableSecretProviders",
  description = "List safe secret provider summaries available to the caller for resource secret configuration.",
  request_body(content = ListAvailableSecretProviders),
  responses(
    (status = 200, description = "Available secret provider summaries", body = ListAvailableSecretProvidersResponse),
  ),
)]
pub fn list_available_secret_providers() {}

/// List safe secret provider summaries available to the caller (for resource editors / key pickers).
/// Response: [ListAvailableSecretProvidersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListAvailableSecretProvidersResponse)]
#[error(mogh_error::Error)]
pub struct ListAvailableSecretProviders {
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListAvailableSecretProvidersResponse =
  Vec<SecretProviderSummary>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSecretProviderGrants",
  description = "**Admin only.** List authorization grants for external secret providers.",
  request_body(content = ListSecretProviderGrants),
  responses(
    (status = 200, description = "The list of secret provider grants", body = ListSecretProviderGrantsResponse),
  ),
)]
pub fn list_secret_provider_grants() {}

/// **Admin only.** List authorization grants for external secret providers.
/// Response: [ListSecretProviderGrantsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSecretProviderGrantsResponse)]
#[error(mogh_error::Error)]
pub struct ListSecretProviderGrants {
  pub provider_id: Option<String>,
}

#[typeshare]
pub type ListSecretProviderGrantsResponse = Vec<SecretProviderGrant>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSecretProviderUsages",
  description = "**Admin only.** List Stacks, Deployments, and Builds referencing a secret provider.",
  request_body(content = ListSecretProviderUsages),
  responses(
    (status = 200, description = "Resources using the secret provider", body = ListSecretProviderUsagesResponse),
  ),
)]
pub fn list_secret_provider_usages() {}

/// **Admin only.** List resources referencing a given secret provider (inline `[[vault.<slug>...]]` or structured bindings).
/// Response: [ListSecretProviderUsagesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSecretProviderUsagesResponse)]
#[error(mogh_error::Error)]
pub struct ListSecretProviderUsages {
  pub provider_id: String,
}

#[typeshare]
pub type ListSecretProviderUsagesResponse =
  Vec<SecretProviderResourceUsage>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListExternalSecretKeys",
  description = "List remote secret key names and metadata (never values) authorized for a target resource.",
  request_body(content = ListExternalSecretKeys),
  responses(
    (status = 200, description = "Remote secret metadata items", body = ListExternalSecretKeysResponse),
  ),
)]
pub fn list_external_secret_keys() {}

/// List remote secret key names and metadata (`viewSecretValue=false`, never secret values)
/// for an authorized caller, target resource, provider slug/ID, and selector.
/// Response: [ListExternalSecretKeysResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListExternalSecretKeysResponse)]
#[error(mogh_error::Error)]
pub struct ListExternalSecretKeys {
  pub target: ResourceTarget,
  /// Provider ID or immutable slug.
  pub provider: String,
  /// Optional explicit selector; falls back to the resource's `SecretSource` or provider `default_selector`.
  pub selector: Option<SecretSelector>,
}

#[typeshare]
pub type ListExternalSecretKeysResponse =
  Vec<ExternalSecretMetadataItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PreviewResourceSecrets",
  description = "Validate external secret references, permissions, and bindings on a resource without exposing secret values.",
  request_body(content = PreviewResourceSecrets),
  responses(
    (status = 200, description = "Non-sensitive secret preview status", body = PreviewResourceSecretsResponse),
  ),
)]
pub fn preview_resource_secrets() {}

/// Validate inline `[[vault.<slug>.<key>]]` references and structured bindings on a Stack, Deployment, or Build.
/// Returns variable/target names, referenced keys, and status — never returns plaintext values.
/// Response: [PreviewResourceSecretsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(PreviewResourceSecretsResponse)]
#[error(mogh_error::Error)]
pub struct PreviewResourceSecrets {
  pub target: ResourceTarget,
}
