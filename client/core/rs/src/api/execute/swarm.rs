use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  api::execute::KomodoExecuteRequest, entities::update::Update,
};

// ========
// = Node =
// ========

/// `docker node rm [OPTIONS] NODE [NODE...]`
///
/// https://docs.docker.com/reference/cli/docker/node/rm/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
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

// =========
// = Stack =
// =========

/// `docker stack rm [OPTIONS] STACK [STACK...]`
///
/// https://docs.docker.com/reference/cli/docker/stack/rm/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
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

/// `docker service rm SERVICE [SERVICE...]`
///
/// https://docs.docker.com/reference/cli/docker/service/rm/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RemoveSwarmServices {
  /// Name or id
  pub swarm: String,
  /// Service names or ids
  pub services: Vec<String>,
}

// ==========
// = Config =
// ==========

/// `docker config create [OPTIONS] CONFIG file|-`
///
/// https://docs.docker.com/reference/cli/docker/config/create/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
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
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RotateSwarmConfig {
  /// Name or id
  pub swarm: String,
  /// Config name
  pub config: String,
  /// The new config data as a string
  pub data: String,
}

//

/// `docker config rm CONFIG [CONFIG...]`
///
/// https://docs.docker.com/reference/cli/docker/config/rm/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RemoveSwarmConfigs {
  /// Name or id
  pub swarm: String,
  /// Config names or ids
  pub configs: Vec<String>,
}

// ==========
// = Secret =
// ==========

/// `docker config create [OPTIONS] CONFIG file|-`
///
/// https://docs.docker.com/reference/cli/docker/config/create/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
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
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RotateSwarmSecret {
  /// Name or id
  pub swarm: String,
  /// Secret name
  pub secret: String,
  /// The new secret data as a string
  pub data: String,
}

/// `docker secret rm SECRET [SECRET...]`
///
/// https://docs.docker.com/reference/cli/docker/secret/rm/
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RemoveSwarmSecrets {
  /// Name or id
  pub swarm: String,
  /// Secret names or ids
  pub secrets: Vec<String>,
}
