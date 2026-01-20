use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  SearchCombinator, U64,
  docker::{
    container::Container, service::SwarmService, stack::SwarmStack,
  },
  stack::{
    Stack, StackActionState, StackListItem, StackQuery, StackService,
  },
  update::Log,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetStack",
  description = "Get a specific stack.",
  request_body(content = GetStack),
  responses(
    (status = 200, description = "The stack", body = crate::entities::stack::StackSchema),
  ),
)]
pub fn get_stack() {}

/// Get a specific stack. Response: [Stack].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackResponse)]
#[error(mogh_error::Error)]
pub struct GetStack {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type GetStackResponse = Stack;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListStackServices",
  description = "Lists a specific stacks services (the containers).",
  request_body(content = ListStackServices),
  responses(
    (status = 200, description = "The list of services", body = ListStackServicesResponse),
  ),
)]
pub fn list_stack_services() {}

/// Lists a specific stacks services (the containers). Response: [ListStackServicesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListStackServicesResponse)]
#[error(mogh_error::Error)]
pub struct ListStackServices {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type ListStackServicesResponse = Vec<StackService>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/InspectStackContainer",
  description = "Inspect a docker container associated with a Stack.",
  request_body(content = InspectStackContainer),
  responses(
    (status = 200, description = "The container", body = InspectStackContainerResponse),
  ),
)]
pub fn inspect_stack_container() {}

/// Inspect a docker container associated with a Stack.
/// Response: [Container].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectStackContainerResponse)]
#[error(mogh_error::Error)]
pub struct InspectStackContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
  /// The service name to inspect
  pub service: String,
}

#[typeshare]
pub type InspectStackContainerResponse = Container;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/InspectStackSwarmService",
  description = "Inspect a swarm service associated with a Stack.",
  request_body(content = InspectStackSwarmService),
  responses(
    (status = 200, description = "The swarm service", body = InspectStackSwarmServiceResponse),
  ),
)]
pub fn inspect_stack_swarm_service() {}

/// Inspect a swarm service associated with a Stack.
/// Response: [SwarmService].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectStackSwarmServiceResponse)]
#[error(mogh_error::Error)]
pub struct InspectStackSwarmService {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
  /// The service name to inspect
  pub service: String,
}

#[typeshare]
pub type InspectStackSwarmServiceResponse = SwarmService;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/InspectStackSwarmInfo",
  description = "Inspect swarm info associated with a Stack.",
  request_body(content = InspectStackSwarmInfo),
  responses(
    (status = 200, description = "The swarm info", body = InspectStackSwarmInfoResponse),
  ),
)]
pub fn inspect_stack_swarm_info() {}

/// Inspect swarm info associated with a Stack.
/// Response: [SwarmStack].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectStackSwarmInfoResponse)]
#[error(mogh_error::Error)]
pub struct InspectStackSwarmInfo {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type InspectStackSwarmInfoResponse = SwarmStack;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetStackLog",
  description = "Get a stack's logs. Filter down included services.",
  request_body(content = GetStackLog),
  responses(
    (status = 200, description = "The stack log", body = GetStackLogResponse),
  ),
)]
pub fn get_stack_log() {}

/// Get a stack's logs. Filter down included services. Response: [GetStackLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackLogResponse)]
#[error(mogh_error::Error)]
pub struct GetStackLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
  /// Filter the logs to only ones from specific services.
  /// If empty, will include logs from all services.
  pub services: Vec<String>,
  /// The number of lines of the log tail to include.
  /// Default: 100.
  /// Max: 5000.
  #[serde(default = "default_tail")]
  pub tail: U64,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
}

fn default_tail() -> u64 {
  50
}

#[typeshare]
pub type GetStackLogResponse = Log;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/SearchStackLog",
  description = "Search the stack log's tail using `grep`.",
  request_body(content = SearchStackLog),
  responses(
    (status = 200, description = "The search results", body = SearchStackLogResponse),
  ),
)]
pub fn search_stack_log() {}

/// Search the stack log's tail using `grep`. All lines go to stdout.
/// Response: [SearchStackLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(SearchStackLogResponse)]
#[error(mogh_error::Error)]
pub struct SearchStackLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
  /// Filter the logs to only ones from specific services.
  /// If empty, will include logs from all services.
  pub services: Vec<String>,
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
}

#[typeshare]
pub type SearchStackLogResponse = Log;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListCommonStackExtraArgs",
  description = "Gets a list of existing values used as extra args across other stacks.",
  request_body(content = ListCommonStackExtraArgs),
  responses(
    (status = 200, description = "The list of extra args", body = ListCommonStackExtraArgsResponse),
  ),
)]
pub fn list_common_stack_extra_args() {}

/// Gets a list of existing values used as extra args across other stacks.
/// Useful to offer suggestions. Response: [ListCommonStackExtraArgsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonStackExtraArgsResponse)]
#[error(mogh_error::Error)]
pub struct ListCommonStackExtraArgs {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListCommonStackExtraArgsResponse = Vec<String>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListCommonStackBuildExtraArgs",
  description = "Gets a list of existing values used as build extra args across other stacks.",
  request_body(content = ListCommonStackBuildExtraArgs),
  responses(
    (status = 200, description = "The list of build extra args", body = ListCommonStackBuildExtraArgsResponse),
  ),
)]
pub fn list_common_stack_build_extra_args() {}

/// Gets a list of existing values used as build extra args across other stacks.
/// Useful to offer suggestions. Response: [ListCommonStackBuildExtraArgsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonStackBuildExtraArgsResponse)]
#[error(mogh_error::Error)]
pub struct ListCommonStackBuildExtraArgs {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListCommonStackBuildExtraArgsResponse = Vec<String>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListStacks",
  description = "List stacks matching optional query.",
  request_body(content = ListStacks),
  responses(
    (status = 200, description = "The list of stacks", body = ListStacksResponse),
  ),
)]
pub fn list_stacks() {}

/// List stacks matching optional query. Response: [ListStacksResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListStacksResponse)]
#[error(mogh_error::Error)]
pub struct ListStacks {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListStacksResponse = Vec<StackListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListFullStacks",
  description = "List stacks matching optional query.",
  request_body(content = ListFullStacks),
  responses(
    (status = 200, description = "The list of stacks", body = ListFullStacksResponse),
  ),
)]
pub fn list_full_stacks() {}

/// List stacks matching optional query. Response: [ListFullStacksResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullStacksResponse)]
#[error(mogh_error::Error)]
pub struct ListFullStacks {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListFullStacksResponse = Vec<Stack>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetStackActionState",
  description = "Get current action state for the stack.",
  request_body(content = GetStackActionState),
  responses(
    (status = 200, description = "The stack action state", body = GetStackActionStateResponse),
  ),
)]
pub fn get_stack_action_state() {}

/// Get current action state for the stack. Response: [StackActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetStackActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type GetStackActionStateResponse = StackActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetStacksSummary",
  description = "Gets a summary of data relating to all syncs.",
  request_body(content = GetStacksSummary),
  responses(
    (status = 200, description = "The stacks summary", body = GetStacksSummaryResponse),
  ),
)]
pub fn get_stacks_summary() {}

/// Gets a summary of data relating to all syncs.
/// Response: [GetStacksSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetStacksSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetStacksSummary {}

/// Response for [GetStacksSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetStacksSummaryResponse {
  /// The total number of stacks
  pub total: u32,
  /// The number of stacks with Running state.
  pub running: u32,
  /// The number of stacks with Stopped or Paused state.
  pub stopped: u32,
  /// The number of stacks with Down state.
  pub down: u32,
  /// The number of stacks with Unhealthy or Restarting or Dead or Created or Removing state.
  pub unhealthy: u32,
  /// The number of stacks with Unknown state.
  pub unknown: u32,
}
