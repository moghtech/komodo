use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::I64;

pub mod container;
pub mod image;
pub mod network;
pub mod stats;
pub mod volume;

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
