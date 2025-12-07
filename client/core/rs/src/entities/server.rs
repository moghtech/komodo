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
  entities::{MaintenanceWindow, Timelength},
};

use super::{
  alert::SeverityLevel,
  resource::{AddFilters, Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type Server = Resource<ServerConfig, ServerInfo>;

#[typeshare]
pub type ServerListItem = ResourceListItem<ServerListItemInfo>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerListItemInfo {
  /// The server's state.
  pub state: ServerState,
  /// Region of the server.
  pub region: String,
  /// Address of the server, or null if empty.
  pub address: Option<String>,
  /// External address of the server (reachable by users).
  /// Used with links.
  pub external_address: Option<String>,
  /// Host public ip, if it could be resolved.
  pub public_ip: Option<String>,
  /// Whether server is configured to send disconnected alerts.
  pub send_unreachable_alerts: bool,
  /// Whether server is configured to send cpu alerts.
  pub send_cpu_alerts: bool,
  /// Whether server is configured to send mem alerts.
  pub send_mem_alerts: bool,
  /// Whether server is configured to send disk alerts.
  pub send_disk_alerts: bool,
  /// Whether server is configured to send version mismatch alerts.
  pub send_version_mismatch_alerts: bool,
  /// The Komodo Periphery version.
  pub version: Option<String>,
  /// The public key of Periphery
  pub public_key: Option<String>,
  /// If a Periphery fails to authenticate to Core with invalid Periphery public key,
  /// it will be stored here to accept the connection later on.
  pub attempted_public_key: Option<String>,
  /// Whether server is configured to send unreachable alerts.
  /// Whether terminals are disabled for this Server.
  pub terminals_disabled: bool,
  /// Whether container terminals are disabled for this Server.
  pub container_terminals_disabled: bool,
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerInfo {
  /// If a Periphery fails to authenticate to Core
  /// for a disconnected server with invalid Periphery public key,
  /// it will be stored here to accept the connection later on.
  #[serde(default)]
  pub attempted_public_key: String,
  /// The expected public key associated with
  /// private key of the periphery agent.
  #[serde(default)]
  pub public_key: String,
}

#[typeshare(serialized_as = "Partial<ServerConfig>")]
pub type _PartialServerConfig = PartialServerConfig;

/// Server configuration.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct ServerConfig {
  /// The ws/s address of the periphery client.
  /// If unset, Server expects Periphery -> Core connection.
  #[serde(default)]
  #[builder(default)]
  pub address: String,

  /// Only relevant for Core -> Periphery connections.
  /// Whether to skip Periphery tls certificate validation.
  /// This defaults to true because Periphery generates self-signed certificates by default,
  /// but if you use valid certs you can switch this to false.
  #[serde(default = "default_insecure_tls")]
  #[builder(default = "default_insecure_tls()")]
  #[partial_default(default_insecure_tls())]
  pub insecure_tls: bool,

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
  /// Default: false
  #[serde(default = "default_enabled")]
  #[builder(default = "default_enabled()")]
  #[partial_default(default_enabled())]
  pub enabled: bool,

  /// Whether to automatically rotate Server keys when
  /// RotateAllServerKeys is called.
  /// Default: true
  #[serde(default = "default_auto_rotate_keys")]
  #[builder(default = "default_auto_rotate_keys()")]
  #[partial_default(default_auto_rotate_keys())]
  pub auto_rotate_keys: bool,

  /// Deprecated. Use private / public keys instead.
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

  /// Whether to monitor any server stats beyond passing health check.
  /// default: true
  #[serde(default = "default_stats_monitoring")]
  #[builder(default = "default_stats_monitoring()")]
  #[partial_default(default_stats_monitoring())]
  pub stats_monitoring: bool,

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

fn default_insecure_tls() -> bool {
  // Peripheries use self signed certs by default
  true
}

fn default_enabled() -> bool {
  false
}

fn default_auto_rotate_keys() -> bool {
  true
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
      address: Default::default(),
      insecure_tls: default_insecure_tls(),
      external_address: Default::default(),
      enabled: default_enabled(),
      auto_rotate_keys: default_auto_rotate_keys(),
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

/// Info about Periphery configuration
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PeripheryInformation {
  /// The Periphery version.
  pub version: String,
  /// The public key of Periphery
  pub public_key: String,
  /// Whether terminals are disabled on this Periphery server
  pub terminals_disabled: bool,
  /// Whether container exec is disabled on this Periphery server
  pub container_terminals_disabled: bool,
  /// The rate the system stats are being polled from the system
  pub stats_polling_rate: Timelength,
  /// Whether Periphery is successfully connected to docker daemon.
  pub docker_connected: bool,
  /// The host public ip, if it can be resolved.
  pub public_ip: Option<String>,
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
