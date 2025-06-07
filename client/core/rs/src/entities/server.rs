use std::{collections::HashMap, path::PathBuf};

use chrono::Datelike;
use derive_builder::Builder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::deserializers::{
  option_string_list_deserializer, string_list_deserializer,
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerListItemInfo {
  /// The server's state.
  pub state: ServerState,
  /// Region of the server.
  pub region: String,
  /// Address of the server.
  pub address: String,
  /// Whether server is configured to send unreachable alerts.
  pub send_unreachable_alerts: bool,
  /// Whether server is configured to send cpu alerts.
  pub send_cpu_alerts: bool,
  /// Whether server is configured to send mem alerts.
  pub send_mem_alerts: bool,
  /// Whether server is configured to send disk alerts.
  pub send_disk_alerts: bool,
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
      address: Default::default(),
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
  Serialize,
  Deserialize,
  Debug,
  PartialEq,
  Hash,
  Eq,
  Clone,
  Copy,
  Default,
)]
pub enum ServerState {
  /// Server is unreachable.
  #[default]
  NotOk,
  /// Server health check passing.
  Ok,
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

/// Represents a scheduled maintenance window for a server
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaintenanceWindow {
  /// Unique identifier for the maintenance window
  pub id: String,
  /// Human-readable name for the maintenance window
  pub name: String,
  /// The type of maintenance schedule
  pub schedule_type: MaintenanceScheduleType,
  /// Start time for the maintenance window
  pub start_time: MaintenanceTime,
  /// Duration of the maintenance window in minutes
  pub duration_minutes: u32,
  /// Whether this maintenance window is currently enabled
  #[serde(default = "default_enabled")]
  pub enabled: bool,
  /// Optional description of what maintenance is performed
  #[serde(default)]
  pub description: String,
}

/// Types of maintenance schedules
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum MaintenanceScheduleType {
  /// Daily at the specified time
  Daily,
  /// Weekly on the specified day and time
  Weekly { day_of_week: DayOfWeek },
  /// One-time maintenance on a specific date and time
  OneTime { date: String }, // ISO 8601 date format (YYYY-MM-DD)
}

/// Days of the week for weekly maintenance schedules
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DayOfWeek {
  Monday,
  Tuesday,
  Wednesday,
  Thursday,
  Friday,
  Saturday,
  Sunday,
}

/// Time specification for maintenance windows
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MaintenanceTime {
  /// Hour in 24-hour format (0-23)
  pub hour: u8,
  /// Minute (0-59)
  pub minute: u8,
  /// Timezone offset from UTC in minutes (e.g., -300 for EST, 120 for CEST)
  pub timezone_offset_minutes: i16,
}

impl MaintenanceWindow {
  /// Check if the current timestamp falls within this maintenance window
  pub fn is_active_at(&self, timestamp: i64) -> bool {
    if !self.enabled {
      return false;
    }

    let dt = chrono::DateTime::from_timestamp(timestamp / 1000, 0)
      .unwrap_or_else(|| chrono::Utc::now());
    
    // Convert to the maintenance window's timezone
    let timezone_offset = chrono::FixedOffset::east_opt(self.start_time.timezone_offset_minutes as i32 * 60)
      .unwrap_or(chrono::FixedOffset::east_opt(0).unwrap());
    let local_dt = dt.with_timezone(&timezone_offset);

    match &self.schedule_type {
      MaintenanceScheduleType::Daily => {
        self.is_time_in_window(local_dt.time())
      }
      MaintenanceScheduleType::Weekly { day_of_week } => {
        let current_day = match local_dt.weekday() {
          chrono::Weekday::Mon => DayOfWeek::Monday,
          chrono::Weekday::Tue => DayOfWeek::Tuesday,
          chrono::Weekday::Wed => DayOfWeek::Wednesday,
          chrono::Weekday::Thu => DayOfWeek::Thursday,
          chrono::Weekday::Fri => DayOfWeek::Friday,
          chrono::Weekday::Sat => DayOfWeek::Saturday,
          chrono::Weekday::Sun => DayOfWeek::Sunday,
        };
        
        std::mem::discriminant(&current_day) == std::mem::discriminant(day_of_week)
          && self.is_time_in_window(local_dt.time())
      }
      MaintenanceScheduleType::OneTime { date } => {
        // Parse the date string and check if it matches current date
        if let Ok(maintenance_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
          local_dt.date_naive() == maintenance_date && self.is_time_in_window(local_dt.time())
        } else {
          false
        }
      }
    }
  }

  fn is_time_in_window(&self, current_time: chrono::NaiveTime) -> bool {
    let start_time = chrono::NaiveTime::from_hms_opt(
      self.start_time.hour as u32,
      self.start_time.minute as u32,
      0
    ).unwrap_or(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    
    let end_time = start_time + chrono::Duration::minutes(self.duration_minutes as i64);
    
    // Handle case where maintenance window crosses midnight
    if end_time < start_time {
      current_time >= start_time || current_time <= end_time
    } else {
      current_time >= start_time && current_time <= end_time
    }
  }
}
