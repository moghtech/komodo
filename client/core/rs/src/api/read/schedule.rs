use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  deserializers::string_list_deserializer,
  entities::{resource::TagQueryBehavior, schedule::Schedule},
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSchedules",
  description = "List configured schedules.",
  request_body(content = ListSchedules),
  responses(
    (status = 200, description = "The list of schedules", body = ListSchedulesResponse),
  ),
)]
pub fn list_schedules() {}

/// List configured schedules.
/// Response: [ListSchedulesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSchedulesResponse)]
#[error(mogh_error::Error)]
pub struct ListSchedules {
  /// Pass Vec of tag ids or tag names
  #[serde(default, deserialize_with = "string_list_deserializer")]
  pub tags: Vec<String>,
  /// 'All' or 'Any'
  #[serde(default)]
  pub tag_behavior: TagQueryBehavior,
}

#[typeshare]
pub type ListSchedulesResponse = Vec<Schedule>;
