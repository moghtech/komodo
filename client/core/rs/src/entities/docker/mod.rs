use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  docker::{
    config::SwarmConfigListItem, container::ContainerListItem,
    image::ImageListItem, network::NetworkListItem,
    node::SwarmNodeListItem, secret::SwarmSecretListItem,
    service::SwarmServiceListItem, stack::SwarmStackListItem,
    task::SwarmTaskListItem, volume::VolumeListItem,
  },
  stack::ComposeProject,
};

use super::{I64, U64};

pub mod config;
pub mod container;
pub mod image;
pub mod network;
pub mod node;
pub mod secret;
pub mod service;
pub mod stack;
pub mod stats;
pub mod swarm;
pub mod task;
pub mod volume;

/// Swarm lists available from a manager node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmLists {
  pub nodes: Vec<SwarmNodeListItem>,
  pub stacks: Vec<SwarmStackListItem>,
  pub services: Vec<SwarmServiceListItem>,
  pub tasks: Vec<SwarmTaskListItem>,
  pub configs: Vec<SwarmConfigListItem>,
  pub secrets: Vec<SwarmSecretListItem>,
}

/// Standard docker lists available from a Server.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct DockerLists {
  pub containers: Vec<ContainerListItem>,
  pub networks: Vec<NetworkListItem>,
  pub images: Vec<ImageListItem>,
  pub volumes: Vec<VolumeListItem>,
  pub projects: Vec<ComposeProject>,
}

/// PortBinding represents a binding between a host IP address and a host port.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct PortBinding {
  /// Host IP address that the container's port is mapped to.
  #[serde(rename = "HostIp")]
  pub host_ip: Option<String>,

  /// Host port number that the container's port is mapped to.
  #[serde(rename = "HostPort")]
  pub host_port: Option<String>,
}

/// Information about the storage driver used to store the container's and image's filesystem.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct GraphDriverData {
  /// Name of the storage driver.
  #[serde(default, rename = "Name")]
  pub name: String,
  /// Low-level storage metadata, provided as key/value pairs.  This information is driver-specific, and depends on the storage-driver in use, and should be used for informational purposes only.
  #[serde(default, rename = "Data")]
  pub data: HashMap<String, String>,
}

/// Configuration for a container that is portable between hosts.  When used as `ContainerConfig` field in an image, `ContainerConfig` is an optional field containing the configuration of the container that was last committed when creating the image.  Previous versions of Docker builder used this field to store build cache, and it is not in active use anymore.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ContainerConfig {
  /// The hostname to use for the container, as a valid RFC 1123 hostname.
  #[serde(rename = "Hostname")]
  pub hostname: Option<String>,

  /// The domain name to use for the container.
  #[serde(rename = "Domainname")]
  pub domainname: Option<String>,

  /// The user that commands are run as inside the container.
  #[serde(rename = "User")]
  pub user: Option<String>,

  /// Whether to attach to `stdin`.
  #[serde(rename = "AttachStdin")]
  pub attach_stdin: Option<bool>,

  /// Whether to attach to `stdout`.
  #[serde(rename = "AttachStdout")]
  pub attach_stdout: Option<bool>,

  /// Whether to attach to `stderr`.
  #[serde(rename = "AttachStderr")]
  pub attach_stderr: Option<bool>,

  /// An object mapping ports to an empty object in the form:  `{\"<port>/<tcp|udp|sctp>\": {}}`
  #[serde(default, rename = "ExposedPorts")]
  pub exposed_ports: HashMap<String, HashMap<String, ()>>,

  /// Attach standard streams to a TTY, including `stdin` if it is not closed.
  #[serde(rename = "Tty")]
  pub tty: Option<bool>,

  /// Open `stdin`
  #[serde(rename = "OpenStdin")]
  pub open_stdin: Option<bool>,

  /// Close `stdin` after one attached client disconnects
  #[serde(rename = "StdinOnce")]
  pub stdin_once: Option<bool>,

  /// A list of environment variables to set inside the container in the form `[\"VAR=value\", ...]`. A variable without `=` is removed from the environment, rather than to have an empty value.
  #[serde(default, rename = "Env")]
  pub env: Vec<String>,

  /// Command to run specified as a string or an array of strings.
  #[serde(default, rename = "Cmd")]
  pub cmd: Vec<String>,

  #[serde(rename = "Healthcheck")]
  pub healthcheck: Option<HealthConfig>,

  /// Command is already escaped (Windows only)
  #[serde(rename = "ArgsEscaped")]
  pub args_escaped: Option<bool>,

  /// The name (or reference) of the image to use when creating the container, or which was used when the container was created.
  #[serde(rename = "Image")]
  pub image: Option<String>,

  /// An object mapping mount point paths inside the container to empty objects.
  #[serde(default, rename = "Volumes")]
  pub volumes: HashMap<String, HashMap<String, ()>>,

  /// The working directory for commands to run in.
  #[serde(rename = "WorkingDir")]
  pub working_dir: Option<String>,

  /// The entry point for the container as a string or an array of strings.  If the array consists of exactly one empty string (`[\"\"]`) then the entry point is reset to system default (i.e., the entry point used by docker when there is no `ENTRYPOINT` instruction in the `Dockerfile`).
  #[serde(default, rename = "Entrypoint")]
  pub entrypoint: Vec<String>,

  /// Disable networking for the container.
  #[serde(rename = "NetworkDisabled")]
  pub network_disabled: Option<bool>,

  /// MAC address of the container.  Deprecated: this field is deprecated in API v1.44 and up. Use EndpointSettings.MacAddress instead.
  #[serde(rename = "MacAddress")]
  pub mac_address: Option<String>,

  /// `ONBUILD` metadata that were defined in the image's `Dockerfile`.
  #[serde(default, rename = "OnBuild")]
  pub on_build: Vec<String>,

  /// User-defined key/value metadata.
  #[serde(default, rename = "Labels")]
  pub labels: HashMap<String, String>,

  /// Signal to stop a container as a string or unsigned integer.
  #[serde(rename = "StopSignal")]
  pub stop_signal: Option<String>,

  /// Timeout to stop a container in seconds.
  #[serde(rename = "StopTimeout")]
  pub stop_timeout: Option<I64>,

  /// Shell for when `RUN`, `CMD`, and `ENTRYPOINT` uses a shell.
  #[serde(default, rename = "Shell")]
  pub shell: Vec<String>,
}

/// A test to perform to check that the container is healthy.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct HealthConfig {
  /// The test to perform. Possible values are:  - `[]` inherit healthcheck from image or parent image - `[\"NONE\"]` disable healthcheck - `[\"CMD\", args...]` exec arguments directly - `[\"CMD-SHELL\", command]` run command with system's default shell
  #[serde(default, rename = "Test")]
  pub test: Vec<String>,

  /// The time to wait between checks in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "Interval")]
  pub interval: Option<I64>,

  /// The time to wait before considering the check to have hung. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "Timeout")]
  pub timeout: Option<I64>,

  /// The number of consecutive failures needed to consider a container as unhealthy. 0 means inherit.
  #[serde(rename = "Retries")]
  pub retries: Option<I64>,

  /// Start period for the container to initialize before starting health-retries countdown in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "StartPeriod")]
  pub start_period: Option<I64>,

  /// The time to wait between checks in nanoseconds during the start period. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "StartInterval")]
  pub start_interval: Option<I64>,
}

/// The version number of the object such as node, service, etc. This is needed to avoid conflicting writes. The client must send the version number along with the modified specification when updating these objects.  This approach ensures safe concurrency and determinism in that the change on the object may not be applied if the version number has changed from the last read. In other words, if two update requests specify the same base version, only one of the requests can succeed. As a result, two separate update requests that happen at the same time will not unintentionally overwrite each other.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ObjectVersion {
  #[serde(rename = "Index")]
  pub index: Option<U64>,
}

/// Driver represents a driver (network, logging, secrets).
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Driver {
  /// Name of the driver.
  #[serde(rename = "Name")]
  pub name: String,

  /// Key/value map of driver-specific options.
  #[serde(rename = "Options")]
  pub options: Option<HashMap<String, String>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Mount {
  /// Container path.
  #[serde(rename = "Target")]
  pub target: Option<String>,

  /// Mount source (e.g. a volume name, a host path).
  #[serde(rename = "Source")]
  pub source: Option<String>,

  /// The mount type. Available types:
  ///   - `bind` Mounts a file or directory from the host into the container. Must exist prior to creating the container.
  ///   - `volume` Creates a volume with the given name and options (or uses a pre-existing volume with the same name and options). These are **not** removed when the container is removed.
  ///   - `tmpfs` Create a tmpfs with the given options. The mount source cannot be specified for tmpfs. - `npipe` Mounts a named pipe from the host into the container. Must exist prior to creating the container.
  ///   - `cluster` a Swarm cluster volume
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
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ResourceObject {
  #[serde(rename = "NanoCPUs")]
  pub nano_cpus: Option<I64>,

  #[serde(rename = "MemoryBytes")]
  pub memory_bytes: Option<I64>,

  #[serde(rename = "GenericResources")]
  pub generic_resources: Option<GenericResources>,
}

/// User-defined resources can be either Integer resources (e.g, `SSD=3`) or String resources (e.g, `GPU=UUID1`).
#[typeshare]
pub type GenericResources = Vec<GenericResourcesInner>;

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct GenericResourcesInner {
  #[serde(rename = "NamedResourceSpec")]
  pub named_resource_spec:
    Option<GenericResourcesInnerNamedResourceSpec>,

  #[serde(rename = "DiscreteResourceSpec")]
  pub discrete_resource_spec:
    Option<GenericResourcesInnerDiscreteResourceSpec>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct GenericResourcesInnerDiscreteResourceSpec {
  #[serde(rename = "Kind")]
  pub kind: Option<String>,

  #[serde(rename = "Value")]
  pub value: Option<I64>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct GenericResourcesInnerNamedResourceSpec {
  #[serde(rename = "Kind")]
  pub kind: Option<String>,

  #[serde(rename = "Value")]
  pub value: Option<String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Platform {
  /// Architecture represents the hardware architecture (for example, `x86_64`).
  #[serde(rename = "Architecture")]
  pub architecture: Option<String>,

  /// OS represents the Operating System (for example, `linux` or `windows`).
  #[serde(rename = "OS")]
  pub os: Option<String>,
}

/// Specifies how a service should be attached to a particular network.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NetworkAttachmentConfig {
  /// The target network for attachment. Must be a network name or ID.
  #[serde(rename = "Target")]
  pub target: Option<String>,

  /// Discoverable alternate names for the service on this network.
  #[serde(rename = "Aliases")]
  pub aliases: Option<Vec<String>>,

  /// Driver attachment options for the network target.
  #[serde(rename = "DriverOpts")]
  pub driver_opts: Option<HashMap<String, String>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EndpointPortConfig {
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "Protocol")]
  pub protocol: Option<EndpointPortConfigProtocolEnum>,

  /// The port inside the container.
  #[serde(rename = "TargetPort")]
  pub target_port: Option<I64>,

  /// The port on the swarm hosts.
  #[serde(rename = "PublishedPort")]
  pub published_port: Option<I64>,

  /// The mode in which port is published.  <p><br /></p>  - \"ingress\" makes the target port accessible on every node,   regardless of whether there is a task for the service running on   that node or not. - \"host\" bypasses the routing mesh and publish the port directly on   the swarm node where that service is running.
  #[serde(rename = "PublishMode")]
  pub publish_mode: Option<EndpointPortConfigPublishModeEnum>,
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
pub enum EndpointPortConfigProtocolEnum {
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
pub enum EndpointPortConfigPublishModeEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "ingress")]
  INGRESS,
  #[serde(rename = "host")]
  HOST,
}

/// Information about the issuer of leaf TLS certificates and the trusted root CA certificate.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct TlsInfo {
  /// The root CA certificate(s) that are used to validate leaf TLS certificates.
  #[serde(rename = "TrustRoot")]
  pub trust_root: Option<String>,

  /// The base64-url-safe-encoded raw subject bytes of the issuer.
  #[serde(rename = "CertIssuerSubject")]
  pub cert_issuer_subject: Option<String>,

  /// The base64-url-safe-encoded raw public key bytes of the issuer.
  #[serde(rename = "CertIssuerPublicKey")]
  pub cert_issuer_public_key: Option<String>,
}
