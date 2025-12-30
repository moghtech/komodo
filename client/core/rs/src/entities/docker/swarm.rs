use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::*;

/// Docker-level information about the Swarm.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmInspectInfo {
  /// The (Docker) ID of the swarm.
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  /// Date and time at which the swarm was initialised in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  /// Date and time at which the swarm was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<SwarmSpec>,

  #[serde(rename = "TLSInfo")]
  pub tls_info: Option<TlsInfo>,

  /// Whether there is currently a root CA rotation in progress for the swarm
  #[serde(rename = "RootRotationInProgress")]
  pub root_rotation_in_progress: Option<bool>,

  /// DataPathPort specifies the data path port number for data traffic. Acceptable port range is 1024 to 49151. If no port is set or is set to 0, the default port (4789) is used.
  #[serde(rename = "DataPathPort")]
  pub data_path_port: Option<u32>,

  /// Default Address Pool specifies default subnet pools for global scope networks.
  #[serde(rename = "DefaultAddrPool")]
  pub default_addr_pool: Option<Vec<String>>,

  /// SubnetSize specifies the subnet size of the networks created from the default subnet pool.
  #[serde(rename = "SubnetSize")]
  pub subnet_size: Option<u32>,

  #[serde(rename = "JoinTokens")]
  pub join_tokens: Option<JoinTokens>,
}

/// User modifiable swarm configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpec {
  /// Name of the swarm.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  #[serde(rename = "Orchestration")]
  pub orchestration: Option<SwarmSpecOrchestration>,

  #[serde(rename = "Raft")]
  pub raft: Option<SwarmSpecRaft>,

  #[serde(rename = "Dispatcher")]
  pub dispatcher: Option<SwarmSpecDispatcher>,

  #[serde(rename = "CAConfig")]
  pub ca_config: Option<SwarmSpecCaConfig>,

  #[serde(rename = "EncryptionConfig")]
  pub encryption_config: Option<SwarmSpecEncryptionConfig>,

  #[serde(rename = "TaskDefaults")]
  pub task_defaults: Option<SwarmSpecTaskDefaults>,
}

/// Orchestration configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecOrchestration {
  /// The number of historic tasks to keep per instance or node.
  /// If negative, never remove completed or failed tasks.
  #[serde(rename = "TaskHistoryRetentionLimit")]
  pub task_history_retention_limit: Option<I64>,
}

/// Raft configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecRaft {
  /// The number of log entries between snapshots.
  #[serde(rename = "SnapshotInterval")]
  pub snapshot_interval: Option<U64>,

  /// The number of snapshots to keep beyond the current snapshot.
  #[serde(rename = "KeepOldSnapshots")]
  pub keep_old_snapshots: Option<U64>,

  /// The number of log entries to keep around to sync up slow followers after a snapshot is created.
  #[serde(rename = "LogEntriesForSlowFollowers")]
  pub log_entries_for_slow_followers: Option<U64>,

  /// The number of ticks that a follower will wait for a message from the leader before becoming a candidate and starting an election. `ElectionTick` must be greater than `HeartbeatTick`.  A tick currently defaults to one second, so these translate directly to seconds currently, but this is NOT guaranteed.
  #[serde(rename = "ElectionTick")]
  pub election_tick: Option<I64>,

  /// The number of ticks between heartbeats.
  /// Every HeartbeatTick ticks, the leader will send a heartbeat to the followers.
  /// A tick currently defaults to one second, so these translate directly to seconds currently, but this is NOT guaranteed.
  #[serde(rename = "HeartbeatTick")]
  pub heartbeat_tick: Option<I64>,
}

/// Dispatcher configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecDispatcher {
  /// The delay for an agent to send a heartbeat to the dispatcher.
  #[serde(rename = "HeartbeatPeriod")]
  pub heartbeat_period: Option<I64>,
}

/// CA configuration.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecCaConfig {
  /// The duration node certificates are issued for.
  #[serde(rename = "NodeCertExpiry")]
  pub node_cert_expiry: Option<I64>,

  /// Configuration for forwarding signing requests to an external certificate authority.
  #[serde(rename = "ExternalCAs")]
  pub external_cas: Option<Vec<SwarmSpecCaConfigExternalCas>>,

  /// The desired signing CA certificate for all swarm node TLS leaf certificates, in PEM format.
  #[serde(rename = "SigningCACert")]
  pub signing_ca_cert: Option<String>,

  /// The desired signing CA key for all swarm node TLS leaf certificates, in PEM format.
  #[serde(rename = "SigningCAKey")]
  pub signing_ca_key: Option<String>,

  /// An integer whose purpose is to force swarm to generate a new signing CA certificate and key, if none have been specified in `SigningCACert` and `SigningCAKey`
  #[serde(rename = "ForceRotate")]
  pub force_rotate: Option<U64>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecCaConfigExternalCas {
  /// Protocol for communication with the external CA (currently only `cfssl` is supported).
  #[serde(rename = "Protocol")]
  pub protocol: Option<SwarmSpecCaConfigExternalCasProtocolEnum>,

  /// URL where certificate signing requests should be sent.
  #[serde(rename = "URL")]
  pub url: Option<String>,

  /// An object with key/value pairs that are interpreted as protocol-specific options for the external CA driver.
  #[serde(rename = "Options")]
  pub options: Option<HashMap<String, String>>,

  /// The root CA certificate (in PEM format) this external CA uses to issue TLS certificates (assumed to be to the current swarm root CA certificate if not provided).
  #[serde(rename = "CACert")]
  pub ca_cert: Option<String>,
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SwarmSpecCaConfigExternalCasProtocolEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "cfssl")]
  CFSSL,
}

/// Parameters related to encryption-at-rest.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecEncryptionConfig {
  /// If set, generate a key and use it to lock data stored on the managers.
  #[serde(rename = "AutoLockManagers")]
  pub auto_lock_managers: Option<bool>,
}

/// Defaults for creating tasks in this cluster.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecTaskDefaults {
  #[serde(rename = "LogDriver")]
  pub log_driver: Option<SwarmSpecTaskDefaultsLogDriver>,
}

/// The log driver to use for tasks created in the orchestrator if unspecified by a service.  Updating this value only affects new tasks. Existing tasks continue to use their previously configured log driver until recreated.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SwarmSpecTaskDefaultsLogDriver {
  /// The log driver to use as a default for new tasks.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// Driver-specific options for the selected log driver, specified as key/value pairs.
  #[serde(rename = "Options")]
  pub options: Option<HashMap<String, String>>,
}

/// JoinTokens contains the tokens workers and managers need to join the swarm.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JoinTokens {
  /// The token workers can use to join the swarm.
  #[serde(rename = "Worker")]
  pub worker: Option<String>,

  /// The token managers can use to join the swarm.
  #[serde(rename = "Manager")]
  pub manager: Option<String>,
}
