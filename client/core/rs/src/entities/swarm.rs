use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::deserializers::{
  option_string_list_deserializer, string_list_deserializer,
};

use super::resource::{Resource, ResourceListItem, ResourceQuery};

#[typeshare]
pub type SwarmListItem = ResourceListItem<SwarmListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmListItemInfo {
  /// Servers part of the swarm
  pub server_ids: Vec<String>,
  /// The Swarm state
  pub state: SwarmState,
  /// If there is an error reaching
  /// Swarm, message will be given here.
  pub err: Option<String>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
  Display,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SwarmState {
  /// The Swarm is healthy, all nodes OK
  Healthy,
  /// The Swarm is unhealthy
  Unhealthy,
  /// Unknown case
  #[default]
  Unknown,
}

#[typeshare]
pub type Swarm = Resource<SwarmConfig, SwarmInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmInfo {}

#[typeshare(serialized_as = "Partial<SwarmConfig>")]
pub type _PartialSwarmConfig = PartialSwarmConfig;

#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, Builder, Partial,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[diff_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct SwarmConfig {
  /// The Servers which are swarm manager nodes.
  /// If a Server is not reachable or gives error,
  /// tries the next Server.
  #[serde(default, alias = "servers")]
  #[partial_attr(serde(alias = "servers"))]
  #[builder(default)]
  pub server_ids: Vec<String>,

  /// Configure quick links that are displayed in the resource header
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub links: Vec<String>,
}

#[cfg(feature = "openapi")]
impl utoipa::PartialSchema for PartialSwarmConfig {
  fn schema()
  -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    utoipa::schema!(#[inline] std::collections::HashMap<String, serde_json::Value>).into()
  }
}

#[cfg(feature = "openapi")]
impl utoipa::ToSchema for PartialSwarmConfig {}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmActionState {}

#[typeshare]
pub type SwarmQuery = ResourceQuery<SwarmQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmQuerySpecifics {
  /// Filter swarms by server ids.
  pub servers: Vec<String>,
}

impl super::resource::AddFilters for SwarmQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if !self.servers.is_empty() {
      filters
        .insert("config.server_ids", doc! { "$in": &self.servers });
    }
  }
}
