use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  SearchCombinator, U64,
  docker::{container::Container, service::SwarmService},
  stack::{
    Stack, StackActionState, StackListItem, StackQuery, StackService,
  },
  update::Log,
};

use super::KomodoReadRequest;

//

/// Get a specific stack. Response: [Stack].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackResponse)]
#[error(serror::Error)]
pub struct GetStack {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type GetStackResponse = Stack;

//

/// Lists a specific stacks services (the containers). Response: [ListStackServicesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListStackServicesResponse)]
#[error(serror::Error)]
pub struct ListStackServices {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type ListStackServicesResponse = Vec<StackService>;

//

/// Inspect a docker container associated with a Stack.
/// Response: [Container].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectStackContainerResponse)]
#[error(serror::Error)]
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

/// Inspect a swarm service associated with a Stack.
/// Response: [Container].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectStackSwarmServiceResponse)]
#[error(serror::Error)]
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

/// Get a stack's logs. Filter down included services. Response: [GetStackLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackLogResponse)]
#[error(serror::Error)]
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

/// Search the stack log's tail using `grep`. All lines go to stdout.
/// Response: [SearchStackLogResponse].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(SearchStackLogResponse)]
#[error(serror::Error)]
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

/// Gets a list of existing values used as extra args across other stacks.
/// Useful to offer suggestions. Response: [ListCommonStackExtraArgsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonStackExtraArgsResponse)]
#[error(serror::Error)]
pub struct ListCommonStackExtraArgs {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListCommonStackExtraArgsResponse = Vec<String>;

//

/// Gets a list of existing values used as build extra args across other stacks.
/// Useful to offer suggestions. Response: [ListCommonStackBuildExtraArgsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonStackBuildExtraArgsResponse)]
#[error(serror::Error)]
pub struct ListCommonStackBuildExtraArgs {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListCommonStackBuildExtraArgsResponse = Vec<String>;

//

/// List stacks matching optional query. Response: [ListStacksResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListStacksResponse)]
#[error(serror::Error)]
pub struct ListStacks {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListStacksResponse = Vec<StackListItem>;

//

/// List stacks matching optional query. Response: [ListFullStacksResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullStacksResponse)]
#[error(serror::Error)]
pub struct ListFullStacks {
  /// optional structured query to filter stacks.
  #[serde(default)]
  pub query: StackQuery,
}

#[typeshare]
pub type ListFullStacksResponse = Vec<Stack>;

//

/// Get current action state for the stack. Response: [StackActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetStackActionStateResponse)]
#[error(serror::Error)]
pub struct GetStackActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

#[typeshare]
pub type GetStackActionStateResponse = StackActionState;

//

/// Gets a summary of data relating to all syncs.
/// Response: [GetStacksSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetStacksSummaryResponse)]
#[error(serror::Error)]
pub struct GetStacksSummary {}

/// Response for [GetStacksSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
