use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  api::execute::KomodoExecuteRequest,
  entities::{
    docker::node::{NodeSpecAvailabilityEnum, NodeSpecRoleEnum},
    update::Update,
  },
};

// ========
// = Node =
// ========

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveSwarmNodes",
  description = "Remove swarm nodes.",
  request_body(content = RemoveSwarmNodes),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn remove_swarm_nodes() {}

/// `docker node rm [OPTIONS] NODE [NODE...]`
///
/// https://docs.docker.com/reference/cli/docker/node/rm/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RemoveSwarmNodes {
  /// Name or id
  pub swarm: String,
  /// Node names or ids to remove
  pub nodes: Vec<String>,
  /// Force remove a node from the swarm
  #[serde(default)]
  #[arg(long, short, default_value_t = false)]
  pub force: bool,
}

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateSwarmNode",
  description = "Update a swarm node's availability, labels, and role.",
  request_body(content = UpdateSwarmNode),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn update_swarm_node() {}

/// `docker node update [OPTIONS] NODE`
///
/// https://docs.docker.com/reference/cli/docker/node/update/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UpdateSwarmNode {
  /// Name or id
  pub swarm: String,
  /// Node hostname or id
  pub node: String,
  /// Update the node's availability: 'active', 'pause', or 'drain'
  #[arg(long, short = 'a')]
  pub availability: Option<NodeSpecAvailabilityEnum>,
  /// Add labels to node (`key=value`).
  #[arg(long, short = 'l')]
  pub label_add: Option<Vec<String>>,
  /// Add labels to node (`key=value`). (alias: `lr`)
  #[arg(long, alias = "lr")]
  pub label_rm: Option<Vec<String>>,
  /// Update the node's role: 'worker' or 'manager'
  #[arg(long, short = 'r')]
  pub role: Option<NodeSpecRoleEnum>,
}

// =========
// = Stack =
// =========

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveSwarmStacks",
  description = "Remove swarm stacks.",
  request_body(content = RemoveSwarmStacks),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn remove_swarm_stacks() {}

/// `docker stack rm [OPTIONS] STACK [STACK...]`
///
/// https://docs.docker.com/reference/cli/docker/stack/rm/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RemoveSwarmStacks {
  /// Name or id
  pub swarm: String,
  /// Node names to remove
  pub stacks: Vec<String>,
  /// Do not wait for stack removal
  #[serde(default = "default_detach")]
  #[arg(long, short, default_value_t = default_detach())]
  pub detach: bool,
}

fn default_detach() -> bool {
  true
}

// ===========
// = Service =
// ===========

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveSwarmServices",
  description = "Remove swarm services.",
  request_body(content = RemoveSwarmServices),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn remove_swarm_services() {}

/// `docker service rm SERVICE [SERVICE...]`
///
/// https://docs.docker.com/reference/cli/docker/service/rm/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RemoveSwarmServices {
  /// Name or id
  pub swarm: String,
  /// Service names or ids
  pub services: Vec<String>,
}

// ==========
// = Config =
// ==========

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateSwarmConfig",
  description = "Create a swarm config.",
  request_body(content = CreateSwarmConfig),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn create_swarm_config() {}

/// `docker config create [OPTIONS] CONFIG file|-`
///
/// https://docs.docker.com/reference/cli/docker/config/create/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CreateSwarmConfig {
  /// Name or id
  pub swarm: String,
  /// The name of the config to create
  pub name: String,
  /// The data to store in the config
  pub data: String,
  /// Docker labels to give the config
  #[serde(default)]
  pub labels: Vec<String>,
  /// Optional custom template driver
  pub template_driver: Option<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RotateSwarmConfig",
  description = "Rotate a swarm config.",
  request_body(content = RotateSwarmConfig),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rotate_swarm_config() {}

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
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RotateSwarmConfig {
  /// Name or id
  pub swarm: String,
  /// Config name
  pub config: String,
  /// The new config data as a string
  pub data: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveSwarmConfigs",
  description = "Remove swarm configs.",
  request_body(content = RemoveSwarmConfigs),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn remove_swarm_configs() {}

/// `docker config rm CONFIG [CONFIG...]`
///
/// https://docs.docker.com/reference/cli/docker/config/rm/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RemoveSwarmConfigs {
  /// Name or id
  pub swarm: String,
  /// Config names or ids
  pub configs: Vec<String>,
}

// ==========
// = Secret =
// ==========

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateSwarmSecret",
  description = "Create a swarm secret.",
  request_body(content = CreateSwarmSecret),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn create_swarm_secret() {}

/// `docker config create [OPTIONS] CONFIG file|-`
///
/// https://docs.docker.com/reference/cli/docker/config/create/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CreateSwarmSecret {
  /// Name or id
  pub swarm: String,
  /// The name of the secret to create
  pub name: String,
  /// The data to store in the secret
  pub data: String,
  /// Optional custom secret driver
  pub driver: Option<String>,
  /// Docker labels to give the secret
  #[serde(default)]
  pub labels: Vec<String>,
  /// Optional custom template driver
  pub template_driver: Option<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RotateSwarmSecret",
  description = "Rotate a swarm secret.",
  request_body(content = RotateSwarmSecret),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rotate_swarm_secret() {}

/// https://docs.docker.com/engine/swarm/secrets/#example-rotate-a-secret
///
/// Swarm configs / secrets are immutable after creation.
/// This making updating values awkward when you have services actively using them.
/// The following steps allows for secret rotation while minimizing downtime.
///
/// 1. Query for all services using the secret
///    - If not in use by any services, can simply `remove` and `create` the secret.
///    - Otherwise, continue with following steps
/// 2. `Create` secret `{secret}-tmp` using provided data
/// 3. `Update` services to use `tmp` secret
/// 4. `Remove` and `create` the actual secret. This is now possible because services are using the tmp secret.
/// 5. `Update` services to use actual (not `tmp`) secret again.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RotateSwarmSecret {
  /// Name or id
  pub swarm: String,
  /// Secret name
  pub secret: String,
  /// The new secret data as a string
  pub data: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RemoveSwarmSecrets",
  description = "Remove swarm secrets.",
  request_body(content = RemoveSwarmSecrets),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn remove_swarm_secrets() {}

/// `docker secret rm SECRET [SECRET...]`
///
/// https://docs.docker.com/reference/cli/docker/secret/rm/
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RemoveSwarmSecrets {
  /// Name or id
  pub swarm: String,
  /// Secret names or ids
  pub secrets: Vec<String>,
}
