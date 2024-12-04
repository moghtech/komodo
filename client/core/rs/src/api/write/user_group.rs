use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::user_group::UserGroup;

use super::KomodoWriteRequest;

/// **Admin only.** Create a user group. Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct CreateUserGroup {
  /// The name to assign to the new UserGroup
  pub name: String,
}

//

/// **Admin only.** Rename a user group. Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct RenameUserGroup {
  /// The id of the UserGroup
  pub id: String,
  /// The new name for the UserGroup
  pub name: String,
}

//

/// **Admin only.** Delete a user group. Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct DeleteUserGroup {
  /// The id of the UserGroup
  pub id: String,
}

//

/// **Admin only.** Add a user to a user group. Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct AddUserToUserGroup {
  /// The name or id of UserGroup that user should be added to.
  pub user_group: String,
  /// The id or username of the user to add
  pub user: String,
}

//

/// **Admin only.** Remove a user from a user group. Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct RemoveUserFromUserGroup {
  /// The name or id of UserGroup that user should be removed from.
  pub user_group: String,
  /// The id or username of the user to remove
  pub user: String,
}

//

/// **Admin only.** Completely override the user in the group.
/// Response: [UserGroup]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UserGroup)]
#[error(serror::Error)]
pub struct SetUsersInUserGroup {
  /// Id or name.
  pub user_group: String,
  /// The user ids or usernames to hard set as the group's users.
  pub users: Vec<String>,
}
