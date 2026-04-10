use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::{_Serror, MaintenanceWindow},
};

use super::resource::{Resource, ResourceListItem, ResourceQuery};

#[typeshare]
pub type SwarmListItem = ResourceListItem<SwarmListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmListItemInfo {
  /// Servers part of the swarm
  pub server_ids: Vec<String>,
  /// The Swarm state
  pub state: SwarmState,
  /// If there is an error reaching
  /// Swarm, message will be given here.
  pub err: Option<_Serror>,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SwarmState {
  /// All nodes /tasks OK
  Healthy,
  /// Some nodes / tasks don't match desired state
  Unhealthy,
  /// All nodes / tasks down.
  Down,
  /// Unknown case
  #[default]
  Unknown,
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::ToSchema)]
#[schema(as = Swarm)]
pub struct SwarmSchema(
  #[schema(inline)] pub Resource<SwarmConfig, SwarmInfo>,
);

#[typeshare]
pub type Swarm = Resource<SwarmConfig, SwarmInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmInfo {}

#[typeshare(serialized_as = "Partial<SwarmConfig>")]
pub type _PartialSwarmConfig = PartialSwarmConfig;

#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, Builder, Partial,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "schemars",
  partial_derive(schemars::JsonSchema)
)]
#[diff_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct SwarmConfig {
  /// The Servers which are swarm manager nodes.
  /// If a Server is not reachable or gives error,
  /// tries the next Server.
  #[serde(default, alias = "servers")]
  #[partial_attr(serde(alias = "servers"))]
  #[cfg_attr(
    feature = "schemars",
    partial_attr(schemars(rename = "servers"))
  )]
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

  /// Whether to send alerts about the swarm health.
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_unhealthy_alerts: bool,

  /// Scheduled maintenance windows during which alerts will be suppressed.
  #[serde(default)]
  #[builder(default)]
  pub maintenance_windows: Vec<MaintenanceWindow>,
}

fn default_send_alerts() -> bool {
  true
}

#[cfg(feature = "utoipa")]
impl utoipa::PartialSchema for PartialSwarmConfig {
  fn schema()
  -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    utoipa::schema!(#[inline] std::collections::HashMap<String, serde_json::Value>).into()
  }
}

#[cfg(feature = "utoipa")]
impl utoipa::ToSchema for PartialSwarmConfig {}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmActionState {}

#[typeshare]
pub type SwarmQuery = ResourceQuery<SwarmQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
