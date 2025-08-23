use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, Timelength};

/// System information of a server
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SystemInformation {
  /// The system name
  pub name: Option<String>,
  /// The system long os version
  pub os: Option<String>,
  /// System's kernel version
  pub kernel: Option<String>,
  /// Physical core count
  pub core_count: Option<u32>,
  /// System hostname based off DNS
  pub host_name: Option<String>,
  /// The CPU's brand
  pub cpu_brand: String,
  /// Whether terminals are disabled on this Periphery server
  pub terminals_disabled: bool,
  /// Whether container exec is disabled on this Periphery server
  pub container_exec_disabled: bool,
}

/// System stats stored on the database.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", collection_name(Stats))]
pub struct SystemStatsRecord {
  /// Unix timestamp in milliseconds
  #[cfg_attr(feature = "mongo", index)]
  pub ts: I64,
  /// Server id
  #[cfg_attr(feature = "mongo", index)]
  pub sid: String,
  // basic stats
  /// Cpu usage percentage
  pub cpu_perc: f32,
  /// Load average (1m, 5m, 15m)
  pub load_average: SystemLoadAverage,
  /// Memory used in GB
  pub mem_used_gb: f64,
  /// Total memory in GB
  pub mem_total_gb: f64,
  /// Disk used in GB
  pub disk_used_gb: f64,
  /// Total disk size in GB
  pub disk_total_gb: f64,
  /// Breakdown of individual disks, including their usage, total size, and mount point
  pub disks: Vec<SingleDiskUsage>,
  /// Total network ingress in bytes
  #[serde(default)]
  pub network_ingress_bytes: f64,
  /// Total network egress in bytes
  #[serde(default)]
  pub network_egress_bytes: f64,
  // /// Network usage by interface name (ingress, egress in bytes)
  // #[serde(default)]
  // pub network_usage_interface: Vec<SingleNetworkInterfaceUsage>, // interface -> (ingress, egress)
}

/// Realtime system stats data.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SystemStats {
  /// Cpu usage percentage
  pub cpu_perc: f32,
  ///  Load average (1m, 5m, 15m)
  pub load_average: SystemLoadAverage,
  /// [1.15.9+]
  /// Free memory in GB.
  /// This is really the 'Free' memory, not the 'Available' memory.
  /// It may be different than mem_total_gb - mem_used_gb.
  #[serde(default)]
  pub mem_free_gb: f64,
  /// Used memory in GB. 'Total' - 'Available' (not free) memory.
  pub mem_used_gb: f64,
  /// Total memory in GB
  pub mem_total_gb: f64,
  /// Breakdown of individual disks, ie their usages, sizes, and mount points
  pub disks: Vec<SingleDiskUsage>,
  /// Network ingress usage in MB
  #[serde(default)]
  pub network_ingress_bytes: f64,
  /// Network egress usage in MB
  #[serde(default)]
  pub network_egress_bytes: f64,
  // /// Network usage by interface name (ingress, egress in bytes)
  // #[serde(default)]
  // pub network_usage_interface: Vec<SingleNetworkInterfaceUsage>, // interface -> (ingress, egress)
  // metadata
  /// The rate the system stats are being polled from the system
  pub polling_rate: Timelength,
  /// Unix timestamp in milliseconds when stats were last polled
  pub refresh_ts: I64,
  /// Unix timestamp in milliseconds when disk list was last refreshed
  pub refresh_list_ts: I64,
}

/// Info for a single disk mounted on the system.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleDiskUsage {
  /// The mount point of the disk
  pub mount: PathBuf,
  /// Detected file system
  pub file_system: String,
  /// Used portion of the disk in GB
  pub used_gb: f64,
  /// Total size of the disk in GB
  pub total_gb: f64,
}

/// Info for network interface usage.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleNetworkInterfaceUsage {
  /// The network interface name
  pub name: String,
  /// The ingress in bytes
  pub ingress_bytes: f64,
  /// The egress in bytes
  pub egress_bytes: f64,
}

pub fn sum_disk_usage(disks: &[SingleDiskUsage]) -> TotalDiskUsage {
  disks
    .iter()
    .fold(TotalDiskUsage::default(), |mut total, disk| {
      total.used_gb += disk.used_gb;
      total.total_gb += disk.total_gb;
      total
    })
}

/// Info for the all system disks combined.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TotalDiskUsage {
  /// Used portion in GB
  pub used_gb: f64,
  /// Total size in GB
  pub total_gb: f64,
}

/// Information about a process on the system.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemProcess {
  /// The process PID
  pub pid: u32,
  /// The process name
  pub name: String,
  /// The path to the process executable
  #[serde(default)]
  pub exe: String,
  /// The command used to start the process
  pub cmd: Vec<String>,
  /// The time the process was started
  #[serde(default)]
  pub start_time: f64,
  /// The cpu usage percentage of the process.
  /// This is in core-percentage, eg 100% is 1 full core, and
  /// an 8 core machine would max at 800%.
  pub cpu_perc: f32,
  /// The memory usage of the process in MB
  pub mem_mb: f64,
  /// Process disk read in KB/s
  pub disk_read_kb: f64,
  /// Process disk write in KB/s
  pub disk_write_kb: f64,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SystemLoadAverage {
  /// 1m load average
  pub one: f64,
  /// 5m load average
  pub five: f64,
  /// 15m load average
  pub fifteen: f64,
}