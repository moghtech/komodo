use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, Usize};

use super::{ContainerConfig, GraphDriverData, PortBinding};

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerListItem {
  /// The Server which holds the container.
  pub server_id: Option<String>,
  /// The first name in Names, not including the initial '/'
  pub name: String,
  /// The ID of this container
  pub id: Option<String>,
  /// The name of the image used when creating this container
  pub image: Option<String>,
  /// The ID of the image that this container was created from
  pub image_id: Option<String>,
  /// When the container was created
  pub created: Option<I64>,
  /// The size of files that have been created or changed by this container
  pub size_rw: Option<I64>,
  /// The total size of all the files in this container
  pub size_root_fs: Option<I64>,
  /// The state of this container (e.g. `exited`)
  pub state: ContainerStateStatusEnum,
  /// Additional human-readable status of this container (e.g. `Exit 0`)
  pub status: Option<String>,
  /// The network mode
  pub network_mode: Option<String>,
  /// The network names attached to container
  pub networks: Vec<String>,
  /// Port mappings for the container
  pub ports: Vec<Port>,
  /// The volume names attached to container
  pub volumes: Vec<String>,
  /// The labels attached to container.
  /// It's too big to send with container list,
  /// can get it using InspectContainer
  #[serde(default, skip_serializing)]
  pub labels: HashMap<String, String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NameAndId {
  pub name: String,
  pub id: String,
}

/// An open port on a container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Port {
  /// Host IP address that the container's port is mapped to
  #[serde(rename = "IP")]
  pub ip: Option<String>,

  /// Port on the container
  #[serde(default, rename = "PrivatePort")]
  pub private_port: u16,

  /// Port exposed on the host
  #[serde(rename = "PublicPort")]
  pub public_port: Option<u16>,

  #[serde(default, rename = "Type")]
  pub typ: PortTypeEnum,
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
)]
pub enum PortTypeEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "tcp")]
  TCP,
  #[serde(rename = "udp")]
  UDP,
  #[serde(rename = "sctp")]
  SCTP,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Container {
  /// The ID of the container
  #[serde(rename = "Id")]
  pub id: Option<String>,

  /// The time the container was created
  #[serde(rename = "Created")]
  pub created: Option<String>,

  /// The path to the command being run
  #[serde(rename = "Path")]
  pub path: Option<String>,

  /// The arguments to the command being run
  #[serde(default, rename = "Args")]
  pub args: Vec<String>,

  #[serde(rename = "State")]
  pub state: Option<ContainerState>,

  /// The container's image ID
  #[serde(rename = "Image")]
  pub image: Option<String>,

  #[serde(rename = "ResolvConfPath")]
  pub resolv_conf_path: Option<String>,

  #[serde(rename = "HostnamePath")]
  pub hostname_path: Option<String>,

  #[serde(rename = "HostsPath")]
  pub hosts_path: Option<String>,

  #[serde(rename = "LogPath")]
  pub log_path: Option<String>,

  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "RestartCount")]
  pub restart_count: Option<I64>,

  #[serde(rename = "Driver")]
  pub driver: Option<String>,

  #[serde(rename = "Platform")]
  pub platform: Option<String>,

  #[serde(rename = "MountLabel")]
  pub mount_label: Option<String>,

  #[serde(rename = "ProcessLabel")]
  pub process_label: Option<String>,

  #[serde(rename = "AppArmorProfile")]
  pub app_armor_profile: Option<String>,

  /// IDs of exec instances that are running in the container.
  #[serde(default, rename = "ExecIDs")]
  pub exec_ids: Vec<String>,

  #[serde(rename = "HostConfig")]
  pub host_config: Option<HostConfig>,

  #[serde(rename = "GraphDriver")]
  pub graph_driver: Option<GraphDriverData>,

  /// The size of files that have been created or changed by this container.
  #[serde(rename = "SizeRw")]
  pub size_rw: Option<I64>,

  /// The total size of all the files in this container.
  #[serde(rename = "SizeRootFs")]
  pub size_root_fs: Option<I64>,

  #[serde(default, rename = "Mounts")]
  pub mounts: Vec<MountPoint>,

  #[serde(rename = "Config")]
  pub config: Option<ContainerConfig>,

  #[serde(rename = "NetworkSettings")]
  pub network_settings: Option<NetworkSettings>,
}

/// ContainerState stores container's running state. It's part of ContainerJSONBase and will be returned by the \"inspect\" command.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerState {
  /// String representation of the container state. Can be one of \"created\", \"running\", \"paused\", \"restarting\", \"removing\", \"exited\", or \"dead\".
  #[serde(default, rename = "Status")]
  pub status: ContainerStateStatusEnum,

  /// Whether this container is running.  Note that a running container can be _paused_. The `Running` and `Paused` booleans are not mutually exclusive:  When pausing a container (on Linux), the freezer cgroup is used to suspend all processes in the container. Freezing the process requires the process to be running. As a result, paused containers are both `Running` _and_ `Paused`.  Use the `Status` field instead to determine if a container's state is \"running\".
  #[serde(rename = "Running")]
  pub running: Option<bool>,

  /// Whether this container is paused.
  #[serde(rename = "Paused")]
  pub paused: Option<bool>,

  /// Whether this container is restarting.
  #[serde(rename = "Restarting")]
  pub restarting: Option<bool>,

  /// Whether a process within this container has been killed because it ran out of memory since the container was last started.
  #[serde(rename = "OOMKilled")]
  pub oom_killed: Option<bool>,

  #[serde(rename = "Dead")]
  pub dead: Option<bool>,

  /// The process ID of this container
  #[serde(rename = "Pid")]
  pub pid: Option<I64>,

  /// The last exit code of this container
  #[serde(rename = "ExitCode")]
  pub exit_code: Option<I64>,

  #[serde(rename = "Error")]
  pub error: Option<String>,

  /// The time when this container was last started.
  #[serde(rename = "StartedAt")]
  pub started_at: Option<String>,

  /// The time when this container last exited.
  #[serde(rename = "FinishedAt")]
  pub finished_at: Option<String>,

  #[serde(rename = "Health")]
  pub health: Option<ContainerHealth>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum ContainerStateStatusEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "created")]
  Created,
  #[serde(rename = "running")]
  Running,
  #[serde(rename = "paused")]
  Paused,
  #[serde(rename = "restarting")]
  Restarting,
  #[serde(rename = "exited")]
  Exited,
  #[serde(rename = "removing")]
  Removing,
  #[serde(rename = "dead")]
  Dead,
}

impl ::std::str::FromStr for ContainerStateStatusEnum {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "" => Ok(ContainerStateStatusEnum::Empty),
      "created" => Ok(ContainerStateStatusEnum::Created),
      "running" => Ok(ContainerStateStatusEnum::Running),
      "paused" => Ok(ContainerStateStatusEnum::Paused),
      "restarting" => Ok(ContainerStateStatusEnum::Restarting),
      "removing" => Ok(ContainerStateStatusEnum::Removing),
      "exited" => Ok(ContainerStateStatusEnum::Exited),
      "dead" => Ok(ContainerStateStatusEnum::Dead),
      x => Err(anyhow!("Invalid container state: {}", x)),
    }
  }
}

/// Health stores information about the container's healthcheck results.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerHealth {
  /// Status is one of `none`, `starting`, `healthy` or `unhealthy`  - \"none\"      Indicates there is no healthcheck - \"starting\"  Starting indicates that the container is not yet ready - \"healthy\"   Healthy indicates that the container is running correctly - \"unhealthy\" Unhealthy indicates that the container has a problem
  #[serde(default, rename = "Status")]
  pub status: HealthStatusEnum,

  /// FailingStreak is the number of consecutive failures
  #[serde(rename = "FailingStreak")]
  pub failing_streak: Option<I64>,

  /// Log contains the last few results (oldest first)
  #[serde(default, rename = "Log")]
  pub log: Vec<HealthcheckResult>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum HealthStatusEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "none")]
  None,
  #[serde(rename = "starting")]
  Starting,
  #[serde(rename = "healthy")]
  Healthy,
  #[serde(rename = "unhealthy")]
  Unhealthy,
}

/// HealthcheckResult stores information about a single run of a healthcheck probe
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct HealthcheckResult {
  /// Date and time at which this check started in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "Start")]
  pub start: Option<String>,

  /// Date and time at which this check ended in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "End")]
  pub end: Option<String>,

  /// ExitCode meanings:  - `0` healthy - `1` unhealthy - `2` reserved (considered unhealthy) - other values: error running probe
  #[serde(rename = "ExitCode")]
  pub exit_code: Option<I64>,

  /// Output from last check
  #[serde(rename = "Output")]
  pub output: Option<String>,
}

/// Container configuration that depends on the host we are running on
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct HostConfig {
  /// An integer value representing this container's relative CPU weight versus other containers.
  #[serde(rename = "CpuShares")]
  pub cpu_shares: Option<I64>,

  /// Memory limit in bytes.
  #[serde(rename = "Memory")]
  pub memory: Option<I64>,

  /// Path to `cgroups` under which the container's `cgroup` is created. If the path is not absolute, the path is considered to be relative to the `cgroups` path of the init process. Cgroups are created if they do not already exist.
  #[serde(rename = "CgroupParent")]
  pub cgroup_parent: Option<String>,

  /// Block IO weight (relative weight).
  #[serde(rename = "BlkioWeight")]
  pub blkio_weight: Option<u16>,

  /// Block IO weight (relative device weight) in the form:  ``` [{\"Path\": \"device_path\", \"Weight\": weight}] ```
  #[serde(default, rename = "BlkioWeightDevice")]
  pub blkio_weight_device: Vec<ResourcesBlkioWeightDevice>,

  /// Limit read rate (bytes per second) from a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ```
  #[serde(default, rename = "BlkioDeviceReadBps")]
  pub blkio_device_read_bps: Vec<ThrottleDevice>,

  /// Limit write rate (bytes per second) to a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ```
  #[serde(default, rename = "BlkioDeviceWriteBps")]
  pub blkio_device_write_bps: Vec<ThrottleDevice>,

  /// Limit read rate (IO per second) from a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ```
  #[serde(default, rename = "BlkioDeviceReadIOps")]
  pub blkio_device_read_iops: Vec<ThrottleDevice>,

  /// Limit write rate (IO per second) to a device, in the form:  ``` [{\"Path\": \"device_path\", \"Rate\": rate}] ```
  #[serde(default, rename = "BlkioDeviceWriteIOps")]
  pub blkio_device_write_iops: Vec<ThrottleDevice>,

  /// The length of a CPU period in microseconds.
  #[serde(rename = "CpuPeriod")]
  pub cpu_period: Option<I64>,

  /// Microseconds of CPU time that the container can get in a CPU period.
  #[serde(rename = "CpuQuota")]
  pub cpu_quota: Option<I64>,

  /// The length of a CPU real-time period in microseconds. Set to 0 to allocate no time allocated to real-time tasks.
  #[serde(rename = "CpuRealtimePeriod")]
  pub cpu_realtime_period: Option<I64>,

  /// The length of a CPU real-time runtime in microseconds. Set to 0 to allocate no time allocated to real-time tasks.
  #[serde(rename = "CpuRealtimeRuntime")]
  pub cpu_realtime_runtime: Option<I64>,

  /// CPUs in which to allow execution (e.g., `0-3`, `0,1`).
  #[serde(rename = "CpusetCpus")]
  pub cpuset_cpus: Option<String>,

  /// Memory nodes (MEMs) in which to allow execution (0-3, 0,1). Only effective on NUMA systems.
  #[serde(rename = "CpusetMems")]
  pub cpuset_mems: Option<String>,

  /// A list of devices to add to the container.
  #[serde(default, rename = "Devices")]
  pub devices: Vec<DeviceMapping>,

  /// a list of cgroup rules to apply to the container
  #[serde(default, rename = "DeviceCgroupRules")]
  pub device_cgroup_rules: Vec<String>,

  /// A list of requests for devices to be sent to device drivers.
  #[serde(default, rename = "DeviceRequests")]
  pub device_requests: Vec<DeviceRequest>,

  /// Hard limit for kernel TCP buffer memory (in bytes). Depending on the OCI runtime in use, this option may be ignored. It is no longer supported by the default (runc) runtime.  This field is omitted when empty.
  #[serde(rename = "KernelMemoryTCP")]
  pub kernel_memory_tcp: Option<I64>,

  /// Memory soft limit in bytes.
  #[serde(rename = "MemoryReservation")]
  pub memory_reservation: Option<I64>,

  /// Total memory limit (memory + swap). Set as `-1` to enable unlimited swap.
  #[serde(rename = "MemorySwap")]
  pub memory_swap: Option<I64>,

  /// Tune a container's memory swappiness behavior. Accepts an integer between 0 and 100.
  #[serde(rename = "MemorySwappiness")]
  pub memory_swappiness: Option<I64>,

  /// CPU quota in units of 10<sup>-9</sup> CPUs.
  #[serde(rename = "NanoCpus")]
  pub nano_cpus: Option<I64>,

  /// Disable OOM Killer for the container.
  #[serde(rename = "OomKillDisable")]
  pub oom_kill_disable: Option<bool>,

  /// Run an init inside the container that forwards signals and reaps processes. This field is omitted if empty, and the default (as configured on the daemon) is used.
  #[serde(rename = "Init")]
  pub init: Option<bool>,

  /// Tune a container's PIDs limit. Set `0` or `-1` for unlimited, or `null` to not change.
  #[serde(rename = "PidsLimit")]
  pub pids_limit: Option<I64>,

  /// A list of resource limits to set in the container. For example:  ``` {\"Name\": \"nofile\", \"Soft\": 1024, \"Hard\": 2048} ```
  #[serde(default, rename = "Ulimits")]
  pub ulimits: Vec<ResourcesUlimits>,

  /// The number of usable CPUs (Windows only).  On Windows Server containers, the processor resource controls are mutually exclusive. The order of precedence is `CPUCount` first, then `CPUShares`, and `CPUPercent` last.
  #[serde(rename = "CpuCount")]
  pub cpu_count: Option<I64>,

  /// The usable percentage of the available CPUs (Windows only).  On Windows Server containers, the processor resource controls are mutually exclusive. The order of precedence is `CPUCount` first, then `CPUShares`, and `CPUPercent` last.
  #[serde(rename = "CpuPercent")]
  pub cpu_percent: Option<I64>,

  /// Maximum IOps for the container system drive (Windows only)
  #[serde(rename = "IOMaximumIOps")]
  pub io_maximum_iops: Option<I64>,

  /// Maximum IO in bytes per second for the container system drive (Windows only).
  #[serde(rename = "IOMaximumBandwidth")]
  pub io_maximum_bandwidth: Option<I64>,

  /// A list of volume bindings for this container. Each volume binding is a string in one of these forms:  - `host-src:container-dest[:options]` to bind-mount a host path   into the container. Both `host-src`, and `container-dest` must   be an _absolute_ path. - `volume-name:container-dest[:options]` to bind-mount a volume   managed by a volume driver into the container. `container-dest`   must be an _absolute_ path.  `options` is an optional, comma-delimited list of:  - `nocopy` disables automatic copying of data from the container   path to the volume. The `nocopy` flag only applies to named volumes. - `[ro|rw]` mounts a volume read-only or read-write, respectively.   If omitted or set to `rw`, volumes are mounted read-write. - `[z|Z]` applies SELinux labels to allow or deny multiple containers   to read and write to the same volume.     - `z`: a _shared_ content label is applied to the content. This       label indicates that multiple containers can share the volume       content, for both reading and writing.     - `Z`: a _private unshared_ label is applied to the content.       This label indicates that only the current container can use       a private volume. Labeling systems such as SELinux require       proper labels to be placed on volume content that is mounted       into a container. Without a label, the security system can       prevent a container's processes from using the content. By       default, the labels set by the host operating system are not       modified. - `[[r]shared|[r]slave|[r]private]` specifies mount   [propagation behavior](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt).   This only applies to bind-mounted volumes, not internal volumes   or named volumes. Mount propagation requires the source mount   point (the location where the source directory is mounted in the   host operating system) to have the correct propagation properties.   For shared volumes, the source mount point must be set to `shared`.   For slave volumes, the mount must be set to either `shared` or   `slave`.
  #[serde(default, rename = "Binds")]
  pub binds: Vec<String>,

  /// Path to a file where the container ID is written
  #[serde(rename = "ContainerIDFile")]
  pub container_id_file: Option<String>,

  #[serde(rename = "LogConfig")]
  pub log_config: Option<HostConfigLogConfig>,

  /// Network mode to use for this container. Supported standard values are: `bridge`, `host`, `none`, and `container:<name|id>`. Any other value is taken as a custom network's name to which this container should connect to.
  #[serde(rename = "NetworkMode")]
  pub network_mode: Option<String>,

  #[serde(default, rename = "PortBindings")]
  pub port_bindings: HashMap<String, Vec<PortBinding>>,

  #[serde(rename = "RestartPolicy")]
  pub restart_policy: Option<RestartPolicy>,

  /// Automatically remove the container when the container's process exits. This has no effect if `RestartPolicy` is set.
  #[serde(rename = "AutoRemove")]
  pub auto_remove: Option<bool>,

  /// Driver that this container uses to mount volumes.
  #[serde(rename = "VolumeDriver")]
  pub volume_driver: Option<String>,

  /// A list of volumes to inherit from another container, specified in the form `<container name>[:<ro|rw>]`.
  #[serde(default, rename = "VolumesFrom")]
  pub volumes_from: Vec<String>,

  /// Specification for mounts to be added to the container.
  #[serde(default, rename = "Mounts")]
  pub mounts: Vec<ContainerMount>,

  /// Initial console size, as an `[height, width]` array.
  #[serde(default, rename = "ConsoleSize")]
  pub console_size: Vec<i32>,

  /// Arbitrary non-identifying metadata attached to container and provided to the runtime when the container is started.
  #[serde(default, rename = "Annotations")]
  pub annotations: HashMap<String, String>,

  /// A list of kernel capabilities to add to the container. Conflicts with option 'Capabilities'.
  #[serde(default, rename = "CapAdd")]
  pub cap_add: Vec<String>,

  /// A list of kernel capabilities to drop from the container. Conflicts with option 'Capabilities'.
  #[serde(default, rename = "CapDrop")]
  pub cap_drop: Vec<String>,

  /// cgroup namespace mode for the container. Possible values are:  - `\"private\"`: the container runs in its own private cgroup namespace - `\"host\"`: use the host system's cgroup namespace  If not specified, the daemon default is used, which can either be `\"private\"` or `\"host\"`, depending on daemon version, kernel support and configuration.
  #[serde(rename = "CgroupnsMode")]
  pub cgroupns_mode: Option<HostConfigCgroupnsModeEnum>,

  /// A list of DNS servers for the container to use.
  #[serde(default, rename = "Dns")]
  pub dns: Vec<String>,

  /// A list of DNS options.
  #[serde(default, rename = "DnsOptions")]
  pub dns_options: Vec<String>,

  /// A list of DNS search domains.
  #[serde(default, rename = "DnsSearch")]
  pub dns_search: Vec<String>,

  /// A list of hostnames/IP mappings to add to the container's `/etc/hosts` file. Specified in the form `[\"hostname:IP\"]`.
  #[serde(default, rename = "ExtraHosts")]
  pub extra_hosts: Vec<String>,

  /// A list of additional groups that the container process will run as.
  #[serde(default, rename = "GroupAdd")]
  pub group_add: Vec<String>,

  /// IPC sharing mode for the container. Possible values are:  - `\"none\"`: own private IPC namespace, with /dev/shm not mounted - `\"private\"`: own private IPC namespace - `\"shareable\"`: own private IPC namespace, with a possibility to share it with other containers - `\"container:<name|id>\"`: join another (shareable) container's IPC namespace - `\"host\"`: use the host system's IPC namespace  If not specified, daemon default is used, which can either be `\"private\"` or `\"shareable\"`, depending on daemon version and configuration.
  #[serde(rename = "IpcMode")]
  pub ipc_mode: Option<String>,

  /// Cgroup to use for the container.
  #[serde(rename = "Cgroup")]
  pub cgroup: Option<String>,

  /// A list of links for the container in the form `container_name:alias`.
  #[serde(default, rename = "Links")]
  pub links: Vec<String>,

  /// An integer value containing the score given to the container in order to tune OOM killer preferences.
  #[serde(rename = "OomScoreAdj")]
  pub oom_score_adj: Option<I64>,

  /// Set the PID (Process) Namespace mode for the container. It can be either:  - `\"container:<name|id>\"`: joins another container's PID namespace - `\"host\"`: use the host's PID namespace inside the container
  #[serde(rename = "PidMode")]
  pub pid_mode: Option<String>,

  /// Gives the container full access to the host.
  #[serde(rename = "Privileged")]
  pub privileged: Option<bool>,

  /// Allocates an ephemeral host port for all of a container's exposed ports.  Ports are de-allocated when the container stops and allocated when the container starts. The allocated port might be changed when restarting the container.  The port is selected from the ephemeral port range that depends on the kernel. For example, on Linux the range is defined by `/proc/sys/net/ipv4/ip_local_port_range`.
  #[serde(rename = "PublishAllPorts")]
  pub publish_all_ports: Option<bool>,

  /// Mount the container's root filesystem as read only.
  #[serde(rename = "ReadonlyRootfs")]
  pub readonly_rootfs: Option<bool>,

  /// A list of string values to customize labels for MLS systems, such as SELinux.
  #[serde(default, rename = "SecurityOpt")]
  pub security_opt: Vec<String>,

  /// Storage driver options for this container, in the form `{\"size\": \"120G\"}`.
  #[serde(default, rename = "StorageOpt")]
  pub storage_opt: HashMap<String, String>,

  /// A map of container directories which should be replaced by tmpfs mounts, and their corresponding mount options. For example:  ``` { \"/run\": \"rw,noexec,nosuid,size=65536k\" } ```
  #[serde(default, rename = "Tmpfs")]
  pub tmpfs: HashMap<String, String>,

  /// UTS namespace to use for the container.
  #[serde(rename = "UTSMode")]
  pub uts_mode: Option<String>,

  /// Sets the usernamespace mode for the container when usernamespace remapping option is enabled.
  #[serde(rename = "UsernsMode")]
  pub userns_mode: Option<String>,

  /// Size of `/dev/shm` in bytes. If omitted, the system uses 64MB.
  #[serde(rename = "ShmSize")]
  pub shm_size: Option<I64>,

  /// A list of kernel parameters (sysctls) to set in the container. For example:  ``` {\"net.ipv4.ip_forward\": \"1\"} ```
  #[serde(default, rename = "Sysctls")]
  pub sysctls: HashMap<String, String>,

  /// Runtime to use with this container.
  #[serde(rename = "Runtime")]
  pub runtime: Option<String>,

  /// Isolation technology of the container. (Windows only)
  #[serde(default, rename = "Isolation")]
  pub isolation: HostConfigIsolationEnum,

  /// The list of paths to be masked inside the container (this overrides the default set of paths).
  #[serde(default, rename = "MaskedPaths")]
  pub masked_paths: Vec<String>,

  /// The list of paths to be set as read-only inside the container (this overrides the default set of paths).
  #[serde(default, rename = "ReadonlyPaths")]
  pub readonly_paths: Vec<String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ResourcesBlkioWeightDevice {
  #[serde(rename = "Path")]
  pub path: Option<String>,

  #[serde(rename = "Weight")]
  pub weight: Option<Usize>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ThrottleDevice {
  /// Device path
  #[serde(rename = "Path")]
  pub path: Option<String>,

  /// Rate
  #[serde(rename = "Rate")]
  pub rate: Option<I64>,
}

/// A device mapping between the host and container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DeviceMapping {
  #[serde(rename = "PathOnHost")]
  pub path_on_host: Option<String>,

  #[serde(rename = "PathInContainer")]
  pub path_in_container: Option<String>,

  #[serde(rename = "CgroupPermissions")]
  pub cgroup_permissions: Option<String>,
}

/// A request for devices to be sent to device drivers
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DeviceRequest {
  #[serde(rename = "Driver")]
  pub driver: Option<String>,

  #[serde(rename = "Count")]
  pub count: Option<I64>,

  #[serde(default, rename = "DeviceIDs")]
  pub device_ids: Vec<String>,

  /// A list of capabilities; an OR list of AND lists of capabilities.
  #[serde(default, rename = "Capabilities")]
  pub capabilities: Vec<Vec<String>>,

  /// Driver-specific options, specified as a key/value pairs. These options are passed directly to the driver.
  #[serde(default, rename = "Options")]
  pub options: HashMap<String, String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ResourcesUlimits {
  /// Name of ulimit
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// Soft limit
  #[serde(rename = "Soft")]
  pub soft: Option<I64>,

  /// Hard limit
  #[serde(rename = "Hard")]
  pub hard: Option<I64>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum HostConfigIsolationEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "default")]
  Default,
  #[serde(rename = "process")]
  Process,
  #[serde(rename = "hyperv")]
  Hyperv,
}

/// The logging configuration for this container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct HostConfigLogConfig {
  #[serde(rename = "Type")]
  pub typ: Option<String>,

  #[serde(default, rename = "Config")]
  pub config: HashMap<String, String>,
}

/// The behavior to apply when the container exits. The default is not to restart.  An ever increasing delay (double the previous delay, starting at 100ms) is added before each restart to prevent flooding the server.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct RestartPolicy {
  /// - Empty string means not to restart - `no` Do not automatically restart - `always` Always restart - `unless-stopped` Restart always except when the user has manually stopped the container - `on-failure` Restart only when the container exit code is non-zero
  #[serde(default, rename = "Name")]
  pub name: RestartPolicyNameEnum,

  /// If `on-failure` is used, the number of times to retry before giving up.
  #[serde(rename = "MaximumRetryCount")]
  pub maximum_retry_count: Option<I64>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum RestartPolicyNameEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "no")]
  No,
  #[serde(rename = "always")]
  Always,
  #[serde(rename = "unless-stopped")]
  UnlessStopped,
  #[serde(rename = "on-failure")]
  OnFailure,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerMount {
  /// Container path.
  #[serde(rename = "Target")]
  pub target: Option<String>,

  /// Mount source (e.g. a volume name, a host path).
  #[serde(rename = "Source")]
  pub source: Option<String>,

  /// The mount type. Available types:  - `bind` Mounts a file or directory from the host into the container. Must exist prior to creating the container. - `volume` Creates a volume with the given name and options (or uses a pre-existing volume with the same name and options). These are **not** removed when the container is removed. - `tmpfs` Create a tmpfs with the given options. The mount source cannot be specified for tmpfs. - `npipe` Mounts a named pipe from the host into the container. Must exist prior to creating the container. - `cluster` a Swarm cluster volume
  #[serde(default, rename = "Type")]
  pub typ: MountTypeEnum,

  /// Whether the mount should be read-only.
  #[serde(rename = "ReadOnly")]
  pub read_only: Option<bool>,

  /// The consistency requirement for the mount: `default`, `consistent`, `cached`, or `delegated`.
  #[serde(rename = "Consistency")]
  pub consistency: Option<String>,

  #[serde(rename = "BindOptions")]
  pub bind_options: Option<MountBindOptions>,

  #[serde(rename = "VolumeOptions")]
  pub volume_options: Option<MountVolumeOptions>,

  #[serde(rename = "TmpfsOptions")]
  pub tmpfs_options: Option<MountTmpfsOptions>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum MountTypeEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "bind")]
  Bind,
  #[serde(rename = "volume")]
  Volume,
  #[serde(rename = "image")]
  Image,
  #[serde(rename = "tmpfs")]
  Tmpfs,
  #[serde(rename = "npipe")]
  Npipe,
  #[serde(rename = "cluster")]
  Cluster,
}

/// Optional configuration for the `bind` type.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct MountBindOptions {
  /// A propagation mode with the value `[r]private`, `[r]shared`, or `[r]slave`.
  #[serde(default, rename = "Propagation")]
  pub propagation: MountBindOptionsPropagationEnum,

  /// Disable recursive bind mount.
  #[serde(rename = "NonRecursive")]
  pub non_recursive: Option<bool>,

  /// Create mount point on host if missing
  #[serde(rename = "CreateMountpoint")]
  pub create_mountpoint: Option<bool>,

  /// Make the mount non-recursively read-only, but still leave the mount recursive (unless NonRecursive is set to `true` in conjunction).  Addded in v1.44, before that version all read-only mounts were non-recursive by default. To match the previous behaviour this will default to `true` for clients on versions prior to v1.44.
  #[serde(rename = "ReadOnlyNonRecursive")]
  pub read_only_non_recursive: Option<bool>,

  /// Raise an error if the mount cannot be made recursively read-only.
  #[serde(rename = "ReadOnlyForceRecursive")]
  pub read_only_force_recursive: Option<bool>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum MountBindOptionsPropagationEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "private")]
  Private,
  #[serde(rename = "rprivate")]
  Rprivate,
  #[serde(rename = "shared")]
  Shared,
  #[serde(rename = "rshared")]
  Rshared,
  #[serde(rename = "slave")]
  Slave,
  #[serde(rename = "rslave")]
  Rslave,
}

/// Optional configuration for the `volume` type.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct MountVolumeOptions {
  /// Populate volume with data from the target.
  #[serde(rename = "NoCopy")]
  pub no_copy: Option<bool>,

  /// User-defined key/value metadata.
  #[serde(default, rename = "Labels")]
  pub labels: HashMap<String, String>,

  #[serde(rename = "DriverConfig")]
  pub driver_config: Option<MountVolumeOptionsDriverConfig>,

  /// Source path inside the volume. Must be relative without any back traversals.
  #[serde(rename = "Subpath")]
  pub subpath: Option<String>,
}

/// Map of driver specific options
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct MountVolumeOptionsDriverConfig {
  /// Name of the driver to use to create the volume.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// key/value map of driver specific options.
  #[serde(default, rename = "Options")]
  pub options: HashMap<String, String>,
}

/// Optional configuration for the `tmpfs` type.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct MountTmpfsOptions {
  /// The size for the tmpfs mount in bytes.
  #[serde(rename = "SizeBytes")]
  pub size_bytes: Option<I64>,

  /// The permission mode for the tmpfs mount in an integer.
  #[serde(rename = "Mode")]
  pub mode: Option<I64>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
pub enum HostConfigCgroupnsModeEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "private")]
  Private,
  #[serde(rename = "host")]
  Host,
}

/// MountPoint represents a mount point configuration inside the container. This is used for reporting the mountpoints in use by a container.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct MountPoint {
  /// The mount type:  - `bind` a mount of a file or directory from the host into the container. - `volume` a docker volume with the given `Name`. - `tmpfs` a `tmpfs`. - `npipe` a named pipe from the host into the container. - `cluster` a Swarm cluster volume
  #[serde(default, rename = "Type")]
  pub typ: MountTypeEnum,

  /// Name is the name reference to the underlying data defined by `Source` e.g., the volume name.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// Source location of the mount.  For volumes, this contains the storage location of the volume (within `/var/lib/docker/volumes/`). For bind-mounts, and `npipe`, this contains the source (host) part of the bind-mount. For `tmpfs` mount points, this field is empty.
  #[serde(rename = "Source")]
  pub source: Option<String>,

  /// Destination is the path relative to the container root (`/`) where the `Source` is mounted inside the container.
  #[serde(rename = "Destination")]
  pub destination: Option<String>,

  /// Driver is the volume driver used to create the volume (if it is a volume).
  #[serde(rename = "Driver")]
  pub driver: Option<String>,

  /// Mode is a comma separated list of options supplied by the user when creating the bind/volume mount.  The default is platform-specific (`\"z\"` on Linux, empty on Windows).
  #[serde(rename = "Mode")]
  pub mode: Option<String>,

  /// Whether the mount is mounted writable (read-write).
  #[serde(rename = "RW")]
  pub rw: Option<bool>,

  /// Propagation describes how mounts are propagated from the host into the mount point, and vice-versa. Refer to the [Linux kernel documentation](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt) for details. This field is not used on Windows.
  #[serde(rename = "Propagation")]
  pub propagation: Option<String>,
}

/// NetworkSettings exposes the network settings in the API
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NetworkSettings {
  /// Name of the default bridge interface when dockerd's --bridge flag is set.
  #[serde(rename = "Bridge")]
  pub bridge: Option<String>,

  /// SandboxID uniquely represents a container's network stack.
  #[serde(rename = "SandboxID")]
  pub sandbox_id: Option<String>,

  #[serde(default, rename = "Ports")]
  pub ports: HashMap<String, Vec<PortBinding>>,

  /// SandboxKey is the full path of the netns handle
  #[serde(rename = "SandboxKey")]
  pub sandbox_key: Option<String>,

  /// Information about all networks that the container is connected to.
  #[serde(default, rename = "Networks")]
  pub networks: HashMap<String, EndpointSettings>,
}

/// Configuration for a network endpoint.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EndpointSettings {
  #[serde(rename = "IPAMConfig")]
  pub ipam_config: Option<EndpointIpamConfig>,

  #[serde(default, rename = "Links")]
  pub links: Vec<String>,

  /// MAC address for the endpoint on this network. The network driver might ignore this parameter.
  #[serde(rename = "MacAddress")]
  pub mac_address: Option<String>,

  #[serde(default, rename = "Aliases")]
  pub aliases: Vec<String>,

  /// Unique ID of the network.
  #[serde(rename = "NetworkID")]
  pub network_id: Option<String>,

  /// Unique ID for the service endpoint in a Sandbox.
  #[serde(rename = "EndpointID")]
  pub endpoint_id: Option<String>,

  /// Gateway address for this network.
  #[serde(rename = "Gateway")]
  pub gateway: Option<String>,

  /// IPv4 address.
  #[serde(rename = "IPAddress")]
  pub ip_address: Option<String>,

  /// Mask length of the IPv4 address.
  #[serde(rename = "IPPrefixLen")]
  pub ip_prefix_len: Option<I64>,

  /// IPv6 gateway address.
  #[serde(rename = "IPv6Gateway")]
  pub ipv6_gateway: Option<String>,

  /// Global IPv6 address.
  #[serde(rename = "GlobalIPv6Address")]
  pub global_ipv6_address: Option<String>,

  /// Mask length of the global IPv6 address.
  #[serde(rename = "GlobalIPv6PrefixLen")]
  pub global_ipv6_prefix_len: Option<I64>,

  /// DriverOpts is a mapping of driver options and values. These options are passed directly to the driver and are driver specific.
  #[serde(default, rename = "DriverOpts")]
  pub driver_opts: HashMap<String, String>,

  /// List of all DNS names an endpoint has on a specific network. This list is based on the container name, network aliases, container short ID, and hostname.  These DNS names are non-fully qualified but can contain several dots. You can get fully qualified DNS names by appending `.<network-name>`. For instance, if container name is `my.ctr` and the network is named `testnet`, `DNSNames` will contain `my.ctr` and the FQDN will be `my.ctr.testnet`.
  #[serde(default, rename = "DNSNames")]
  pub dns_names: Vec<String>,
}

/// EndpointIPAMConfig represents an endpoint's IPAM configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EndpointIpamConfig {
  #[serde(rename = "IPv4Address")]
  pub ipv4_address: Option<String>,

  #[serde(rename = "IPv6Address")]
  pub ipv6_address: Option<String>,

  #[serde(default, rename = "LinkLocalIPs")]
  pub link_local_ips: Vec<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
  #[serde(alias = "Name")]
  pub name: String,
  #[serde(alias = "CPUPerc")]
  pub cpu_perc: String,
  #[serde(alias = "MemPerc")]
  pub mem_perc: String,
  #[serde(alias = "MemUsage")]
  pub mem_usage: String,
  #[serde(alias = "NetIO")]
  pub net_io: String,
  #[serde(alias = "BlockIO")]
  pub block_io: String,
  #[serde(alias = "PIDs")]
  pub pids: String,
}
