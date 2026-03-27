use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{NoData, ResourceTarget, user::User};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PushRecentlyViewed",
  description = "Add a resource to calling user's recently viewed.",
  request_body(content = PushRecentlyViewed),
  responses(
    (status = 200, description = "Successful", body = PushRecentlyViewedResponse),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn push_recently_viewed() {}

/// Push a resource to the front of the users 10 most recently viewed resources.
/// Response: [NoData].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(PushRecentlyViewedResponse)]
#[error(mogh_error::Error)]
pub struct PushRecentlyViewed {
  /// The target to push.
  pub resource: ResourceTarget,
}

#[typeshare]
pub type PushRecentlyViewedResponse = NoData;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SetLastSeenUpdate",
  description = "Set the time the calling user most recently opened the UI updates dropdown.",
  request_body(content = SetLastSeenUpdate),
  responses(
    (status = 200, description = "Successful", body = SetLastSeenUpdateResponse),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn set_last_seen_update() {}

/// Set the time the calling user most recently opened the UI updates dropdown.
/// Used for unseen notification dot.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(SetLastSeenUpdateResponse)]
#[error(mogh_error::Error)]
pub struct SetLastSeenUpdate {}

#[typeshare]
pub type SetLastSeenUpdateResponse = NoData;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteUser",
  description = "**Admin only.** Delete a user.",
  request_body(content = DeleteUser),
  responses(
    (status = 200, description = "The deleted user", body = DeleteUserResponse),
  ),
)]
pub fn delete_user() {}

/// **Admin only**. Delete a user.
/// Admins can delete any non-admin user.
/// Only Super Admin can delete an admin.
/// No users can delete a Super Admin user.
/// User cannot delete themselves.
/// Response: [NoData].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteUserResponse)]
#[error(mogh_error::Error)]
pub struct DeleteUser {
  /// User id or username
  #[serde(alias = "username", alias = "id")]
  pub user: String,
}

#[typeshare]
pub type DeleteUserResponse = User;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateLocalUser",
  description = "**Admin only.** Create a local user.",
  request_body(content = CreateLocalUser),
  responses(
    (status = 200, description = "The new user", body = CreateLocalUserResponse),
  ),
)]
pub fn create_local_user() {}

/// **Admin only.** Create a local user.
/// Response: [User].
///
/// Note. Not to be confused with /auth/SignUpLocalUser.
/// This method requires admin user credentials, and can
/// bypass disabled user registration.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateLocalUserResponse)]
#[error(mogh_error::Error)]
pub struct CreateLocalUser {
  /// The username for the local user.
  pub username: String,
  /// A password for the local user.
  pub password: String,
}

#[typeshare]
pub type CreateLocalUserResponse = User;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateServiceUser",
  description = "**Admin only.** Create a service user.",
  request_body(content = CreateServiceUser),
  responses(
    (status = 200, description = "The new service user", body = CreateServiceUserResponse),
  ),
)]
pub fn create_service_user() {}

/// **Admin only.** Create a service user.
/// Response: [User].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateServiceUserResponse)]
#[error(mogh_error::Error)]
pub struct CreateServiceUser {
  /// The username for the service user.
  pub username: String,
  /// A description for the service user.
  pub description: String,
}

#[typeshare]
pub type CreateServiceUserResponse = User;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateServiceUserDescription",
  description = "**Admin only.** Update a service user's description.",
  request_body(content = UpdateServiceUserDescription),
  responses(
    (status = 200, description = "The updated user", body = UpdateServiceUserDescriptionResponse),
  ),
)]
pub fn update_service_user_description() {}

/// **Admin only.** Update a service user's description.
/// Response: [User].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateServiceUserDescriptionResponse)]
#[error(mogh_error::Error)]
pub struct UpdateServiceUserDescription {
  /// The service user's username
  pub username: String,
  /// A new description for the service user.
  pub description: String,
}

#[typeshare]
pub type UpdateServiceUserDescriptionResponse = User;
