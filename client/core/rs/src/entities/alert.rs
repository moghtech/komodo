use std::path::PathBuf;

use derive_variants::EnumVariants;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use typeshare::typeshare;

use crate::entities::{I64, MongoId};

use super::{
  _Serror, ResourceTarget, ResourceTargetVariant, Version,
  deployment::DeploymentState, stack::StackState,
};

/// Representation of an alert in the system.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", doc_index({ "data.type": 1 }))]
#[cfg_attr(feature = "mongo", doc_index({ "target.type": 1 }))]
#[cfg_attr(feature = "mongo", doc_index({ "target.id": 1 }))]
pub struct Alert {
  /// The Mongo ID of the alert.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of serialized Alert) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,

  /// Unix timestamp in milliseconds the alert was opened
  #[cfg_attr(feature = "mongo", index)]
  pub ts: I64,

  /// Whether the alert is already resolved
  #[cfg_attr(feature = "mongo", index)]
  pub resolved: bool,

  /// The severity of the alert
  #[cfg_attr(feature = "mongo", index)]
  pub level: SeverityLevel,

  /// The target of the alert
  pub target: ResourceTarget,

  /// The data attached to the alert
  pub data: AlertData,

  /// The timestamp of alert resolution
  pub resolved_ts: Option<I64>,
}

/// The variants of data related to the alert.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, EnumVariants)]
#[variant_derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash
)]
#[serde(tag = "type", content = "data")]
pub enum AlertData {
  /// A null alert
  None {},

  /// The user triggered a test of the
  /// Alerter configuration.
  Test {
    /// The id of the alerter
    id: String,
    /// The name of the alerter
    name: String,
  },

  /// A server could not be reached.
  ServerUnreachable {
    /// The id of the server
    id: String,
    /// The name of the server
    name: String,
    /// The region of the server
    region: Option<String>,
    /// The error data
    err: Option<_Serror>,
  },

  /// A server has high CPU usage.
  ServerCpu {
    /// The id of the server
    id: String,
    /// The name of the server
    name: String,
    /// The region of the server
    region: Option<String>,
    /// The cpu usage percentage
    percentage: f64,
  },

  /// A server has high memory usage.
  ServerMem {
    /// The id of the server
    id: String,
    /// The name of the server
    name: String,
    /// The region of the server
    region: Option<String>,
    /// The used memory
    used_gb: f64,
    /// The total memory
    total_gb: f64,
  },

  /// A server has high disk usage.
  ServerDisk {
    /// The id of the server
    id: String,
    /// The name of the server
    name: String,
    /// The region of the server
    region: Option<String>,
    /// The mount path of the disk
    path: PathBuf,
    /// The used portion of the disk in GB
    used_gb: f64,
    /// The total size of the disk in GB
    total_gb: f64,
  },

  /// A container's state has changed unexpectedly.
  ContainerStateChange {
    /// The id of the deployment
    id: String,
    /// The name of the deployment
    name: String,
    /// The server id of server that the deployment is on
    server_id: String,
    /// The server name
    server_name: String,
    /// The previous container state
    from: DeploymentState,
    /// The current container state
    to: DeploymentState,
  },

  /// A Deployment has an image update available
  DeploymentImageUpdateAvailable {
    /// The id of the deployment
    id: String,
    /// The name of the deployment
    name: String,
    /// The server id of server that the deployment is on
    server_id: String,
    /// The server name
    server_name: String,
    /// The image with update
    image: String,
  },

  /// A Deployment has an image update available
  DeploymentAutoUpdated {
    /// The id of the deployment
    id: String,
    /// The name of the deployment
    name: String,
    /// The server id of server that the deployment is on
    server_id: String,
    /// The server name
    server_name: String,
    /// The updated image
    image: String,
  },

  /// A stack's state has changed unexpectedly.
  StackStateChange {
    /// The id of the stack
    id: String,
    /// The name of the stack
    name: String,
    /// The server id of server that the stack is on
    server_id: String,
    /// The server name
    server_name: String,
    /// The previous stack state
    from: StackState,
    /// The current stack state
    to: StackState,
  },

  /// A Stack has an image update available
  StackImageUpdateAvailable {
    /// The id of the stack
    id: String,
    /// The name of the stack
    name: String,
    /// The server id of server that the stack is on
    server_id: String,
    /// The server name
    server_name: String,
    /// The service name to update
    service: String,
    /// The image with update
    image: String,
  },

  /// A Stack was auto updated
  StackAutoUpdated {
    /// The id of the stack
    id: String,
    /// The name of the stack
    name: String,
    /// The server id of server that the stack is on
    server_id: String,
    /// The server name
    server_name: String,
    /// One or more images that were updated
    images: Vec<String>,
  },

  /// An AWS builder failed to terminate.
  AwsBuilderTerminationFailed {
    /// The id of the aws instance which failed to terminate
    instance_id: String,
    /// A reason for the failure
    message: String,
  },

  /// A resource sync has pending updates
  ResourceSyncPendingUpdates {
    /// The id of the resource sync
    id: String,
    /// The name of the resource sync
    name: String,
  },

  /// A build has failed
  BuildFailed {
    /// The id of the build
    id: String,
    /// The name of the build
    name: String,
    /// The version that failed to build
    version: Version,
  },

  /// A repo has failed
  RepoBuildFailed {
    /// The id of the repo
    id: String,
    /// The name of the repo
    name: String,
  },

  /// A procedure has failed
  ProcedureFailed {
    /// The id of the procedure
    id: String,
    /// The name of the procedure
    name: String,
  },

  /// An action has failed
  ActionFailed {
    /// The id of the action
    id: String,
    /// The name of the action
    name: String,
  },

  /// A schedule was run
  ScheduleRun {
    /// Procedure or Action
    resource_type: ResourceTargetVariant,
    /// The resource id
    id: String,
    /// The resource name
    name: String,
  },

  /// Custom header / body.
  /// Produced using `/execute/SendAlert`
  Custom {
    /// The alert message.
    message: String,
    /// Message details. May be empty string.
    #[serde(default)]
    details: String,
  },
}

impl Default for AlertData {
  fn default() -> Self {
    AlertData::None {}
  }
}

#[allow(clippy::derivable_impls)]
impl Default for AlertDataVariant {
  fn default() -> Self {
    AlertDataVariant::None
  }
}

/// Severity level of problem.
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Default,
  Display,
  EnumString,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum SeverityLevel {
  /// No problem.
  ///
  /// Aliases: ok, low, l
  #[default]
  #[strum(serialize = "ok", serialize = "low", serialize = "l")]
  Ok,
  /// Problem is imminent.
  ///
  /// Aliases: warning, w, medium, m
  #[strum(
    serialize = "warning",
    serialize = "w",
    serialize = "medium",
    serialize = "m"
  )]
  Warning,
  /// Problem fully realized.
  ///
  /// Aliases: critical, c, high, h
  #[strum(
    serialize = "critical",
    serialize = "c",
    serialize = "high",
    serialize = "h"
  )]
  Critical,
}
