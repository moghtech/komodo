use std::collections::HashMap;

use komodo_client::entities::{
  SearchCombinator,
  deployment::Deployment,
  docker::{
    SwarmLists,
    config::SwarmConfig,
    node::{NodeSpecAvailabilityEnum, NodeSpecRoleEnum, SwarmNode},
    secret::SwarmSecret,
    service::SwarmService,
    stack::SwarmStack,
    swarm::SwarmInspectInfo,
    task::SwarmTask,
  },
  repo::Repo,
  stack::Stack,
  update::Log,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

use crate::api::DeployStackResponse;

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(PollSwarmStatusResponse)]
#[error(anyhow::Error)]
pub struct PollSwarmStatus {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollSwarmStatusResponse {
  /// Inspect swarm response
  pub inspect: Option<SwarmInspectInfo>,
  pub lists: SwarmLists,
}

// ======
//  Node
// ======

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(SwarmNode)]
#[error(anyhow::Error)]
pub struct InspectSwarmNode {
  pub node: String,
}

/// `docker node rm [OPTIONS] NODE [NODE...]`
///
/// https://docs.docker.com/reference/cli/docker/node/rm/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct RemoveSwarmNodes {
  pub nodes: Vec<String>,
  pub force: bool,
}

/// `docker node update [OPTIONS] NODE`
///
/// https://docs.docker.com/reference/cli/docker/node/update/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct UpdateSwarmNode {
  pub node: String,
  pub availability: Option<NodeSpecAvailabilityEnum>,
  /// Add metadata to a swarm node using node labels (`key=value`).
  /// You can specify a node label as a key with an empty value.
  pub label_add: Option<HashMap<String, Option<String>>>,
  /// Remove labels by the label key.
  pub label_rm: Option<Vec<String>>,
  /// Update the node role (`worker`, `manager`)
  pub role: Option<NodeSpecRoleEnum>,
}

// =======
//  Stack
// =======

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(SwarmStack)]
#[error(anyhow::Error)]
pub struct InspectSwarmStack {
  /// The swarm stack name
  pub stack: String,
}

/// `docker stack deploy [OPTIONS] STACK`
///
/// https://docs.docker.com/reference/cli/docker/stack/deploy/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(DeployStackResponse)]
#[error(anyhow::Error)]
pub struct DeploySwarmStack {
  /// The stack to deploy
  pub stack: Stack,
  /// The linked repo, if it exists.
  pub repo: Option<Repo>,
  /// If provided, use it to login in. Otherwise check periphery local registries.
  pub git_token: Option<String>,
  /// If provided, use it to login in. Otherwise check periphery local git providers.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

/// `docker stack rm [OPTIONS] STACK [STACK...]`
///
/// https://docs.docker.com/reference/cli/docker/stack/rm/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct RemoveSwarmStacks {
  pub stacks: Vec<String>,
  /// Do not wait for stack removal
  pub detach: bool,
}

// =========
//  Service
// =========

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(SwarmService)]
#[error(anyhow::Error)]
pub struct InspectSwarmService {
  pub service: String,
}

/// Get a swarm service's logs.
///
/// https://docs.docker.com/reference/cli/docker/service/logs/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct GetSwarmServiceLog {
  /// The name or id of service.
  /// Also accepts task id.
  pub service: String,
  /// Pass `--tail` for only recent log contents. Max of 5000
  #[serde(default = "default_tail")]
  pub tail: u64,
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

//

/// Search a swarm service's logs.
///
/// https://docs.docker.com/reference/cli/docker/service/logs/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct GetSwarmServiceLogSearch {
  /// The name or id of service.
  /// Also accepts task id.
  pub service: String,
  /// The search terms.
  pub terms: Vec<String>,
  /// And: Only lines matching all terms
  /// Or: Lines matching any one of the terms
  #[serde(default)]
  pub combinator: SearchCombinator,
  /// Invert the search (search for everything not matching terms)
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

/// `docker service create [OPTIONS] IMAGE [COMMAND] [ARG...]`
///
/// https://docs.docker.com/reference/cli/docker/service/create/
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(anyhow::Error)]
pub struct CreateSwarmService {
  pub deployment: Deployment,
  /// Override registry token with one sent from core.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

/// `docker service update [OPTIONS] SERVICE`
///
/// https://docs.docker.com/reference/cli/docker/service/create/
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct UpdateSwarmService {
  /// Th service name
  pub service: String,
  /// The image may require login to another registry
  pub registry_account: Option<String>,
  pub registry_token: Option<String>,
  pub image: Option<String>,
  pub replicas: Option<u32>,
  pub rollback: bool,
  pub extra_args: Vec<String>,
}

/// `docker service rm SERVICE [SERVICE...]`
///
/// https://docs.docker.com/reference/cli/docker/service/rm/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct RemoveSwarmServices {
  pub services: Vec<String>,
}

// ======
//  Task
// ======

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(SwarmTask)]
#[error(anyhow::Error)]
pub struct InspectSwarmTask {
  pub task: String,
}

// ========
//  Config
// ========

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Vec<SwarmConfig>)]
#[error(anyhow::Error)]
pub struct InspectSwarmConfig {
  pub config: String,
}

//

/// `docker config rm CONFIG [CONFIG...]`
///
/// https://docs.docker.com/reference/cli/docker/config/rm/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct RemoveSwarmConfigs {
  pub configs: Vec<String>,
}

//

/// https://docs.docker.com/engine/swarm/configs/#example-rotate-a-config
///
/// Swarm configs / secrets are immutable after creation.
/// This making updating values awkward when you have services actively using them.
/// The following steps allows for config rotation while minimizing downtime.
///
/// 1. Query for all services using the config
///    - If not in use by any services, can simply `remove` and `create` the config.
///    - Otherwise, continue with following steps
/// 2. `Create` config `{config}-tmp` using provided data
/// 3. `Update` services to use `tmp` config
/// 4. `Remove` and `create` the actual config. This is now possible because services are using the tmp config.
/// 5. `Update` services to use actual (not `tmp`) config again.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Vec<Log>)]
#[error(anyhow::Error)]
pub struct RotateSwarmConfig {
  /// Config name
  pub config: String,
  /// The config file data as a string
  pub data: String,
}

// ========
//  Secret
// ========

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(SwarmSecret)]
#[error(anyhow::Error)]
pub struct InspectSwarmSecret {
  pub secret: String,
}

/// `docker secret rm SECRET [SECRET...]`
///
/// https://docs.docker.com/reference/cli/docker/secret/rm/
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct RemoveSwarmSecrets {
  pub secrets: Vec<String>,
}
