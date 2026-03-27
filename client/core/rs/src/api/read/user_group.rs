use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::user_group::UserGroup;

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetUserGroup",
  description = "Get a specific user group by name or id.",
  request_body(content = GetUserGroup),
  responses(
    (status = 200, description = "The user group", body = GetUserGroupResponse),
  ),
)]
pub fn get_user_group() {}

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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListUserGroups",
  description = "List all user groups which user can see.",
  request_body(content = ListUserGroups),
  responses(
    (status = 200, description = "The list of user groups", body = ListUserGroupsResponse),
  ),
)]
pub fn list_user_groups() {}

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
