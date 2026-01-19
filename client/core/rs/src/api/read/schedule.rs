use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  deserializers::string_list_deserializer,
  entities::{resource::TagQueryBehavior, schedule::Schedule},
};

use super::KomodoReadRequest;

/// List configured schedules.
/// Response: [ListSchedulesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSchedulesResponse)]
#[error(serror::Error)]
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
