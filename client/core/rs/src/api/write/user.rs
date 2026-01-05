use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::user::User;

use super::KomodoWriteRequest;

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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

/// **Admin only.** Create a local user.
/// Response: [User].
///
/// Note. Not to be confused with /auth/SignUpLocalUser.
/// This method requires admin user credentials, and can
/// bypass disabled user registration.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateLocalUserResponse)]
#[error(serror::Error)]
pub struct CreateLocalUser {
  /// The username for the local user.
  pub username: String,
  /// A password for the local user.
  pub password: String,
}

#[typeshare]
pub type CreateLocalUserResponse = User;

//

/// **Admin only.** Create a service user.
/// Response: [User].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
