use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::provider::*;

use super::KomodoWriteRequest;

/// **Admin only.** Create a git provider account.
/// Response: [GitProviderAccount].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateGitProviderAccountResponse)]
#[error(serror::Error)]
pub struct CreateGitProviderAccount {
  /// The initial account config. Anything in the _id field will be ignored,
  /// as this is generated on creation.
  pub account: _PartialGitProviderAccount,
}

#[typeshare]
pub type CreateGitProviderAccountResponse = GitProviderAccount;

//

/// **Admin only.** Update a git provider account.
/// Response: [GitProviderAccount].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateGitProviderAccountResponse)]
#[error(serror::Error)]
pub struct UpdateGitProviderAccount {
  /// The id of the git provider account to update.
  pub id: String,
  /// The partial git provider account.
  pub account: _PartialGitProviderAccount,
}

#[typeshare]
pub type UpdateGitProviderAccountResponse = GitProviderAccount;

//

/// **Admin only.** Delete a git provider account.
/// Response: [DeleteGitProviderAccountResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteGitProviderAccountResponse)]
#[error(serror::Error)]
pub struct DeleteGitProviderAccount {
  /// The id of the git provider to delete
  pub id: String,
}

#[typeshare]
pub type DeleteGitProviderAccountResponse = GitProviderAccount;

//

/// **Admin only.** Create a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateDockerRegistryAccountResponse)]
#[error(serror::Error)]
pub struct CreateDockerRegistryAccount {
  pub account: _PartialDockerRegistryAccount,
}

#[typeshare]
pub type CreateDockerRegistryAccountResponse = DockerRegistryAccount;

//

/// **Admin only.** Update a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateDockerRegistryAccountResponse)]
#[error(serror::Error)]
pub struct UpdateDockerRegistryAccount {
  /// The id of the docker registry to update
  pub id: String,
  /// The partial docker registry account.
  pub account: _PartialDockerRegistryAccount,
}

#[typeshare]
pub type UpdateDockerRegistryAccountResponse = DockerRegistryAccount;

//

/// **Admin only.** Delete a docker registry account.
/// Response: [DockerRegistryAccount].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteDockerRegistryAccountResponse)]
#[error(serror::Error)]
pub struct DeleteDockerRegistryAccount {
  /// The id of the docker registry account to delete
  pub id: String,
}

#[typeshare]
pub type DeleteDockerRegistryAccountResponse = DockerRegistryAccount;
