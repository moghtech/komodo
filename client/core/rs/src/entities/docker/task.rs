use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::*;

/// Swarm task list item.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmTaskListItem {
  /// The ID of the task.
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Name of the task.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// The ID of the node that this task is on.
  #[serde(rename = "NodeID")]
  pub node_id: Option<String>,

  /// The ID of the service this task is part of.
  #[serde(rename = "ServiceID")]
  pub service_id: Option<String>,

  /// The ID of container associated with this task.
  #[serde(rename = "ContainerID")]
  pub container_id: Option<String>,

  #[serde(rename = "State")]
  pub state: Option<TaskState>,

  #[serde(rename = "DesiredState")]
  pub desired_state: Option<TaskState>,

  /// Attached config names
  #[serde(rename = "Configs")]
  pub configs: Vec<String>,

  /// Attached secret names
  #[serde(rename = "Secrets")]
  pub secrets: Vec<String>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,
}

/// Swarm task details.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmTask {
  /// The ID of the task.
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  /// Name of the task.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  #[serde(rename = "Spec")]
  pub spec: Option<TaskSpec>,

  /// The ID of the service this task is part of.
  #[serde(rename = "ServiceID")]
  pub service_id: Option<String>,

  #[serde(rename = "Slot")]
  pub slot: Option<I64>,

  /// The ID of the node that this task is on.
  #[serde(rename = "NodeID")]
  pub node_id: Option<String>,

  #[serde(rename = "AssignedGenericResources")]
  pub assigned_generic_resources: Option<GenericResources>,

  #[serde(rename = "Status")]
  pub status: Option<TaskStatus>,

  #[serde(rename = "DesiredState")]
  pub desired_state: Option<TaskState>,

  /// If the Service this Task belongs to is a job-mode service, contains the JobIteration of the Service this Task was created for. Absent if the Task was created for a Replicated or Global Service.
  #[serde(rename = "JobIteration")]
  pub job_iteration: Option<ObjectVersion>,
}

/// User modifiable task configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpec {
  #[serde(rename = "PluginSpec")]
  pub plugin_spec: Option<TaskSpecPluginSpec>,

  #[serde(rename = "ContainerSpec")]
  pub container_spec: Option<TaskSpecContainerSpec>,

  #[serde(rename = "NetworkAttachmentSpec")]
  pub network_attachment_spec: Option<TaskSpecNetworkAttachmentSpec>,

  #[serde(rename = "Resources")]
  pub resources: Option<TaskSpecResources>,

  #[serde(rename = "RestartPolicy")]
  pub restart_policy: Option<TaskSpecRestartPolicy>,

  #[serde(rename = "Placement")]
  pub placement: Option<TaskSpecPlacement>,

  /// A counter that triggers an update even if no relevant parameters have been changed.
  #[serde(rename = "ForceUpdate")]
  pub force_update: Option<U64>,

  /// Runtime is the type of runtime specified for the task executor.
  #[serde(rename = "Runtime")]
  pub runtime: Option<String>,

  /// Specifies which networks the service should attach to.
  #[serde(rename = "Networks")]
  pub networks: Option<Vec<NetworkAttachmentConfig>>,

  #[serde(rename = "LogDriver")]
  pub log_driver: Option<TaskSpecLogDriver>,
}

/// Plugin spec for the service.
/// *(Experimental release only.)*
///  <p><br /></p>  > **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive. PluginSpec is only used when the Runtime field > is set to `plugin`. NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecPluginSpec {
  /// The name or 'alias' to use for the plugin.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// The plugin image reference to use.
  #[serde(rename = "Remote")]
  pub remote: Option<String>,

  /// Disable the plugin once scheduled.
  #[serde(rename = "Disabled")]
  pub disabled: Option<bool>,

  #[serde(rename = "PluginPrivilege")]
  pub plugin_privilege: Option<Vec<PluginPrivilege>>,
}

/// Describes a permission the user has to accept upon installing the plugin.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PluginPrivilege {
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "Description")]
  pub description: Option<String>,

  #[serde(rename = "Value")]
  pub value: Option<Vec<String>>,
}

/// Container spec for the service.
/// **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive.
/// PluginSpec is only used when the Runtime field > is set to `plugin`.
/// NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpec {
  /// The image name to use for the container
  #[serde(rename = "Image")]
  pub image: Option<String>,

  /// User-defined key/value data.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  /// The command to be run in the image.
  #[serde(rename = "Command")]
  pub command: Option<Vec<String>>,

  /// Arguments to the command.
  #[serde(rename = "Args")]
  pub args: Option<Vec<String>>,

  /// The hostname to use for the container, as a valid [RFC 1123](https://tools.ietf.org/html/rfc1123) hostname.
  #[serde(rename = "Hostname")]
  pub hostname: Option<String>,

  /// A list of environment variables in the form `VAR=value`.
  #[serde(rename = "Env")]
  pub env: Option<Vec<String>>,

  /// The working directory for commands to run in.
  #[serde(rename = "Dir")]
  pub dir: Option<String>,

  /// The user inside the container.
  #[serde(rename = "User")]
  pub user: Option<String>,

  /// A list of additional groups that the container process will run as.
  #[serde(rename = "Groups")]
  pub groups: Option<Vec<String>>,

  #[serde(rename = "Privileges")]
  pub privileges: Option<TaskSpecContainerSpecPrivileges>,

  /// Whether a pseudo-TTY should be allocated.
  #[serde(rename = "TTY")]
  pub tty: Option<bool>,

  /// Open `stdin`
  #[serde(rename = "OpenStdin")]
  pub open_stdin: Option<bool>,

  /// Mount the container's root filesystem as read only.
  #[serde(rename = "ReadOnly")]
  pub read_only: Option<bool>,

  /// Specification for mounts to be added to containers created as part of the service.
  #[serde(rename = "Mounts")]
  pub mounts: Option<Vec<Mount>>,

  /// Signal to stop the container.
  #[serde(rename = "StopSignal")]
  pub stop_signal: Option<String>,

  /// Amount of time to wait for the container to terminate before forcefully killing it.
  #[serde(rename = "StopGracePeriod")]
  pub stop_grace_period: Option<I64>,

  #[serde(rename = "HealthCheck")]
  pub health_check: Option<HealthConfig>,

  /// A list of hostname/IP mappings to add to the container's `hosts` file. The format of extra hosts is specified in the [hosts(5)](http://man7.org/linux/man-pages/man5/hosts.5.html) man page:      IP_address canonical_hostname [aliases...]
  #[serde(rename = "Hosts")]
  pub hosts: Option<Vec<String>>,

  #[serde(rename = "DNSConfig")]
  pub dns_config: Option<TaskSpecContainerSpecDnsConfig>,

  /// Secrets contains references to zero or more secrets that will be exposed to the service.
  #[serde(rename = "Secrets")]
  pub secrets: Option<Vec<TaskSpecContainerSpecSecrets>>,

  /// An integer value containing the score given to the container in order to tune OOM killer preferences.
  #[serde(rename = "OomScoreAdj")]
  pub oom_score_adj: Option<I64>,

  /// Configs contains references to zero or more configs that will be exposed to the service.
  #[serde(rename = "Configs")]
  pub configs: Option<Vec<TaskSpecContainerSpecConfigs>>,

  /// Isolation technology of the containers running the service. (Windows only)
  #[serde(rename = "Isolation")]
  pub isolation: Option<TaskSpecContainerSpecIsolationEnum>,

  /// Run an init inside the container that forwards signals and reaps processes. This field is omitted if empty, and the default (as configured on the daemon) is used.
  #[serde(rename = "Init")]
  pub init: Option<bool>,

  /// Set kernel namedspaced parameters (sysctls) in the container. The Sysctls option on services accepts the same sysctls as the are supported on containers. Note that while the same sysctls are supported, no guarantees or checks are made about their suitability for a clustered environment, and it's up to the user to determine whether a given sysctl will work properly in a Service.
  #[serde(rename = "Sysctls")]
  pub sysctls: Option<HashMap<String, String>>,

  /// A list of kernel capabilities to add to the default set for the container.
  #[serde(rename = "CapabilityAdd")]
  pub capability_add: Option<Vec<String>>,

  /// A list of kernel capabilities to drop from the default set for the container.
  #[serde(rename = "CapabilityDrop")]
  pub capability_drop: Option<Vec<String>>,

  /// A list of resource limits to set in the container. For example: `{\"Name\": \"nofile\", \"Soft\": 1024, \"Hard\": 2048}`\"
  #[serde(rename = "Ulimits")]
  pub ulimits: Option<Vec<ResourcesUlimits>>,
}

/// Security options for the container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecPrivileges {
  #[serde(rename = "CredentialSpec")]
  pub credential_spec:
    Option<TaskSpecContainerSpecPrivilegesCredentialSpec>,

  #[serde(rename = "SELinuxContext")]
  pub se_linux_context:
    Option<TaskSpecContainerSpecPrivilegesSeLinuxContext>,

  #[serde(rename = "Seccomp")]
  pub seccomp: Option<TaskSpecContainerSpecPrivilegesSeccomp>,

  #[serde(rename = "AppArmor")]
  pub app_armor: Option<TaskSpecContainerSpecPrivilegesAppArmor>,

  /// Configuration of the no_new_privs bit in the container
  #[serde(rename = "NoNewPrivileges")]
  pub no_new_privileges: Option<bool>,
}

/// CredentialSpec for managed service account (Windows only)
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecPrivilegesCredentialSpec {
  /// Load credential spec from a Swarm Config with the given ID. The specified config must also be present in the Configs field with the Runtime property set.  <p><br /></p>   > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive.
  #[serde(rename = "Config")]
  pub config: Option<String>,

  /// Load credential spec from this file. The file is read by the daemon, and must be present in the `CredentialSpecs` subdirectory in the docker data directory, which defaults to `C:\\ProgramData\\Docker\\` on Windows.  For example, specifying `spec.json` loads `C:\\ProgramData\\Docker\\CredentialSpecs\\spec.json`.  <p><br /></p>  > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive.
  #[serde(rename = "File")]
  pub file: Option<String>,

  /// Load credential spec from this value in the Windows registry. The specified registry value must be located in:  `HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Virtualization\\Containers\\CredentialSpecs`  <p><br /></p>   > **Note**: `CredentialSpec.File`, `CredentialSpec.Registry`, > and `CredentialSpec.Config` are mutually exclusive.
  #[serde(rename = "Registry")]
  pub registry: Option<String>,
}

/// SELinux labels of the container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecPrivilegesSeLinuxContext {
  /// Disable SELinux
  #[serde(rename = "Disable")]
  pub disable: Option<bool>,

  /// SELinux user label
  #[serde(rename = "User")]
  pub user: Option<String>,

  /// SELinux role label
  #[serde(rename = "Role")]
  pub role: Option<String>,

  /// SELinux type label
  #[serde(rename = "Type")]
  pub typ: Option<String>,

  /// SELinux level label
  #[serde(rename = "Level")]
  pub level: Option<String>,
}

/// Options for configuring seccomp on the container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecPrivilegesSeccomp {
  #[serde(rename = "Mode")]
  pub mode: Option<TaskSpecContainerSpecPrivilegesSeccompModeEnum>,

  /// The custom seccomp profile as a json object
  #[serde(rename = "Profile")]
  pub profile: Option<String>,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskSpecContainerSpecPrivilegesSeccompModeEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "default")]
  DEFAULT,
  #[serde(rename = "unconfined")]
  UNCONFINED,
  #[serde(rename = "custom")]
  CUSTOM,
}

/// Options for configuring AppArmor on the container
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecPrivilegesAppArmor {
  #[serde(rename = "Mode")]
  pub mode: Option<TaskSpecContainerSpecPrivilegesAppArmorModeEnum>,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskSpecContainerSpecPrivilegesAppArmorModeEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "default")]
  DEFAULT,
  #[serde(rename = "disabled")]
  DISABLED,
}

/// Specification for DNS related configurations in resolver configuration file (`resolv.conf`).
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecDnsConfig {
  /// The IP addresses of the name servers.
  #[serde(rename = "Nameservers")]
  pub nameservers: Option<Vec<String>>,

  /// A search list for host-name lookup.
  #[serde(rename = "Search")]
  pub search: Option<Vec<String>>,

  /// A list of internal resolver variables to be modified (e.g., `debug`, `ndots:3`, etc.).
  #[serde(rename = "Options")]
  pub options: Option<Vec<String>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecSecrets {
  #[serde(rename = "File")]
  pub file: Option<TaskSpecContainerSpecFile>,

  /// SecretID represents the ID of the specific secret that we're referencing.
  #[serde(rename = "SecretID")]
  pub secret_id: Option<String>,

  /// SecretName is the name of the secret that this references, but this is just provided for lookup/display purposes. The secret in the reference will be identified by its ID.
  #[serde(rename = "SecretName")]
  pub secret_name: Option<String>,
}

/// File represents a specific target that is backed by a file.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecFile {
  /// Name represents the final filename in the filesystem.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// UID represents the file UID.
  #[serde(rename = "UID")]
  pub uid: Option<String>,

  /// GID represents the file GID.
  #[serde(rename = "GID")]
  pub gid: Option<String>,

  /// Mode represents the FileMode of the file.
  #[serde(rename = "Mode")]
  pub mode: Option<u32>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecContainerSpecConfigs {
  #[serde(rename = "File")]
  pub file: Option<TaskSpecContainerSpecFile>,

  // /// Runtime represents a target that is not mounted into the container but is used by the task  <p><br /><p>  > **Note**: `Configs.File` and `Configs.Runtime` are mutually > exclusive
  // #[serde(rename = "Runtime")]
  // pub runtime: Option<HashMap<(), ()>>,
  /// ConfigID represents the ID of the specific config that we're referencing.
  #[serde(rename = "ConfigID")]
  pub config_id: Option<String>,

  /// ConfigName is the name of the config that this references, but this is just provided for lookup/display purposes. The config in the reference will be identified by its ID.
  #[serde(rename = "ConfigName")]
  pub config_name: Option<String>,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskSpecContainerSpecIsolationEnum {
  #[serde(rename = "default")]
  DEFAULT,
  #[serde(rename = "process")]
  PROCESS,
  #[serde(rename = "hyperv")]
  HYPERV,
  #[default]
  #[serde(rename = "")]
  EMPTY,
}

/// Read-only spec type for non-swarm containers attached to swarm overlay networks.  <p><br /></p>  > **Note**: ContainerSpec, NetworkAttachmentSpec, and PluginSpec are > mutually exclusive. PluginSpec is only used when the Runtime field > is set to `plugin`. NetworkAttachmentSpec is used when the Runtime > field is set to `attachment`.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecNetworkAttachmentSpec {
  /// ID of the container represented by this task
  #[serde(rename = "ContainerID")]
  pub container_id: Option<String>,
}

/// Resource requirements which apply to each individual container created as part of the service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecResources {
  /// Define resources limits.
  #[serde(rename = "Limits")]
  pub limits: Option<Limit>,

  /// Define resources reservation.
  #[serde(rename = "Reservations")]
  pub reservations: Option<ResourceObject>,
}

/// An object describing a limit on resources which can be requested by a task.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Limit {
  #[serde(rename = "NanoCPUs")]
  pub nano_cpus: Option<I64>,

  #[serde(rename = "MemoryBytes")]
  pub memory_bytes: Option<I64>,

  /// Limits the maximum number of PIDs in the container. Set `0` for unlimited.
  #[serde(rename = "Pids")]
  pub pids: Option<I64>,
}

/// Specification for the restart policy which applies to containers created as part of this service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecRestartPolicy {
  /// Condition for restart.
  #[serde(rename = "Condition")]
  pub condition: Option<TaskSpecRestartPolicyConditionEnum>,

  /// Delay between restart attempts.
  #[serde(rename = "Delay")]
  pub delay: Option<I64>,

  /// Maximum attempts to restart a given container before giving up (default value is 0, which is ignored).
  #[serde(rename = "MaxAttempts")]
  pub max_attempts: Option<I64>,

  /// Windows is the time window used to evaluate the restart policy (default value is 0, which is unbounded).
  #[serde(rename = "Window")]
  pub window: Option<I64>,
}

#[allow(non_camel_case_types)]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskSpecRestartPolicyConditionEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "none")]
  NONE,
  #[serde(rename = "on-failure")]
  ON_FAILURE,
  #[serde(rename = "any")]
  ANY,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecPlacement {
  /// An array of constraint expressions to limit the set of nodes where a task can be scheduled. Constraint expressions can either use a _match_ (`==`) or _exclude_ (`!=`) rule. Multiple constraints find nodes that satisfy every expression (AND match). Constraints can match node or Docker Engine labels as follows:  node attribute       | matches                        | example ---------------------|--------------------------------|----------------------------------------------- `node.id`            | Node ID                        | `node.id==2ivku8v2gvtg4` `node.hostname`      | Node hostname                  | `node.hostname!=node-2` `node.role`          | Node role (`manager`/`worker`) | `node.role==manager` `node.platform.os`   | Node operating system          | `node.platform.os==windows` `node.platform.arch` | Node architecture              | `node.platform.arch==x86_64` `node.labels`        | User-defined node labels       | `node.labels.security==high` `engine.labels`      | Docker Engine's labels         | `engine.labels.operatingsystem==ubuntu-24.04`  `engine.labels` apply to Docker Engine labels like operating system, drivers, etc. Swarm administrators add `node.labels` for operational purposes by using the [`node update endpoint`](#operation/NodeUpdate).
  #[serde(rename = "Constraints")]
  pub constraints: Option<Vec<String>>,

  /// Preferences provide a way to make the scheduler aware of factors such as topology. They are provided in order from highest to lowest precedence.
  #[serde(rename = "Preferences")]
  pub preferences: Option<Vec<TaskSpecPlacementPreferences>>,

  /// Maximum number of replicas for per node (default value is 0, which is unlimited)
  #[serde(rename = "MaxReplicas")]
  pub max_replicas: Option<I64>,

  /// Platforms stores all the platforms that the service's image can run on. This field is used in the platform filter for scheduling. If empty, then the platform filter is off, meaning there are no scheduling restrictions.
  #[serde(rename = "Platforms")]
  pub platforms: Option<Vec<Platform>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecPlacementPreferences {
  #[serde(rename = "Spread")]
  pub spread: Option<TaskSpecPlacementSpread>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecPlacementSpread {
  /// label descriptor, such as `engine.labels.az`.
  #[serde(rename = "SpreadDescriptor")]
  pub spread_descriptor: Option<String>,
}

/// Specifies the log driver to use for tasks created from this spec.
/// If not present, the default one for the swarm will be used,
/// finally falling back to the engine default if not specified.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskSpecLogDriver {
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "Options")]
  pub options: Option<HashMap<String, String>>,
}

/// represents the status of a task.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TaskStatus {
  #[serde(rename = "Timestamp")]
  pub timestamp: Option<String>,

  #[serde(rename = "State")]
  pub state: Option<TaskState>,

  #[serde(rename = "Message")]
  pub message: Option<String>,

  #[serde(rename = "Err")]
  pub err: Option<String>,

  #[serde(rename = "ContainerStatus")]
  pub container_status: Option<ContainerStatus>,

  #[serde(rename = "PortStatus")]
  pub port_status: Option<PortStatus>,
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
  Serialize,
  Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskState {
  #[serde(rename = "new")]
  NEW,
  #[serde(rename = "allocated")]
  ALLOCATED,
  #[serde(rename = "pending")]
  PENDING,
  #[serde(rename = "assigned")]
  ASSIGNED,
  #[serde(rename = "accepted")]
  ACCEPTED,
  #[serde(rename = "preparing")]
  PREPARING,
  #[serde(rename = "ready")]
  READY,
  #[serde(rename = "starting")]
  STARTING,
  #[serde(rename = "running")]
  RUNNING,
  #[serde(rename = "complete")]
  COMPLETE,
  #[serde(rename = "shutdown")]
  SHUTDOWN,
  #[serde(rename = "failed")]
  FAILED,
  #[serde(rename = "rejected")]
  REJECTED,
  #[serde(rename = "remove")]
  REMOVE,
  #[serde(rename = "orphaned")]
  ORPHANED,
}

/// represents the status of a container.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ContainerStatus {
  #[serde(rename = "ContainerID")]
  pub container_id: Option<String>,

  #[serde(rename = "PID")]
  pub pid: Option<I64>,

  #[serde(rename = "ExitCode")]
  pub exit_code: Option<I64>,
}

/// represents the port status of a task's host ports whose service has published host ports
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PortStatus {
  #[serde(rename = "Ports")]
  pub ports: Option<Vec<EndpointPortConfig>>,
}
