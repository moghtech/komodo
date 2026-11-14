use indexmap::IndexSet;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, provider::*};

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
  path = "/CreateImageRegistryAccount",
  description = "**Admin only.** Create an image registry account.",
  request_body(content = CreateImageRegistryAccount),
  responses(
    (status = 200, description = "The created account", body = CreateImageRegistryAccountResponse),
  ),
)]
pub fn create_image_registry_account() {}

/// **Admin only.** Create an image registry account.
/// Response: [ImageRegistryAccount].
///
/// Pre v2.3.0, called `CreateDockerRegistryAccount`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateImageRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct CreateImageRegistryAccount {
  pub account: _PartialImageRegistryAccount,
}

#[typeshare]
pub type CreateImageRegistryAccountResponse = ImageRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateImageRegistryAccount",
  description = "**Admin only.** Update an image registry account.",
  request_body(content = UpdateImageRegistryAccount),
  responses(
    (status = 200, description = "The updated account", body = UpdateImageRegistryAccountResponse),
  ),
)]
pub fn update_image_registry_account() {}

/// **Admin only.** Update a image registry account.
/// Response: [ImageRegistryAccount].
///
/// Pre v2.3.0, called `UpdateDockerRegistryAccount`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateImageRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct UpdateImageRegistryAccount {
  /// The id of the image registry to update
  pub id: String,
  /// The partial image registry account.
  pub account: _PartialImageRegistryAccount,
}

#[typeshare]
pub type UpdateImageRegistryAccountResponse = ImageRegistryAccount;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteImageRegistryAccount",
  description = "**Admin only.** Delete an image registry account.",
  request_body(content = DeleteImageRegistryAccount),
  responses(
    (status = 200, description = "The deleted account", body = DeleteImageRegistryAccountResponse),
  ),
)]
pub fn delete_image_registry_account() {}

/// **Admin only.** Delete an image registry account.
/// Response: [ImageRegistryAccount].
///
/// Pre v2.3.0, called `DeleteDockerRegistryAccount`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteImageRegistryAccountResponse)]
#[error(mogh_error::Error)]
pub struct DeleteImageRegistryAccount {
  /// The id of the image registry account to delete
  pub id: String,
}

#[typeshare]
pub type DeleteImageRegistryAccountResponse = ImageRegistryAccount;

// ====================================
// = External Secret Provider (Admin) =
// ====================================

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateSecretProvider",
  description = "**Admin only.** Create an external secret provider.",
  request_body(content = CreateSecretProvider),
  responses(
    (status = 200, description = "The created secret provider", body = CreateSecretProviderResponse),
  ),
)]
pub fn create_secret_provider() {}

/// **Admin only.** Create an external secret provider.
/// Response: [CreateSecretProviderResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateSecretProviderResponse)]
#[error(mogh_error::Error)]
pub struct CreateSecretProvider {
  /// Immutable, globally unique slug (`[a-z0-9-]+`, no dots).
  pub slug: String,
  pub name: String,
  #[serde(default)]
  pub description: String,
  pub enabled: Option<bool>,
  pub config: SecretProviderConfig,
  pub credential: Option<SecretProviderCredentialInput>,
}

#[typeshare]
pub type CreateSecretProviderResponse = SecretProvider;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateSecretProvider",
  description = "**Admin only.** Update an external secret provider with optimistic concurrency (`expected_revision`) and explicit `Keep`/`Replace`/`Clear` credential semantics.",
  request_body(content = UpdateSecretProvider),
  responses(
    (status = 200, description = "The updated secret provider", body = UpdateSecretProviderResponse),
  ),
)]
pub fn update_secret_provider() {}

/// **Admin only.** Update an external secret provider.
/// Changing endpoint/identity/CA requires `credential_update = Replace(...)` or `Clear` (rejects `Keep`).
/// Response: [UpdateSecretProviderResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateSecretProviderResponse)]
#[error(mogh_error::Error)]
pub struct UpdateSecretProvider {
  pub id: String,
  pub expected_revision: I64,
  pub name: Option<String>,
  pub description: Option<String>,
  pub enabled: Option<bool>,
  pub config: Option<SecretProviderConfig>,
  #[serde(default)]
  pub credential_update: CredentialUpdate,
}

#[typeshare]
pub type UpdateSecretProviderResponse = SecretProvider;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteSecretProvider",
  description = "**Admin only.** Delete an external secret provider (fails if referenced by any Stack, Deployment, or Build).",
  request_body(content = DeleteSecretProvider),
  responses(
    (status = 200, description = "The deleted secret provider", body = DeleteSecretProviderResponse),
  ),
)]
pub fn delete_secret_provider() {}

/// **Admin only.** Delete an external secret provider if no resources depend on it.
/// Response: [DeleteSecretProviderResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteSecretProviderResponse)]
#[error(mogh_error::Error)]
pub struct DeleteSecretProvider {
  pub id: String,
}

#[typeshare]
pub type DeleteSecretProviderResponse = SecretProvider;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/TestSecretProviderConnection",
  description = "**Admin only.** Run a 3-stage connection test (authentication, metadata access, value read access) against a saved or draft secret provider.",
  request_body(content = TestSecretProviderConnection),
  responses(
    (status = 200, description = "Structured 3-stage connection test result", body = TestSecretProviderResponse),
  ),
)]
pub fn test_secret_provider_connection() {}

/// **Admin only.** Test authentication, metadata listing, and value-read permission on a saved or unsaved provider.
/// Response: [TestSecretProviderResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(TestSecretProviderResponse)]
#[error(mogh_error::Error)]
pub struct TestSecretProviderConnection {
  /// Optional existing provider ID (allows testing with stored credentials when `credential_update = Keep`,
  /// provided target/identity config has not changed).
  pub id: Option<String>,
  /// Optional config override (required when `id` is `None`).
  pub config: Option<SecretProviderConfig>,
  #[serde(default)]
  pub credential_update: CredentialUpdate,
  /// Optional scope selector to verify metadata access and value-read permission.
  pub selector: Option<SecretSelector>,
  /// Optional specific secret key name to verify actual value-read permission without returning the value.
  pub sample_key: Option<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateSecretProviderGrant",
  description = "**Admin only.** Create an authorization grant for a secret provider.",
  request_body(content = CreateSecretProviderGrant),
  responses(
    (status = 200, description = "The created grant", body = CreateSecretProviderGrantResponse),
  ),
)]
pub fn create_secret_provider_grant() {}

/// **Admin only.** Create an authorization grant for a secret provider.
/// Response: [CreateSecretProviderGrantResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateSecretProviderGrantResponse)]
#[error(mogh_error::Error)]
pub struct CreateSecretProviderGrant {
  pub provider_id: String,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: String,
  pub enabled: Option<bool>,
  pub subject: SecretGrantSubject,
  #[serde(default)]
  pub allowed_resources: Vec<SecretResourceScope>,
  pub selector_scope: SecretSelectorScope,
  #[serde(default)]
  #[cfg_attr(feature = "utoipa", schema(value_type = Vec<SecretUsage>))]
  pub allowed_usages: IndexSet<SecretUsage>,
}

#[typeshare]
pub type CreateSecretProviderGrantResponse = SecretProviderGrant;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateSecretProviderGrant",
  description = "**Admin only.** Update an authorization grant for a secret provider.",
  request_body(content = UpdateSecretProviderGrant),
  responses(
    (status = 200, description = "The updated grant", body = UpdateSecretProviderGrantResponse),
  ),
)]
pub fn update_secret_provider_grant() {}

/// **Admin only.** Update an authorization grant for a secret provider.
/// Response: [UpdateSecretProviderGrantResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateSecretProviderGrantResponse)]
#[error(mogh_error::Error)]
pub struct UpdateSecretProviderGrant {
  pub id: String,
  pub expected_revision: I64,
  pub name: Option<String>,
  pub description: Option<String>,
  pub enabled: Option<bool>,
  pub subject: Option<SecretGrantSubject>,
  pub allowed_resources: Option<Vec<SecretResourceScope>>,
  pub selector_scope: Option<SecretSelectorScope>,
  #[cfg_attr(
    feature = "utoipa",
    schema(value_type = Option<Vec<SecretUsage>>)
  )]
  pub allowed_usages: Option<IndexSet<SecretUsage>>,
}

#[typeshare]
pub type UpdateSecretProviderGrantResponse = SecretProviderGrant;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteSecretProviderGrant",
  description = "**Admin only.** Delete an authorization grant for a secret provider.",
  request_body(content = DeleteSecretProviderGrant),
  responses(
    (status = 200, description = "The deleted grant", body = DeleteSecretProviderGrantResponse),
  ),
)]
pub fn delete_secret_provider_grant() {}

/// **Admin only.** Delete an authorization grant for a secret provider.
/// Response: [DeleteSecretProviderGrantResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteSecretProviderGrantResponse)]
#[error(mogh_error::Error)]
pub struct DeleteSecretProviderGrant {
  pub id: String,
}

#[typeshare]
pub type DeleteSecretProviderGrantResponse = SecretProviderGrant;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RotateSecretProviderEncryptionKeys",
  description = "**Admin only.** Re-encrypt all stored secret provider credentials under the active `KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY`.",
  request_body(content = RotateSecretProviderEncryptionKeys),
  responses(
    (status = 200, description = "Rotation summary", body = RotateSecretProviderEncryptionKeysResponse),
  ),
)]
pub fn rotate_secret_provider_encryption_keys() {}

/// **Admin only.** Re-encrypt all stored secret provider credentials under the active master key.
/// Response: [RotateSecretProviderEncryptionKeysResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(RotateSecretProviderEncryptionKeysResponse)]
#[error(mogh_error::Error)]
pub struct RotateSecretProviderEncryptionKeys {}
