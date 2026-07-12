use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  deserializers::string_list_deserializer,
  entities::{
    U64,
    resource::TagQueryBehavior,
    schedule::{Schedule, ScheduleSortBy},
  },
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resolve)]
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

  /// Filter by target name.
  /// Returned schedules have names which contain all terms.
  #[serde(default)]
  pub terms: Vec<String>,

  /// Retrieve more results by incrementing the page.
  /// `page: 0` is default.
  #[serde(default)]
  pub page: U64,

  /// Set the limit for number of schedules per-page.
  /// `limit: 100` is default.
  ///
  /// Passing `limit: 0` returns all results (unlimited).
  ///
  /// Note: the page logic relies on this being consistent
  /// across queries for more pages.
  pub limit: Option<U64>,

  /// Sort the results by this field.
  /// Defaults to Name.
  #[serde(default)]
  pub sort_by: ScheduleSortBy,

  /// Reverse the sort direction.
  #[serde(default)]
  pub sort_desc: bool,
}

#[typeshare]
pub type ListSchedulesResponse = Vec<Schedule>;
