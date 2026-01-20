use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ResourceTarget,
  permission::{Permission, PermissionLevelAndSpecifics, UserTarget},
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListPermissions",
  description = "List permissions for the calling user.",
  request_body(content = ListPermissions),
  responses(
    (status = 200, description = "The list of permissions", body = ListPermissionsResponse),
  ),
)]
pub fn list_permissions() {}

/// List permissions for the calling user.
/// Does not include any permissions on UserGroups they may be a part of.
/// Response: [ListPermissionsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListPermissionsResponse)]
#[error(mogh_error::Error)]
pub struct ListPermissions {}

#[typeshare]
pub type ListPermissionsResponse = Vec<Permission>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetPermission",
  description = "Gets the calling user's permission level on a specific resource.",
  request_body(content = GetPermission),
  responses(
    (status = 200, description = "The permission level", body = GetPermissionResponse),
  ),
)]
pub fn get_permission() {}

/// Gets the calling user's permission level on a specific resource.
/// Factors in any UserGroup's permissions they may be a part of.
/// Response: [PermissionLevel]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetPermissionResponse)]
#[error(mogh_error::Error)]
pub struct GetPermission {
  /// The target to get user permission on.
  pub target: ResourceTarget,
}

#[typeshare]
pub type GetPermissionResponse = PermissionLevelAndSpecifics;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListUserTargetPermissions",
  description = "List permissions for a specific user.",
  request_body(content = ListUserTargetPermissions),
  responses(
    (status = 200, description = "The list of permissions", body = ListUserTargetPermissionsResponse),
  ),
)]
pub fn list_user_target_permissions() {}

/// List permissions for a specific user. **Admin only**.
/// Response: [ListUserTargetPermissionsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListUserTargetPermissionsResponse)]
#[error(mogh_error::Error)]
pub struct ListUserTargetPermissions {
  /// Specify either a user or a user group.
  pub user_target: UserTarget,
}

#[typeshare]
pub type ListUserTargetPermissionsResponse = Vec<Permission>;
