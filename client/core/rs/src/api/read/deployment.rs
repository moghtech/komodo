use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  I64, SearchCombinator, U64,
  deployment::{
    Deployment, DeploymentActionState, DeploymentListItem,
    DeploymentQuery, DeploymentState,
  },
  docker::{
    container::{Container, ContainerListItem, ContainerStats},
    service::SwarmService,
  },
  update::Log,
};

use super::KomodoReadRequest;

//

/// Get a specific deployment by name or id. Response: [Deployment].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetDeploymentResponse)]
#[error(serror::Error)]
pub struct GetDeployment {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

#[typeshare]
pub type GetDeploymentResponse = Deployment;

//

/// List deployments matching optional query.
/// Response: [ListDeploymentsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListDeploymentsResponse)]
#[error(serror::Error)]
pub struct ListDeployments {
  /// optional structured query to filter deployments.
  #[serde(default)]
  pub query: DeploymentQuery,
}

#[typeshare]
pub type ListDeploymentsResponse = Vec<DeploymentListItem>;

//

/// List deployments matching optional query.
/// Response: [ListFullDeploymentsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullDeploymentsResponse)]
#[error(serror::Error)]
pub struct ListFullDeployments {
  /// optional structured query to filter deployments.
  #[serde(default)]
  pub query: DeploymentQuery,
}

#[typeshare]
pub type ListFullDeploymentsResponse = Vec<Deployment>;

//

/// Get the container, including image / status, of the target deployment.
/// Response: [GetDeploymentContainerResponse].
///
/// Note. This does not hit the server directly. The status comes from an
/// in memory cache on the core, which hits the server periodically
/// to keep it up to date.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetDeploymentContainerResponse)]
#[error(serror::Error)]
pub struct GetDeploymentContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

/// Response for [GetDeploymentContainer].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDeploymentContainerResponse {
  pub state: DeploymentState,
  pub container: Option<ContainerListItem>,
}

//

/// Inspect the docker container associated with the Deployment.
/// Response: [Container].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDeploymentContainerResponse)]
#[error(serror::Error)]
pub struct InspectDeploymentContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

#[typeshare]
pub type InspectDeploymentContainerResponse = Container;

//

/// Inspect the swarm service associated with the Deployment.
/// Response: [SwarmService].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDeploymentSwarmServiceResponse)]
#[error(serror::Error)]
pub struct InspectDeploymentSwarmService {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

#[typeshare]
pub type InspectDeploymentSwarmServiceResponse = SwarmService;

//

/// Get the deployment log's tail, split by stdout/stderr.
/// Response: [Log].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetDeploymentLogResponse)]
#[error(serror::Error)]
pub struct GetDeploymentLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
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
pub type GetDeploymentLogResponse = Log;

//

/// Search the deployment log's tail using `grep`. All lines go to stdout.
/// Response: [Log].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(SearchDeploymentLogResponse)]
#[error(serror::Error)]
pub struct SearchDeploymentLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
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
pub type SearchDeploymentLogResponse = Log;

//

/// Get the deployment container's stats using `docker stats`.
/// Response: [GetDeploymentStatsResponse].
///
/// Note. This call will hit the underlying server directly for most up to date stats.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetDeploymentStatsResponse)]
#[error(serror::Error)]
pub struct GetDeploymentStats {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

#[typeshare]
pub type GetDeploymentStatsResponse = ContainerStats;

//

/// Get current action state for the deployment.
/// Response: [DeploymentActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(DeploymentActionState)]
#[error(serror::Error)]
pub struct GetDeploymentActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub deployment: String,
}

#[typeshare]
pub type GetDeploymentActionStateResponse = DeploymentActionState;

//

/// Gets a summary of data relating to all deployments.
/// Response: [GetDeploymentsSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetDeploymentsSummaryResponse)]
#[error(serror::Error)]
pub struct GetDeploymentsSummary {}

/// Response for [GetDeploymentsSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetDeploymentsSummaryResponse {
  /// The total number of Deployments
  pub total: I64,
  /// The number of Deployments with Running state
  pub running: I64,
  /// The number of Deployments with Stopped or Paused state
  pub stopped: I64,
  /// The number of Deployments with NotDeployed state
  pub not_deployed: I64,
  /// The number of Deployments with Restarting or Dead or Created (other) state
  pub unhealthy: I64,
  /// The number of Deployments with Unknown state
  pub unknown: I64,
}

//

/// Gets a list of existing values used as extra args across other deployments.
/// Useful to offer suggestions. Response: [ListCommonDeploymentExtraArgsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonDeploymentExtraArgsResponse)]
#[error(serror::Error)]
pub struct ListCommonDeploymentExtraArgs {
  /// optional structured query to filter deployments.
  #[serde(default)]
  pub query: DeploymentQuery,
}

#[typeshare]
pub type ListCommonDeploymentExtraArgsResponse = Vec<String>;
