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
