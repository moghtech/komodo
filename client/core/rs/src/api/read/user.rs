use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{api_key::ApiKey, user::User};

use super::KomodoReadRequest;

/// Gets list of api keys for the calling user.
/// Response: [ListApiKeysResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListApiKeysResponse)]
#[error(serror::Error)]
pub struct ListApiKeys {}

#[typeshare]
pub type ListApiKeysResponse = Vec<ApiKey>;

//

/// **Admin only.**
/// Gets list of api keys for the user.
/// Will still fail if you call for a user_id that isn't a service user.
/// Response: [ListApiKeysForServiceUserResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListApiKeysForServiceUserResponse)]
#[error(serror::Error)]
pub struct ListApiKeysForServiceUser {
  /// Id or username
  #[serde(alias = "id", alias = "username")]
  pub user: String,
}

#[typeshare]
pub type ListApiKeysForServiceUserResponse = Vec<ApiKey>;

//

/// **Admin only.**
/// Find a user.
/// Response: [FindUserResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(FindUserResponse)]
#[error(serror::Error)]
pub struct FindUser {
  /// Id or username
  #[serde(alias = "id", alias = "username")]
  pub user: String,
}

#[typeshare]
pub type FindUserResponse = User;

//

/// **Admin only.**
/// Gets list of Komodo users.
/// Response: [ListUsersResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListUsersResponse)]
#[error(serror::Error)]
pub struct ListUsers {}

#[typeshare]
pub type ListUsersResponse = Vec<User>;

//

/// Gets the username of a specific user.
/// Response: [GetUsernameResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetUsernameResponse)]
#[error(serror::Error)]
pub struct GetUsername {
  /// The id of the user.
  pub user_id: String,
}

/// Response for [GetUsername].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUsernameResponse {
  /// The username of the user.
  pub username: String,
  /// An optional icon for the user.
  pub avatar: Option<String>,
}
