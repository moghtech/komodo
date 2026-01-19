use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::user_group::UserGroup;

use super::KomodoReadRequest;

/// Get a specific user group by name or id.
/// Response: [UserGroup].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetUserGroupResponse)]
#[error(mogh_error::Error)]
pub struct GetUserGroup {
  /// Name or Id
  pub user_group: String,
}

#[typeshare]
pub type GetUserGroupResponse = UserGroup;

//

/// List all user groups which user can see. Response: [ListUserGroupsResponse].
///
/// Admins can see all user groups,
/// and users can see user groups to which they belong.
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListUserGroupsResponse)]
#[error(mogh_error::Error)]
pub struct ListUserGroups {}

#[typeshare]
pub type ListUserGroupsResponse = Vec<UserGroup>;
