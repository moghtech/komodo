//! Docker Swarm Node

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use typeshare::typeshare;

use super::*;

/// Swarm node list item.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmNodeListItem {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Name for the node.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "Hostname")]
  pub hostname: Option<String>,

  /// Role of the node.
  #[serde(rename = "Role")]
  pub role: Option<NodeSpecRoleEnum>,

  /// Availability of the node.
  #[serde(rename = "Availability")]
  pub availability: Option<NodeSpecAvailabilityEnum>,

  /// State of the node
  #[serde(rename = "State")]
  pub state: Option<NodeState>,

  /// Date and time at which the node was added to the swarm in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  /// Date and time at which the node was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,
}

/// Swarm node details.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmNode {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  /// Date and time at which the node was added to the swarm in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  /// Date and time at which the node was last updated in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<NodeSpec>,

  #[serde(rename = "Description")]
  pub description: Option<NodeDescription>,

  #[serde(rename = "Status")]
  pub status: Option<NodeStatus>,

  #[serde(rename = "ManagerStatus")]
  pub manager_status: Option<ManagerStatus>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NodeSpec {
  /// Name for the node.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  /// Role of the node.
  #[serde(rename = "Role")]
  pub role: Option<NodeSpecRoleEnum>,

  /// Availability of the node.
  #[serde(rename = "Availability")]
  pub availability: Option<NodeSpecAvailabilityEnum>,
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
  AsRefStr,
)]
pub enum NodeSpecRoleEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "worker")]
  WORKER,
  #[serde(rename = "manager")]
  MANAGER,
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
  AsRefStr,
)]
pub enum NodeSpecAvailabilityEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "active")]
  ACTIVE,
  #[serde(rename = "pause")]
  PAUSE,
  #[serde(rename = "drain")]
  DRAIN,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NodeDescription {
  #[serde(rename = "Hostname")]
  pub hostname: Option<String>,

  #[serde(rename = "Platform")]
  pub platform: Option<Platform>,

  #[serde(rename = "Resources")]
  pub resources: Option<ResourceObject>,

  #[serde(rename = "Engine")]
  pub engine: Option<EngineDescription>,

  #[serde(rename = "TLSInfo")]
  pub tls_info: Option<TlsInfo>,
}

/// EngineDescription provides information about an engine.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EngineDescription {
  #[serde(rename = "EngineVersion")]
  pub engine_version: Option<String>,

  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  #[serde(rename = "Plugins")]
  pub plugins: Option<Vec<EngineDescriptionPlugins>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EngineDescriptionPlugins {
  #[serde(rename = "Type")]
  pub typ: Option<String>,

  #[serde(rename = "Name")]
  pub name: Option<String>,
}

/// NodeStatus represents the status of a node.  It provides the current status of the node, as seen by the manager.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct NodeStatus {
  #[serde(rename = "State")]
  pub state: Option<NodeState>,

  #[serde(rename = "Message")]
  pub message: Option<String>,

  /// IP address of the node.
  #[serde(rename = "Addr")]
  pub addr: Option<String>,
}

/// NodeState represents the state of a node.
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
pub enum NodeState {
  #[default]
  #[serde(rename = "unknown")]
  UNKNOWN,
  #[serde(rename = "down")]
  DOWN,
  #[serde(rename = "ready")]
  READY,
  #[serde(rename = "disconnected")]
  DISCONNECTED,
}

/// ManagerStatus represents the status of a manager.  It provides the current status of a node's manager component, if the node is a manager.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ManagerStatus {
  #[serde(rename = "Leader")]
  pub leader: Option<bool>,

  #[serde(rename = "Reachability")]
  pub reachability: Option<NodeReachability>,

  /// The IP address and port at which the manager is reachable.
  #[serde(rename = "Addr")]
  pub addr: Option<String>,
}

/// Reachability represents the reachability of a node.
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
pub enum NodeReachability {
  #[default]
  #[serde(rename = "unknown")]
  UNKNOWN,
  #[serde(rename = "unreachable")]
  UNREACHABLE,
  #[serde(rename = "reachable")]
  REACHABLE,
}
