use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{user::User, NoData};

use super::KomodoWriteRequest;

//

/// **Only for local users**. Update the calling users username.
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateUserUsernameResponse)]
#[error(serror::Error)]
pub struct UpdateUserUsername {
  pub username: String,
}

#[typeshare]
pub type UpdateUserUsernameResponse = NoData;

//

/// **Only for local users**. Update the calling users password.
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateUserPasswordResponse)]
#[error(serror::Error)]
pub struct UpdateUserPassword {
  pub password: String,
}

#[typeshare]
pub type UpdateUserPasswordResponse = NoData;

//

/// **Admin only**. Delete a user.
/// Admins can delete any non-admin user.
/// Only Super Admin can delete an admin.
/// No users can delete a Super Admin user.
/// User cannot delete themselves.
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteUserResponse)]
#[error(serror::Error)]
pub struct DeleteUser {
  /// User id or username
  #[serde(alias = "username", alias = "id")]
  pub user: String,
}

#[typeshare]
pub type DeleteUserResponse = User;

//

/// **Admin only.** Create a service user.
/// Response: [User].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateServiceUserResponse)]
#[error(serror::Error)]
pub struct CreateServiceUser {
  /// The username for the service user.
  pub username: String,
  /// A description for the service user.
  pub description: String,
}

#[typeshare]
pub type CreateServiceUserResponse = User;

//

/// **Admin only.** Update a service user's description.
/// Response: [User].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateServiceUserDescriptionResponse)]
#[error(serror::Error)]
pub struct UpdateServiceUserDescription {
  /// The service user's username
  pub username: String,
  /// A new description for the service user.
  pub description: String,
}

#[typeshare]
pub type UpdateServiceUserDescriptionResponse = User;
