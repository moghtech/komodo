use std::{collections::HashMap, path::PathBuf};

use derive_builder::Builder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::MaintenanceWindow,
};

use super::{
  I64,
  alert::SeverityLevel,
  resource::{AddFilters, Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type Server = Resource<ServerConfig, ()>;

#[typeshare]
pub type ServerListItem = ResourceListItem<ServerListItemInfo>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerListItemInfo {
  /// The server's state.
  pub state: ServerState,
  /// Region of the server.
  pub region: String,
  /// Address of the server.
  pub address: String,
  /// External address of the server (reachable by users).
  /// Used with links.
  #[serde(default)] // API backward compat
  pub external_address: String,
  /// The Komodo Periphery version of the server.
  pub version: String,
  /// Whether server is configured to send unreachable alerts.
  pub send_unreachable_alerts: bool,
  /// Whether server is configured to send cpu alerts.
  pub send_cpu_alerts: bool,
  /// Whether server is configured to send mem alerts.
  pub send_mem_alerts: bool,
  /// Whether server is configured to send disk alerts.
  pub send_disk_alerts: bool,
  /// Whether server is configured to send version mismatch alerts.
  #[serde(default)] // API backward compat
  pub send_version_mismatch_alerts: bool,
  /// Whether terminals are disabled for this Server.
  pub terminals_disabled: bool,
  /// Whether container exec is disabled for this Server.
  pub container_exec_disabled: bool,
}

#[typeshare(serialized_as = "Partial<ServerConfig>")]
pub type _PartialServerConfig = PartialServerConfig;

/// Server configuration.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct ServerConfig {
  /// The http address of the periphery client.
  /// Default: http://localhost:8120
  #[serde(default = "default_address")]
  #[builder(default = "default_address()")]
  #[partial_default(default_address())]
  pub address: String,

  /// The address to use with links for containers on the server.
  /// If empty, will use the 'address' for links.
  #[serde(default)]
  #[builder(default)]
  pub external_address: String,

  /// An optional region label
  #[serde(default)]
  #[builder(default)]
  pub region: String,

  /// Whether a server is enabled.
  /// If a server is disabled,
  /// you won't be able to perform any actions on it or see deployment's status.
  /// default: true
  #[serde(default = "default_enabled")]
  #[builder(default = "default_enabled()")]
  #[partial_default(default_enabled())]
  pub enabled: bool,

  /// The timeout used to reach the server in seconds.
  /// default: 2
  #[serde(default = "default_timeout_seconds")]
  #[builder(default = "default_timeout_seconds()")]
  #[partial_default(default_timeout_seconds())]
  pub timeout_seconds: I64,

  /// An optional override passkey to use
  /// to authenticate with periphery agent.
  /// If this is empty, will use passkey in core config.
  #[serde(default)]
  #[builder(default)]
  pub passkey: String,

  /// Sometimes the system stats reports a mount path that is not desired.
  /// Use this field to filter it out from the report.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub ignore_mounts: Vec<String>,

  /// Whether to monitor any server stats beyond passing health check.
  /// default: true
  #[serde(default = "default_stats_monitoring")]
  #[builder(default = "default_stats_monitoring()")]
  #[partial_default(default_stats_monitoring())]
  pub stats_monitoring: bool,

  /// Whether to trigger 'docker image prune -a -f' every 24 hours.
  /// default: true
  #[serde(default = "default_auto_prune")]
  #[builder(default = "default_auto_prune()")]
  #[partial_default(default_auto_prune())]
  pub auto_prune: bool,

  /// Configure quick links that are displayed in the resource header
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub links: Vec<String>,

  /// Whether to send alerts about the servers reachability
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_unreachable_alerts: bool,

  /// Whether to send alerts about the servers CPU status
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_cpu_alerts: bool,

  /// Whether to send alerts about the servers MEM status
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_mem_alerts: bool,

  /// Whether to send alerts about the servers DISK status
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_disk_alerts: bool,

  /// Whether to send alerts about the servers version mismatch with core
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_version_mismatch_alerts: bool,

  /// The percentage threshhold which triggers WARNING state for CPU.
  #[serde(default = "default_cpu_warning")]
  #[builder(default = "default_cpu_warning()")]
  #[partial_default(default_cpu_warning())]
  pub cpu_warning: f32,

  /// The percentage threshhold which triggers CRITICAL state for CPU.
  #[serde(default = "default_cpu_critical")]
  #[builder(default = "default_cpu_critical()")]
  #[partial_default(default_cpu_critical())]
  pub cpu_critical: f32,

  /// The percentage threshhold which triggers WARNING state for MEM.
  #[serde(default = "default_mem_warning")]
  #[builder(default = "default_mem_warning()")]
  #[partial_default(default_mem_warning())]
  pub mem_warning: f64,

  /// The percentage threshhold which triggers CRITICAL state for MEM.
  #[serde(default = "default_mem_critical")]
  #[builder(default = "default_mem_critical()")]
  #[partial_default(default_mem_critical())]
  pub mem_critical: f64,

  /// The percentage threshhold which triggers WARNING state for DISK.
  #[serde(default = "default_disk_warning")]
  #[builder(default = "default_disk_warning()")]
  #[partial_default(default_disk_warning())]
  pub disk_warning: f64,

  /// The percentage threshhold which triggers CRITICAL state for DISK.
  #[serde(default = "default_disk_critical")]
  #[builder(default = "default_disk_critical()")]
  #[partial_default(default_disk_critical())]
  pub disk_critical: f64,

  /// Scheduled maintenance windows during which alerts will be suppressed.
  #[serde(default)]
  #[builder(default)]
  pub maintenance_windows: Vec<MaintenanceWindow>,
}

impl ServerConfig {
  pub fn builder() -> ServerConfigBuilder {
    ServerConfigBuilder::default()
  }
}

fn default_address() -> String {
  String::from("https://periphery:8120")
}

fn default_enabled() -> bool {
  false
}

fn default_timeout_seconds() -> i64 {
  3
}

fn default_stats_monitoring() -> bool {
  true
}

fn default_auto_prune() -> bool {
  true
}

fn default_send_alerts() -> bool {
  true
}

fn default_cpu_warning() -> f32 {
  90.0
}

fn default_cpu_critical() -> f32 {
  99.0
}

fn default_mem_warning() -> f64 {
  75.0
}

fn default_mem_critical() -> f64 {
  95.0
}

fn default_disk_warning() -> f64 {
  75.0
}

fn default_disk_critical() -> f64 {
  95.0
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      address: default_address(),
      external_address: Default::default(),
      enabled: default_enabled(),
      timeout_seconds: default_timeout_seconds(),
      ignore_mounts: Default::default(),
      stats_monitoring: default_stats_monitoring(),
      auto_prune: default_auto_prune(),
      links: Default::default(),
      send_unreachable_alerts: default_send_alerts(),
      send_cpu_alerts: default_send_alerts(),
      send_mem_alerts: default_send_alerts(),
      send_disk_alerts: default_send_alerts(),
      send_version_mismatch_alerts: default_send_alerts(),
      region: Default::default(),
      passkey: Default::default(),
      cpu_warning: default_cpu_warning(),
      cpu_critical: default_cpu_critical(),
      mem_warning: default_mem_warning(),
      mem_critical: default_mem_critical(),
      disk_warning: default_disk_warning(),
      disk_critical: default_disk_critical(),
      maintenance_windows: Default::default(),
    }
  }
}

/// The health of a part of the server.
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ServerHealthState {
  pub level: SeverityLevel,
  /// Whether the health is good enough to close an open alert.
  pub should_close_alert: bool,
}

/// Summary of the health of the server.
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ServerHealth {
  pub cpu: ServerHealthState,
  pub mem: ServerHealthState,
  pub disks: HashMap<PathBuf, ServerHealthState>,
}

/// Info about an active terminal on a server.
/// Retrieve with [ListTerminals][crate::api::read::server::ListTerminals].
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct TerminalInfo {
  /// The name of the terminal.
  pub name: String,
  /// The root program / args of the pty
  pub command: String,
  /// The size of the terminal history in memory.
  pub stored_size_kb: f64,
}

/// Current pending actions on the server.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ServerActionState {
  /// Server currently pruning networks
  pub pruning_networks: bool,
  /// Server currently pruning containers
  pub pruning_containers: bool,
  /// Server currently pruning images
  pub pruning_images: bool,
  /// Server currently pruning volumes
  pub pruning_volumes: bool,
  /// Server currently pruning docker builders
  pub pruning_builders: bool,
  /// Server currently pruning builx cache
  pub pruning_buildx: bool,
  /// Server currently pruning system
  pub pruning_system: bool,
  /// Server currently starting containers.
  pub starting_containers: bool,
  /// Server currently restarting containers.
  pub restarting_containers: bool,
  /// Server currently pausing containers.
  pub pausing_containers: bool,
  /// Server currently unpausing containers.
  pub unpausing_containers: bool,
  /// Server currently stopping containers.
  pub stopping_containers: bool,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Default,
  Display,
  Serialize,
  Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
pub enum ServerState {
  /// Server health check passing.
  Ok,
  /// Server is unreachable.
  #[default]
  NotOk,
  /// Server is disabled.
  Disabled,
}

/// Server-specific query
#[typeshare]
pub type ServerQuery = ResourceQuery<ServerQuerySpecifics>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ServerQuerySpecifics {}

impl AddFilters for ServerQuerySpecifics {}
