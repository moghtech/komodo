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
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RemoveSwarmSecrets {
  /// Name or id
  pub swarm: String,
  /// Secret names or ids
  pub secrets: Vec<String>,
}
