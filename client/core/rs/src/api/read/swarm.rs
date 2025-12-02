use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  SearchCombinator, U64,
  docker::{
    config::{SwarmConfig, SwarmConfigListItem},
    node::{SwarmNode, SwarmNodeListItem},
    secret::{SwarmSecret, SwarmSecretListItem},
    service::{SwarmService, SwarmServiceListItem},
    stack::{SwarmStack, SwarmStackListItem},
    swarm::SwarmInspectInfo,
    task::{SwarmTask, SwarmTaskListItem},
  },
  swarm::{Swarm, SwarmActionState, SwarmListItem, SwarmQuery},
  update::Log,
};

use super::KomodoReadRequest;

//

/// Get a specific swarm. Response: [Swarm].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(Swarm)]
#[error(serror::Error)]
pub struct GetSwarm {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type GetSwarmResponse = Swarm;

//

/// List Swarms matching optional query. Response: [ListSwarmsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmsResponse)]
#[error(serror::Error)]
pub struct ListSwarms {
  /// Optional structured query to filter Swarms.
  #[serde(default)]
  pub query: SwarmQuery,
}

#[typeshare]
pub type ListSwarmsResponse = Vec<SwarmListItem>;

//

/// List Swarms matching optional query. Response: [ListFullSwarmsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullSwarmsResponse)]
#[error(serror::Error)]
pub struct ListFullSwarms {
  /// optional structured query to filter swarms.
  #[serde(default)]
  pub query: SwarmQuery,
}

#[typeshare]
pub type ListFullSwarmsResponse = Vec<Swarm>;

//

/// Get current action state for the swarm. Response: [SwarmActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmActionStateResponse)]
#[error(serror::Error)]
pub struct GetSwarmActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type GetSwarmActionStateResponse = SwarmActionState;

//

/// Gets a summary of data relating to all swarms.
/// Response: [GetSwarmsSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmsSummaryResponse)]
#[error(serror::Error)]
pub struct GetSwarmsSummary {}

/// Response for [GetSwarmsSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetSwarmsSummaryResponse {
  /// The total number of Swarms
  pub total: u32,
  /// The number of Swarms with Healthy state.
  pub healthy: u32,
  /// The number of Swarms with Unhealthy state
  pub unhealthy: u32,
  /// The number of Swarms with Unknown state
  pub unknown: u32,
}

//

/// Inspect information about the swarm.
/// Response: [SwarmInspectInfo].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmResponse)]
#[error(serror::Error)]
pub struct InspectSwarm {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type InspectSwarmResponse = SwarmInspectInfo;

//

/// List nodes part of the target Swarm.
/// Response: [ListSwarmNodesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmNodesResponse)]
#[error(serror::Error)]
pub struct ListSwarmNodes {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmNodesResponse = Vec<SwarmNodeListItem>;

//

/// Inspect a Swarm node.
/// Response: [SwarmNode].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmNodeResponse)]
#[error(serror::Error)]
pub struct InspectSwarmNode {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Node id
  pub node: String,
}

#[typeshare]
pub type InspectSwarmNodeResponse = SwarmNode;

//

/// List services on the target Swarm.
/// Response: [ListSwarmServicesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmServicesResponse)]
#[error(serror::Error)]
pub struct ListSwarmServices {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmServicesResponse = Vec<SwarmServiceListItem>;

//

/// Inspect a Swarm service.
/// Response: [SwarmService].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmServiceResponse)]
#[error(serror::Error)]
pub struct InspectSwarmService {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Service id
  pub service: String,
}

#[typeshare]
pub type InspectSwarmServiceResponse = SwarmService;

//

/// Get a swarm service's logs. Response: [GetSwarmServiceLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmServiceLogResponse)]
#[error(serror::Error)]
pub struct GetSwarmServiceLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Select the swarm service to get logs for.
  pub service: String,
  /// The number of lines of the log tail to include.
  /// Default: 100.
  /// Max: 5000.
  #[serde(default = "default_tail")]
  pub tail: U64,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
  /// Enable `--no-task-ids`
  #[serde(default)]
  pub no_task_ids: bool,
  /// Enable `--no-resolve`
  #[serde(default)]
  pub no_resolve: bool,
  /// Enable `--details`
  #[serde(default)]
  pub details: bool,
}

fn default_tail() -> u64 {
  50
}

#[typeshare]
pub type GetSwarmServiceLogResponse = Log;

//

/// Search the swarm service log's tail using `grep`. All lines go to stdout.
/// Response: [SearchSwarmServiceLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(SearchSwarmServiceLogResponse)]
#[error(serror::Error)]
pub struct SearchSwarmServiceLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Select the swarm service to get logs for.
  pub service: String,
  /// The terms to search for.
  pub terms: Vec<String>,
  /// When searching for multiple terms, can use `AND` or `OR` combinator.
  ///
  /// - `AND`: Only include lines with **all** terms present in that line.
  /// - `OR`: Include lines that have one or more matches in the terms.
  #[serde(default)]
  pub combinator: SearchCombinator,
  /// Invert the results, ie return all lines that DON'T match the terms / combinator.
  #[serde(default)]
  pub invert: bool,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
  /// Enable `--no-task-ids`
  #[serde(default)]
  pub no_task_ids: bool,
  /// Enable `--no-resolve`
  #[serde(default)]
  pub no_resolve: bool,
  /// Enable `--details`
  #[serde(default)]
  pub details: bool,
}

#[typeshare]
pub type SearchSwarmServiceLogResponse = Log;

//

/// List tasks on the target Swarm.
/// Response: [ListSwarmTasksResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmTasksResponse)]
#[error(serror::Error)]
pub struct ListSwarmTasks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmTasksResponse = Vec<SwarmTaskListItem>;

//

/// Inspect a Swarm task.
/// Response: [SwarmTask].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmTaskResponse)]
#[error(serror::Error)]
pub struct InspectSwarmTask {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Task id
  pub task: String,
}

#[typeshare]
pub type InspectSwarmTaskResponse = SwarmTask;

//

/// List secrets on the target Swarm.
/// Response: [ListSwarmSecretsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmSecretsResponse)]
#[error(serror::Error)]
pub struct ListSwarmSecrets {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmSecretsResponse = Vec<SwarmSecretListItem>;

//

/// Inspect a Swarm secret.
/// Response: [SwarmSecret].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmSecretResponse)]
#[error(serror::Error)]
pub struct InspectSwarmSecret {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Secret id
  pub secret: String,
}

#[typeshare]
pub type InspectSwarmSecretResponse = SwarmSecret;

//

/// List configs on the target Swarm.
/// Response: [ListSwarmConfigsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmConfigsResponse)]
#[error(serror::Error)]
pub struct ListSwarmConfigs {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmConfigsResponse = Vec<SwarmConfigListItem>;

//

/// Inspect a config on the target Swarm.
/// Response: [InspectSwarmConfigResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmConfigResponse)]
#[error(serror::Error)]
pub struct InspectSwarmConfig {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Swarm config ID or Name
  pub config: String,
}

#[typeshare]
pub type InspectSwarmConfigResponse = Vec<SwarmConfig>;

//

/// List stacks on the target Swarm.
/// Response: [ListSwarmStacksResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmStacksResponse)]
#[error(serror::Error)]
pub struct ListSwarmStacks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmStacksResponse = Vec<SwarmStackListItem>;

//

/// Inspect a stack on the target Swarm.
/// Response: [SwarmStackLists].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmStackResponse)]
#[error(serror::Error)]
pub struct InspectSwarmStack {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Swarm stack name
  pub stack: String,
}

#[typeshare]
pub type InspectSwarmStackResponse = SwarmStack;
