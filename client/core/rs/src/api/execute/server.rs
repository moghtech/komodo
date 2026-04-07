use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{TerminationSignal, update::Update};

use super::KomodoExecuteRequest;

// =============
// = CONTAINER =
// =============

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StartContainer",
  description = "Starts the container on the target server.",
  request_body(content = StartContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn start_container() {}

/// Starts the container on the target server. Response: [Update]
///
/// 1. Runs `docker start ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StartContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RestartContainer",
  description = "Restarts the container on the target server.",
  request_body(content = RestartContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn restart_container() {}

/// Restarts the container on the target server. Response: [Update]
///
/// 1. Runs `docker restart ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RestartContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PauseContainer",
  description = "Pauses the container on the target server.",
  request_body(content = PauseContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pause_container() {}

/// Pauses the container on the target server. Response: [Update]
///
/// 1. Runs `docker pause ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PauseContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UnpauseContainer",
  description = "Unpauses the container on the target server.",
  request_body(content = UnpauseContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn unpause_container() {}

/// Unpauses the container on the target server. Response: [Update]
///
/// 1. Runs `docker unpause ${container_name}`.
///
/// Note. This is the only way to restart a paused container.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UnpauseContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StopContainer",
  description = "Stops the container on the target server.",
  request_body(content = StopContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn stop_container() {}

/// Stops the container on the target server. Response: [Update]
///
/// 1. Runs `docker stop ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StopContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
  /// Override the default termination signal.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DestroyContainer",
  description = "Stops and destroys the container on the target server.",
  request_body(content = DestroyContainer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn destroy_container() {}

/// Stops and destroys the container on the target server.
/// Reponse: [Update].
///
/// 1. The container is stopped and removed using `docker container rm ${container_name}`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DestroyContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
  /// Override the default termination signal.
  pub signal: Option<TerminationSignal>,
  /// Override the default termination max time.
  pub time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StartAllContainers",
  description = "Starts all containers on the target server.",
  request_body(content = StartAllContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn start_all_containers() {}

/// Starts all containers on the target server. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StartAllContainers {
  /// Name or id
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RestartAllContainers",
  description = "Restarts all containers on the target server.",
  request_body(content = RestartAllContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn restart_all_containers() {}

/// Restarts all containers on the target server. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RestartAllContainers {
  /// Name or id
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PauseAllContainers",
  description = "Pauses all containers on the target server.",
  request_body(content = PauseAllContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pause_all_containers() {}

/// Pauses all containers on the target server. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PauseAllContainers {
  /// Name or id
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UnpauseAllContainers",
  description = "Unpauses all containers on the target server.",
  request_body(content = UnpauseAllContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn unpause_all_containers() {}

/// Unpauses all containers on the target server. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UnpauseAllContainers {
  /// Name or id
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StopAllContainers",
  description = "Stops all containers on the target server.",
  request_body(content = StopAllContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn stop_all_containers() {}

/// Stops all containers on the target server. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StopAllContainers {
  /// Name or id
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneContainers",
  description = "Prunes the docker containers on the target server.",
  request_body(content = PruneContainers),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_containers() {}

/// Prunes the docker containers on the target server. Response: [Update].
///
/// 1. Runs `docker container prune -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneContainers {
  /// Id or name
  pub server: String,
}

// ============================
// = NETWORK / IMAGE / VOLUME =
// ============================

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteNetwork",
  description = "Delete a docker network.",
  request_body(content = DeleteNetwork),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn delete_network() {}

/// Delete a docker network.
/// Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DeleteNetwork {
  /// Id or name.
  pub server: String,
  /// The name of the network to delete.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneNetworks",
  description = "Prunes the docker networks on the target server.",
  request_body(content = PruneNetworks),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_networks() {}

/// Prunes the docker networks on the target server. Response: [Update].
///
/// 1. Runs `docker network prune -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneNetworks {
  /// Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteImage",
  description = "Delete a docker image.",
  request_body(content = DeleteImage),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn delete_image() {}

/// Delete a docker image.
/// Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DeleteImage {
  /// Id or name.
  pub server: String,
  /// The name of the image to delete.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneImages",
  description = "Prunes the docker images on the target server.",
  request_body(content = PruneImages),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_images() {}

/// Prunes the docker images on the target server. Response: [Update].
///
/// 1. Runs `docker image prune -a -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneImages {
  /// Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteVolume",
  description = "Delete a docker volume.",
  request_body(content = DeleteVolume),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn delete_volume() {}

/// Delete a docker volume.
/// Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DeleteVolume {
  /// Id or name.
  pub server: String,
  /// The name of the volume to delete.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneVolumes",
  description = "Prunes the docker volumes on the target server.",
  request_body(content = PruneVolumes),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_volumes() {}

/// Prunes the docker volumes on the target server. Response: [Update].
///
/// 1. Runs `docker volume prune -a -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneVolumes {
  /// Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneDockerBuilders",
  description = "Prunes the docker builders on the target server.",
  request_body(content = PruneDockerBuilders),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_docker_builders() {}

/// Prunes the docker builders on the target server. Response: [Update].
///
/// 1. Runs `docker builder prune -a -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneDockerBuilders {
  /// Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneBuildx",
  description = "Prunes the docker buildx cache on the target server.",
  request_body(content = PruneBuildx),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_buildx() {}

/// Prunes the docker buildx cache on the target server. Response: [Update].
///
/// 1. Runs `docker buildx prune -a -f`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneBuildx {
  /// Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PruneSystem",
  description = "Prunes the docker system on the target server, including volumes.",
  request_body(content = PruneSystem),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn prune_system() {}

/// Prunes the docker system on the target server, including volumes. Response: [Update].
///
/// 1. Runs `docker system prune -a -f --volumes`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PruneSystem {
  /// Id or name
  pub server: String,
}
