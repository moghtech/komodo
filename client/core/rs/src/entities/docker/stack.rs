use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::swarm::SwarmState;

/// Swarm stack list item.
/// Returned by `docker stack ls --format json`
///
/// https://docs.docker.com/reference/cli/docker/stack/ls/#format
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmStackListItem {
  /// Swarm stack name.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// Swarm stack state.
  /// - Healthy if all associated tasks match their desired state
  /// - Unhealthy otherwise
  ///
  /// Not included in docker cli return, computed by Komodo
  #[serde(rename = "State")]
  pub state: Option<SwarmState>,

  /// Number of services which are part of the stack
  #[serde(rename = "Services")]
  pub services: Option<String>,

  /// The stack orchestrator
  #[serde(rename = "Orchestrator")]
  pub orchestrator: Option<String>,

  /// The stack namespace
  #[serde(rename = "Namespace")]
  pub namespace: Option<String>,
}

/// All entities related to docker stack available over CLI.
/// Returned by:
/// ```
/// docker stack services --format json <STACK>
/// docker stack ps --format json <STACK>
/// ```
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmStack {
  /// Swarm stack name.
  #[serde(rename = "Name")]
  pub name: String,

  /// Swarm stack state.
  /// - Healthy if all associated tasks match their desired state (or report no desired state)
  /// - Unhealthy otherwise
  ///
  /// Not included in docker cli return, computed by Komodo
  #[serde(rename = "State")]
  pub state: SwarmState,

  /// Services part of the stack
  #[serde(rename = "Services")]
  pub services: Vec<SwarmStackServiceListItem>,

  /// Tasks part of the stack
  #[serde(rename = "Tasks")]
  pub tasks: Vec<SwarmStackTaskListItem>,
}

/// Swarm stack service list item.
/// Returned by `docker stack services --format json <NAME>`
///
/// https://docs.docker.com/reference/cli/docker/stack/services/#format
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmStackServiceListItem {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Swarm stack task name.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// The service mode.
  #[serde(rename = "Mode")]
  pub mode: Option<String>,

  /// The service replicas, formatted as string.
  #[serde(rename = "Replicas")]
  pub replicas: Option<String>,

  /// The image associated with service
  #[serde(rename = "Image")]
  pub image: Option<String>,

  /// Task exposed ports, formatted as a string.
  #[serde(rename = "Ports")]
  pub ports: Option<String>,
}

/// Swarm stack task list item.
/// Returned by `docker stack ps --format json <NAME>`
///
/// https://docs.docker.com/reference/cli/docker/stack/ps/#format
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmStackTaskListItem {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Swarm stack task name.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// The image associated with task
  #[serde(rename = "Image")]
  pub image: Option<String>,

  /// The node the task is running on
  #[serde(rename = "Node")]
  pub node: Option<String>,

  #[serde(rename = "DesiredState")]
  pub desired_state: Option<String>,

  #[serde(rename = "CurrentState")]
  pub current_state: Option<String>,

  /// An error message, if one exists
  #[serde(rename = "Error")]
  pub error: Option<String>,

  /// Task exposed ports, formatted as a string.
  #[serde(rename = "Ports")]
  pub ports: Option<String>,
}
