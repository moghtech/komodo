use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  api::user::CreateApiKeyResponse,
  entities::{NoData, I64},
};

use super::KomodoWriteRequest;

//

/// Admin only method to create an api key for a service user.
/// Response: [CreateApiKeyResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateApiKeyForServiceUserResponse)]
#[error(serror::Error)]
pub struct CreateApiKeyForServiceUser {
  /// Must be service user
  pub user_id: String,
  /// The name for the api key
  pub name: String,
  /// A unix timestamp in millseconds specifying api key expire time.
  /// Default is 0, which means no expiry.
  #[serde(default)]
  pub expires: I64,
}

#[typeshare]
pub type CreateApiKeyForServiceUserResponse = CreateApiKeyResponse;

//

/// Admin only method to delete an api key for a service user.
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteApiKeyForServiceUserResponse)]
#[error(serror::Error)]
pub struct DeleteApiKeyForServiceUser {
  pub key: String,
}

#[typeshare]
pub type DeleteApiKeyForServiceUserResponse = NoData;
