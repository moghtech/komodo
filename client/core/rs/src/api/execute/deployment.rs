use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{TerminationSignal, update::Update};

use super::{BatchExecutionResponse, KomodoExecuteRequest};

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/Deploy",
  description = "Deploys the container / swarm service for the target Deployment.",
  request_body(content = Deploy),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn deploy() {}

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
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDeploy",
  description = "Deploys multiple Deployments in parallel that match pattern.",
  request_body(content = BatchDeploy),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_deploy() {}

/// Deploys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PullDeployment",
  description = "Pulls the image for the target deployment.",
  request_body(content = PullDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pull_deployment() {}

/// Pulls the image for the target deployment. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PullDeployment {
  /// Name or id
  pub deployment: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StartDeployment",
  description = "Starts the container for the target deployment.",
  request_body(content = StartDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn start_deployment() {}

/// Starts the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker start ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StartDeployment {
  /// Name or id
  pub deployment: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RestartDeployment",
  description = "Restarts the container for the target deployment.",
  request_body(content = RestartDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn restart_deployment() {}

/// Restarts the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker restart ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RestartDeployment {
  /// Name or id
  pub deployment: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PauseDeployment",
  description = "Pauses the container for the target deployment.",
  request_body(content = PauseDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pause_deployment() {}

/// Pauses the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker pause ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PauseDeployment {
  /// Name or id
  pub deployment: String,
}

//

//
#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UnpauseDeployment",
  description = "Unpauses the container for the target deployment.",
  request_body(content = UnpauseDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn unpause_deployment() {}

/// Unpauses the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker unpause ${container_name}`.
///
/// Note. This is the only way to restart a paused container.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UnpauseDeployment {
  /// Name or id
  pub deployment: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StopDeployment",
  description = "Stops the container for the target deployment.",
  request_body(content = StopDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn stop_deployment() {}

/// Stops the container for the target deployment. Response: [Update]
///
/// 1. Runs `docker stop ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StopDeployment {
  /// Name or id
  pub deployment: String,
  /// Override the default termination signal specified in the deployment.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DestroyDeployment",
  description = "Destroys the container for the target deployment.",
  request_body(content = DestroyDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn destroy_deployment() {}

//

/// Stops and destroys the container for the target deployment.
/// Reponse: [Update].
///
/// 1. The container is stopped and removed using `docker container rm ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DestroyDeployment {
  /// Name or id.
  pub deployment: String,
  /// Override the default termination signal specified in the deployment.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDestroyDeployment",
  description = "Destroys multiple Deployments in parallel that match pattern.",
  request_body(content = BatchDestroyDeployment),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_destroy_deployment() {}

/// Destroys multiple Deployments in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
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
