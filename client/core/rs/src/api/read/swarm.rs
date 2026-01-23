use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  SearchCombinator, U64,
  docker::{
    config::{SwarmConfig, SwarmConfigListItem},
    network::NetworkListItem,
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetSwarm",
  description = "Get a specific swarm.",
  request_body(content = GetSwarm),
  responses(
    (status = 200, description = "The swarm", body = crate::entities::swarm::SwarmSchema),
  ),
)]
pub fn get_swarm() {}

/// Get a specific swarm. Response: [Swarm].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmResponse)]
#[error(mogh_error::Error)]
pub struct GetSwarm {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type GetSwarmResponse = Swarm;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarms",
  description = "List Swarms matching optional query.",
  request_body(content = ListSwarms),
  responses(
    (status = 200, description = "The list of swarms", body = ListSwarmsResponse),
  ),
)]
pub fn list_swarms() {}

/// List Swarms matching optional query. Response: [ListSwarmsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmsResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarms {
  /// Optional structured query to filter Swarms.
  #[serde(default)]
  pub query: SwarmQuery,
}

#[typeshare]
pub type ListSwarmsResponse = Vec<SwarmListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullSwarms",
  description = "List Swarms matching optional query.",
  request_body(content = ListFullSwarms),
  responses(
    (status = 200, description = "The list of swarms", body = ListFullSwarmsResponse),
  ),
)]
pub fn list_full_swarms() {}

/// List Swarms matching optional query. Response: [ListFullSwarmsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullSwarmsResponse)]
#[error(mogh_error::Error)]
pub struct ListFullSwarms {
  /// optional structured query to filter swarms.
  #[serde(default)]
  pub query: SwarmQuery,
}

#[typeshare]
pub type ListFullSwarmsResponse = Vec<Swarm>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetSwarmActionState",
  description = "Get current action state for the swarm.",
  request_body(content = GetSwarmActionState),
  responses(
    (status = 200, description = "The swarm action state", body = GetSwarmActionStateResponse),
  ),
)]
pub fn get_swarm_action_state() {}

/// Get current action state for the swarm. Response: [SwarmActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetSwarmActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type GetSwarmActionStateResponse = SwarmActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetSwarmsSummary",
  description = "Gets a summary of data relating to all swarms.",
  request_body(content = GetSwarmsSummary),
  responses(
    (status = 200, description = "The swarms summary", body = GetSwarmsSummaryResponse),
  ),
)]
pub fn get_swarms_summary() {}

/// Gets a summary of data relating to all swarms.
/// Response: [GetSwarmsSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmsSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetSwarmsSummary {}

/// Response for [GetSwarmsSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetSwarmsSummaryResponse {
  /// The total number of Swarms
  pub total: u32,
  /// The number of Swarms with Healthy state.
  pub healthy: u32,
  /// The number of Swarms with Unhealthy state
  pub unhealthy: u32,
  /// The number of Swarms with Down state
  pub down: u32,
  /// The number of Swarms with Unknown state
  pub unknown: u32,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarm",
  description = "Inspect information about the swarm.",
  request_body(content = InspectSwarm),
  responses(
    (status = 200, description = "The swarm inspect info", body = InspectSwarmResponse),
  ),
)]
pub fn inspect_swarm() {}

/// Inspect information about the swarm.
/// Response: [SwarmInspectInfo].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmResponse)]
#[error(mogh_error::Error)]
pub struct InspectSwarm {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type InspectSwarmResponse = SwarmInspectInfo;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmNodes",
  description = "List nodes part of the target Swarm.",
  request_body(content = ListSwarmNodes),
  responses(
    (status = 200, description = "The list of swarm nodes", body = ListSwarmNodesResponse),
  ),
)]
pub fn list_swarm_nodes() {}

/// List nodes part of the target Swarm.
/// Response: [ListSwarmNodesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmNodesResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmNodes {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmNodesResponse = Vec<SwarmNodeListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmNode",
  description = "Inspect a Swarm node.",
  request_body(content = InspectSwarmNode),
  responses(
    (status = 200, description = "The swarm node", body = InspectSwarmNodeResponse),
  ),
)]
pub fn inspect_swarm_node() {}

/// Inspect a Swarm node.
/// Response: [SwarmNode].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmNodeResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmServices",
  description = "List services on the target Swarm.",
  request_body(content = ListSwarmServices),
  responses(
    (status = 200, description = "The list of swarm services", body = ListSwarmServicesResponse),
  ),
)]
pub fn list_swarm_services() {}

/// List services on the target Swarm.
/// Response: [ListSwarmServicesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmServicesResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmServices {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmServicesResponse = Vec<SwarmServiceListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmService",
  description = "Inspect a Swarm service.",
  request_body(content = InspectSwarmService),
  responses(
    (status = 200, description = "The swarm service", body = InspectSwarmServiceResponse),
  ),
)]
pub fn inspect_swarm_service() {}

/// Inspect a Swarm service.
/// Response: [SwarmService].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmServiceResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetSwarmServiceLog",
  description = "Get a swarm service's logs.",
  request_body(content = GetSwarmServiceLog),
  responses(
    (status = 200, description = "The swarm service log", body = GetSwarmServiceLogResponse),
  ),
)]
pub fn get_swarm_service_log() {}

/// Get a swarm service's logs. Response: [GetSwarmServiceLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSwarmServiceLogResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SearchSwarmServiceLog",
  description = "Search the swarm service log's tail using `grep`.",
  request_body(content = SearchSwarmServiceLog),
  responses(
    (status = 200, description = "The search results", body = SearchSwarmServiceLogResponse),
  ),
)]
pub fn search_swarm_service_log() {}

/// Search the swarm service log's tail using `grep`. All lines go to stdout.
/// Response: [SearchSwarmServiceLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(SearchSwarmServiceLogResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmTasks",
  description = "List tasks on the target Swarm.",
  request_body(content = ListSwarmTasks),
  responses(
    (status = 200, description = "The list of swarm tasks", body = ListSwarmTasksResponse),
  ),
)]
pub fn list_swarm_tasks() {}

/// List tasks on the target Swarm.
/// Response: [ListSwarmTasksResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmTasksResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmTasks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmTasksResponse = Vec<SwarmTaskListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmTask",
  description = "Inspect a Swarm task.",
  request_body(content = InspectSwarmTask),
  responses(
    (status = 200, description = "The swarm task", body = InspectSwarmTaskResponse),
  ),
)]
pub fn inspect_swarm_task() {}

/// Inspect a Swarm task.
/// Response: [SwarmTask].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmTaskResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmSecrets",
  description = "List secrets on the target Swarm.",
  request_body(content = ListSwarmSecrets),
  responses(
    (status = 200, description = "The list of swarm secrets", body = ListSwarmSecretsResponse),
  ),
)]
pub fn list_swarm_secrets() {}

/// List secrets on the target Swarm.
/// Response: [ListSwarmSecretsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmSecretsResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmSecrets {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmSecretsResponse = Vec<SwarmSecretListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmSecret",
  description = "Inspect a Swarm secret.",
  request_body(content = InspectSwarmSecret),
  responses(
    (status = 200, description = "The swarm secret", body = InspectSwarmSecretResponse),
  ),
)]
pub fn inspect_swarm_secret() {}

/// Inspect a Swarm secret.
/// Response: [SwarmSecret].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmSecretResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmConfigs",
  description = "List configs on the target Swarm.",
  request_body(content = ListSwarmConfigs),
  responses(
    (status = 200, description = "The list of swarm configs", body = ListSwarmConfigsResponse),
  ),
)]
pub fn list_swarm_configs() {}

/// List configs on the target Swarm.
/// Response: [ListSwarmConfigsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmConfigsResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmConfigs {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmConfigsResponse = Vec<SwarmConfigListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmConfig",
  description = "Inspect a config on the target Swarm.",
  request_body(content = InspectSwarmConfig),
  responses(
    (status = 200, description = "The swarm config", body = InspectSwarmConfigResponse),
  ),
)]
pub fn inspect_swarm_config() {}

/// Inspect a config on the target Swarm.
/// Response: [InspectSwarmConfigResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmConfigResponse)]
#[error(mogh_error::Error)]
pub struct InspectSwarmConfig {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Swarm config ID or Name
  pub config: String,
}

#[typeshare]
pub type InspectSwarmConfigResponse = SwarmConfig;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmStacks",
  description = "List stacks on the target Swarm.",
  request_body(content = ListSwarmStacks),
  responses(
    (status = 200, description = "The list of swarm stacks", body = ListSwarmStacksResponse),
  ),
)]
pub fn list_swarm_stacks() {}

/// List stacks on the target Swarm.
/// Response: [ListSwarmStacksResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmStacksResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmStacks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmStacksResponse = Vec<SwarmStackListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectSwarmStack",
  description = "Inspect a stack on the target Swarm.",
  request_body(content = InspectSwarmStack),
  responses(
    (status = 200, description = "The swarm stack", body = InspectSwarmStackResponse),
  ),
)]
pub fn inspect_swarm_stack() {}

/// Inspect a stack on the target Swarm.
/// Response: [SwarmStackLists].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectSwarmStackResponse)]
#[error(mogh_error::Error)]
pub struct InspectSwarmStack {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
  /// Swarm stack name
  pub stack: String,
}

#[typeshare]
pub type InspectSwarmStackResponse = SwarmStack;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSwarmNetworks",
  description = "List the networks on the swarm.",
  request_body(content = ListSwarmNetworks),
  responses(
    (status = 200, description = "The list of swarm networks", body = ListSwarmNetworksResponse),
  ),
)]
pub fn list_swarm_networks() {}

/// List the networks on the swarm. Response: [ListSwarmNetworksResponse].
///
/// This only includes the overlay networks.
/// They will be the same across all nodes in the swarm.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSwarmNetworksResponse)]
#[error(mogh_error::Error)]
pub struct ListSwarmNetworks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub swarm: String,
}

#[typeshare]
pub type ListSwarmNetworksResponse = Vec<NetworkListItem>;
