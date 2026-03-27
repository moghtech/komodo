use clap::ValueEnum;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::entities::{api_key::ApiKey, user::User};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListApiKeys",
  description = "Gets list of api keys for the calling user.",
  request_body(content = ListApiKeys),
  responses(
    (status = 200, description = "The list of api keys", body = ListApiKeysResponse),
  ),
)]
pub fn list_api_keys() {}

/// Gets list of api keys for the calling user.
/// Response: [ListApiKeysResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListApiKeysResponse)]
#[error(mogh_error::Error)]
pub struct ListApiKeys {}

#[typeshare]
pub type ListApiKeysResponse = Vec<ApiKey>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListApiKeysForServiceUser",
  description = "**Admin only.** Gets list of api keys for the user.",
  request_body(content = ListApiKeysForServiceUser),
  responses(
    (status = 200, description = "The list of api keys", body = ListApiKeysForServiceUserResponse),
  ),
)]
pub fn list_api_keys_for_service_user() {}

/// **Admin only.**
/// Gets list of api keys for the user.
/// Will still fail if you call for a user_id that isn't a service user.
/// Response: [ListApiKeysForServiceUserResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListApiKeysForServiceUserResponse)]
#[error(mogh_error::Error)]
pub struct ListApiKeysForServiceUser {
  /// Id or username
  #[serde(alias = "id", alias = "username")]
  pub user: String,
}

#[typeshare]
pub type ListApiKeysForServiceUserResponse = Vec<ApiKey>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/FindUser",
  description = "**Admin only.** Find a user.",
  request_body(content = FindUser),
  responses(
    (status = 200, description = "The user", body = FindUserResponse),
  ),
)]
pub fn find_user() {}

/// **Admin only.**
/// Find a user.
/// Response: [FindUserResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(FindUserResponse)]
#[error(mogh_error::Error)]
pub struct FindUser {
  /// Id or username
  #[serde(alias = "id", alias = "username")]
  pub user: String,
}

#[typeshare]
pub type FindUserResponse = User;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListUsers",
  description = "**Admin only.** Gets list of Komodo users.",
  request_body(content = ListUsers),
  responses(
    (status = 200, description = "The list of users", body = Vec<User>),
  ),
)]
pub fn list_users() {}

/// **Admin only.**
/// Gets list of Komodo users.
/// Response: [ListUsersResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListUsersResponse)]
#[error(mogh_error::Error)]
pub struct ListUsers {
  /// Service user query options:
  ///   - Include (default)
  ///   - Exclude
  ///   - Only
  #[serde(default)]
  pub service_users: ServiceUserQueryBehavior,
}

#[typeshare]
pub type ListUsersResponse = Vec<User>;

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  Serialize,
  Deserialize,
  ValueEnum,
  Display,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
// Only strum serializes lowercase for clap compat.
#[strum(serialize_all = "lowercase")]
pub enum ServiceUserQueryBehavior {
  /// Include service users in results. Default.
  #[default]
  Include,
  /// Exclude service users from results.
  Exclude,
  /// Only include service users in results.
  Only,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetUsername",
  description = "Gets the username of a specific user.",
  request_body(content = GetUsername),
  responses(
    (status = 200, description = "The username response", body = GetUsernameResponse),
  ),
)]
pub fn get_username() {}

/// Gets the username of a specific user.
/// Response: [GetUsernameResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetUsernameResponse)]
#[error(mogh_error::Error)]
pub struct GetUsername {
  /// The id of the user.
  pub user_id: String,
}

/// Response for [GetUsername].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetUsernameResponse {
  /// The username of the user.
  pub username: String,
  /// An optional icon for the user.
  pub avatar: Option<String>,
}
