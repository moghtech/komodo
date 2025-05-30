use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ResourceTarget,
  permission::{Permission, PermissionLevelAndSpecifics, UserTarget},
};

use super::KomodoReadRequest;

/// List permissions for the calling user.
/// Does not include any permissions on UserGroups they may be a part of.
/// Response: [ListPermissionsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListPermissionsResponse)]
#[error(serror::Error)]
pub struct ListPermissions {}

#[typeshare]
pub type ListPermissionsResponse = Vec<Permission>;

//

/// Gets the calling user's permission level on a specific resource.
/// Factors in any UserGroup's permissions they may be a part of.
/// Response: [PermissionLevel]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetPermissionResponse)]
#[error(serror::Error)]
pub struct GetPermission {
  /// The target to get user permission on.
  pub target: ResourceTarget,
}

#[typeshare]
pub type GetPermissionResponse = PermissionLevelAndSpecifics;

//

/// List permissions for a specific user. **Admin only**.
/// Response: [ListUserTargetPermissionsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListUserTargetPermissionsResponse)]
#[error(serror::Error)]
pub struct ListUserTargetPermissions {
  /// Specify either a user or a user group.
  pub user_target: UserTarget,
}

#[typeshare]
pub type ListUserTargetPermissionsResponse = Vec<Permission>;
