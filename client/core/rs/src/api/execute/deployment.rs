use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{TerminationSignal, update::Update};

use super::{BatchExecutionResponse, KomodoExecuteRequest};

/// Deploys the container / swarm service for the target Deployment. Response: [Update].
///
/// For Server based Deployments (just a container):
/// 1. Pulls the image onto the target server.
/// 2. If the container is already running,
/// it will be stopped and removed using `docker container rm ${container_name}`.
/// 3. The container will be run using `docker run {...params}`,
/// where params are determined by the deployment's configuration.
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
pub struct Deploy {
  /// Name or id
  pub deployment: String,
  /// Override the default termination signal specified in the deployment.
  /// Only used when deployment needs to be taken down before redeploy.
  pub stop_signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  /// Only used when deployment needs to be taken down before redeploy.
  pub stop_time: Option<i32>,
}

//

/// Deploys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse].
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
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchDeploy {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* deployments
  /// foo-*
  /// # add some more
  /// extra-deployment-1, extra-deployment-2
  /// ```
  pub pattern: String,
}

//

/// Pulls the image for the target deployment. Response: [Update]
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
pub struct PullDeployment {
  /// Name or id
  pub deployment: String,
}

//

/// Starts the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker start ${container_name}`.
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
pub struct StartDeployment {
  /// Name or id
  pub deployment: String,
}

//

/// Restarts the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker restart ${container_name}`.
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
pub struct RestartDeployment {
  /// Name or id
  pub deployment: String,
}

//

/// Pauses the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker pause ${container_name}`.
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
pub struct PauseDeployment {
  /// Name or id
  pub deployment: String,
}

//

/// Unpauses the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker unpause ${container_name}`.
///
/// Note. This is the only way to restart a paused container.
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
pub struct UnpauseDeployment {
  /// Name or id
  pub deployment: String,
}

//

/// Stops the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker stop ${container_name}`.
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
pub struct StopDeployment {
  /// Name or id
  pub deployment: String,
  /// Override the default termination signal specified in the deployment.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

/// Stops and destroys the container for the target deployment.
/// Reponse: [Update].
///
/// 1. The container is stopped and removed using `docker container rm ${container_name}`.
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
pub struct DestroyDeployment {
  /// Name or id.
  pub deployment: String,
  /// Override the default termination signal specified in the deployment.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

/// Destroys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse].
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
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchDestroyDeployment {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* deployments
  /// foo-*
  /// # add some more
  /// extra-deployment-1, extra-deployment-2
  /// ```
  pub pattern: String,
}
