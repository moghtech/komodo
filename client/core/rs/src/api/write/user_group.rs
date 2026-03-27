use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::user_group::UserGroup;

use super::KomodoWriteRequest;
//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateUserGroup",
  description = "**Admin only.** Create a user group.",
  request_body(content = CreateUserGroup),
  responses(
    (status = 200, description = "The new user group", body = UserGroup),
  ),
)]
pub fn create_user_group() {}

/// **Admin only.** Create a user group. Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct CreateUserGroup {
  /// The name to assign to the new UserGroup
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameUserGroup",
  description = "**Admin only.** Rename a user group.",
  request_body(content = RenameUserGroup),
  responses(
    (status = 200, description = "The renamed user group", body = UserGroup),
  ),
)]
pub fn rename_user_group() {}

/// **Admin only.** Rename a user group. Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct RenameUserGroup {
  /// The id of the UserGroup
  pub id: String,
  /// The new name for the UserGroup
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteUserGroup",
  description = "**Admin only.** Delete a user group.",
  request_body(content = DeleteUserGroup),
  responses(
    (status = 200, description = "The deleted user group", body = UserGroup),
  ),
)]
pub fn delete_user_group() {}

/// **Admin only.** Delete a user group. Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct DeleteUserGroup {
  /// The id of the UserGroup
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/AddUserToUserGroup",
  description = "**Admin only.** Add a user to a user group.",
  request_body(content = AddUserToUserGroup),
  responses(
    (status = 200, description = "The updated user group", body = UserGroup),
  ),
)]
pub fn add_user_to_user_group() {}

/// **Admin only.** Add a user to a user group. Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct AddUserToUserGroup {
  /// The name or id of UserGroup that user should be added to.
  pub user_group: String,
  /// The id or username of the user to add
  pub user: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveUserFromUserGroup",
  description = "**Admin only.** Remove a user from a user group.",
  request_body(content = RemoveUserFromUserGroup),
  responses(
    (status = 200, description = "The updated user group", body = UserGroup),
  ),
)]
pub fn remove_user_from_user_group() {}

/// **Admin only.** Remove a user from a user group. Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct RemoveUserFromUserGroup {
  /// The name or id of UserGroup that user should be removed from.
  pub user_group: String,
  /// The id or username of the user to remove
  pub user: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SetUsersInUserGroup",
  description = "**Admin only.** Set users in a user group.",
  request_body(content = SetUsersInUserGroup),
  responses(
    (status = 200, description = "The updated user group", body = UserGroup),
  ),
)]
pub fn set_users_in_user_group() {}

/// **Admin only.** Completely override the users in the group.
/// Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct SetUsersInUserGroup {
  /// Id or name.
  pub user_group: String,
  /// The user ids or usernames to hard set as the group's users.
  pub users: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SetEveryoneUserGroup",
  description = "**Admin only.** Set everyone property of user group.",
  request_body(content = SetEveryoneUserGroup),
  responses(
    (status = 200, description = "The updated user group", body = UserGroup),
  ),
)]
pub fn set_everyone_user_group() {}

/// **Admin only.** Set `everyone` property of User Group.
/// Response: [UserGroup]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(mogh_error::Error)]
pub struct SetEveryoneUserGroup {
  /// Id or name.
  pub user_group: String,
  /// Whether this user group applies to everyone.
  pub everyone: bool,
}
