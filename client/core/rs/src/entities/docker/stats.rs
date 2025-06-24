use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::U64;

/// Statistics sample for a container.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct FullContainerStats {
  /// Name of the container
  pub name: String,

  /// ID of the container
  pub id: Option<String>,

  /// Date and time at which this sample was collected.
  /// The value is formatted as [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) with nano-seconds.
  pub read: Option<String>,

  /// Date and time at which this first sample was collected.
  /// This field is not propagated if the \"one-shot\" option is set.
  /// If the \"one-shot\" option is set, this field may be omitted, empty,
  /// or set to a default date (`0001-01-01T00:00:00Z`).
  /// The value is formatted as [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) with nano-seconds.
  pub preread: Option<String>,

  /// PidsStats contains Linux-specific stats of a container's process-IDs (PIDs).
  /// This type is Linux-specific and omitted for Windows containers.
  pub pids_stats: Option<ContainerPidsStats>,

  /// BlkioStats stores all IO service stats for data read and write.
  /// This type is Linux-specific and holds many fields that are specific to cgroups v1.
  /// On a cgroup v2 host, all fields other than `io_service_bytes_recursive` are omitted or `null`.
  /// This type is only populated on Linux and omitted for Windows containers.
  pub blkio_stats: Option<ContainerBlkioStats>,

  /// The number of processors on the system.
  /// This field is Windows-specific and always zero for Linux containers.
  pub num_procs: Option<u32>,

  #[serde(rename = "storage_stats")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub storage_stats: Option<ContainerStorageStats>,

  #[serde(rename = "cpu_stats")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpu_stats: Option<ContainerCpuStats>,

  #[serde(rename = "precpu_stats")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub precpu_stats: Option<ContainerCpuStats>,

  #[serde(rename = "memory_stats")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_stats: Option<ContainerMemoryStats>,

  /// Network statistics for the container per interface.  This field is omitted if the container has no networking enabled.
  #[serde(rename = "networks")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub networks: Option<ContainerNetworkStats>,
}

/// PidsStats contains Linux-specific stats of a container's process-IDs (PIDs).  This type is Linux-specific and omitted for Windows containers.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerPidsStats {
  /// Current is the number of PIDs in the cgroup.
  pub current: Option<U64>,

  /// Limit is the hard limit on the number of pids in the cgroup. A \"Limit\" of 0 means that there is no limit.
  pub limit: Option<U64>,
}

/// BlkioStats stores all IO service stats for data read and write.
/// This type is Linux-specific and holds many fields that are specific to cgroups v1.
/// On a cgroup v2 host, all fields other than `io_service_bytes_recursive` are omitted or `null`.
/// This type is only populated on Linux and omitted for Windows containers.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerBlkioStats {
  #[serde(rename = "io_service_bytes_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_service_bytes_recursive:
    Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_serviced_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_serviced_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_queue_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_queue_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_service_time_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_service_time_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_wait_time_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_wait_time_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_merged_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_merged_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "io_time_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub io_time_recursive: Option<Vec<ContainerBlkioStatEntry>>,

  /// This field is only available when using Linux containers with cgroups v1.
  /// It is omitted or `null` when using cgroups v2.
  #[serde(rename = "sectors_recursive")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sectors_recursive: Option<Vec<ContainerBlkioStatEntry>>,
}

/// Blkio stats entry.  This type is Linux-specific and omitted for Windows containers.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerBlkioStatEntry {
  pub major: Option<U64>,
  pub minor: Option<U64>,
  pub op: Option<String>,
  pub value: Option<U64>,
}

/// StorageStats is the disk I/O stats for read/write on Windows.
/// This type is Windows-specific and omitted for Linux containers.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerStorageStats {
  pub read_count_normalized: Option<U64>,
  pub read_size_bytes: Option<U64>,
  pub write_count_normalized: Option<U64>,
  pub write_size_bytes: Option<U64>,
}

/// CPU related info of the container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerCpuStats {
  /// All CPU stats aggregated since container inception.
  pub cpu_usage: Option<ContainerCpuUsage>,

  /// System Usage.
  /// This field is Linux-specific and omitted for Windows containers.
  pub system_cpu_usage: Option<U64>,

  /// Number of online CPUs.
  /// This field is Linux-specific and omitted for Windows containers.
  pub online_cpus: Option<u32>,

  /// CPU throttling stats of the container.
  /// This type is Linux-specific and omitted for Windows containers.
  pub throttling_data: Option<ContainerThrottlingData>,
}

/// All CPU stats aggregated since container inception.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerCpuUsage {
  /// Total CPU time consumed in nanoseconds (Linux) or 100's of nanoseconds (Windows).
  pub total_usage: Option<U64>,

  /// Total CPU time (in nanoseconds) consumed per core (Linux).
  /// This field is Linux-specific when using cgroups v1.
  /// It is omitted when using cgroups v2 and Windows containers.
  pub percpu_usage: Option<Vec<U64>>,

  /// Time (in nanoseconds) spent by tasks of the cgroup in kernel mode (Linux),
  /// or time spent (in 100's of nanoseconds) by all container processes in kernel mode (Windows).
  /// Not populated for Windows containers using Hyper-V isolation.
  pub usage_in_kernelmode: Option<U64>,

  /// Time (in nanoseconds) spent by tasks of the cgroup in user mode (Linux),
  /// or time spent (in 100's of nanoseconds) by all container processes in kernel mode (Windows).
  /// Not populated for Windows containers using Hyper-V isolation.
  pub usage_in_usermode: Option<U64>,
}

/// CPU throttling stats of the container.
/// This type is Linux-specific and omitted for Windows containers.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerThrottlingData {
  /// Number of periods with throttling active.
  pub periods: Option<U64>,

  /// Number of periods when the container hit its throttling limit.
  pub throttled_periods: Option<U64>,

  /// Aggregated time (in nanoseconds) the container was throttled for.
  pub throttled_time: Option<U64>,
}

/// Aggregates all memory stats since container inception on Linux.
/// Windows returns stats for commit and private working set only.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerMemoryStats {
  /// Current `res_counter` usage for memory.
  /// This field is Linux-specific and omitted for Windows containers.
  pub usage: Option<U64>,

  /// Maximum usage ever recorded.
  /// This field is Linux-specific and only supported on cgroups v1.
  /// It is omitted when using cgroups v2 and for Windows containers.
  pub max_usage: Option<U64>,

  /// All the stats exported via memory.stat. when using cgroups v2.
  /// This field is Linux-specific and omitted for Windows containers.
  pub stats: Option<HashMap<String, U64>>,

  /// Number of times memory usage hits limits.  This field is Linux-specific and only supported on cgroups v1. It is omitted when using cgroups v2 and for Windows containers.
  pub failcnt: Option<U64>,

  /// This field is Linux-specific and omitted for Windows containers.
  pub limit: Option<U64>,

  /// Committed bytes.
  /// This field is Windows-specific and omitted for Linux containers.
  pub commitbytes: Option<U64>,

  /// Peak committed bytes.
  /// This field is Windows-specific and omitted for Linux containers.
  pub commitpeakbytes: Option<U64>,

  /// Private working set.
  /// This field is Windows-specific and omitted for Linux containers.
  pub privateworkingset: Option<U64>,
}

/// Aggregates the network stats of one container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerNetworkStats {
  /// Bytes received. Windows and Linux.
  pub rx_bytes: Option<U64>,

  /// Packets received. Windows and Linux.
  pub rx_packets: Option<U64>,

  /// Received errors. Not used on Windows.
  /// This field is Linux-specific and always zero for Windows containers.
  pub rx_errors: Option<U64>,

  /// Incoming packets dropped. Windows and Linux.
  pub rx_dropped: Option<U64>,

  /// Bytes sent. Windows and Linux.
  pub tx_bytes: Option<U64>,

  /// Packets sent. Windows and Linux.
  pub tx_packets: Option<U64>,

  /// Sent errors. Not used on Windows.
  /// This field is Linux-specific and always zero for Windows containers.
  pub tx_errors: Option<U64>,

  /// Outgoing packets dropped. Windows and Linux.
  pub tx_dropped: Option<U64>,

  /// Endpoint ID. Not used on Linux.
  /// This field is Windows-specific and omitted for Linux containers.
  pub endpoint_id: Option<String>,

  /// Instance ID. Not used on Linux.
  /// This field is Windows-specific and omitted for Linux containers.
  pub instance_id: Option<String>,
}
