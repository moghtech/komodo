use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{update::Update, TerminationSignal};

use super::KomodoExecuteRequest;

// =============
// = CONTAINER =
// =============

/// Starts the container on the target server. Response: [Update]
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
pub struct StartContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

/// Restarts the container on the target server. Response: [Update]
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
pub struct RestartContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

/// Pauses the container on the target server. Response: [Update]
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
pub struct PauseContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

/// Unpauses the container on the target server. Response: [Update]
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
pub struct UnpauseContainer {
  /// Name or id
  pub server: String,
  /// The container name
  pub container: String,
}

//

/// Stops the container on the target server. Response: [Update]
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

/// Stops and destroys the container on the target server.
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

/// Starts all containers on the target server. Response: [Update]
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
pub struct StartAllContainers {
  /// Name or id
  pub server: String,
}

//

/// Restarts all containers on the target server. Response: [Update]
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
pub struct RestartAllContainers {
  /// Name or id
  pub server: String,
}

//

/// Pauses all containers on the target server. Response: [Update]
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
pub struct PauseAllContainers {
  /// Name or id
  pub server: String,
}

//

/// Unpauses all containers on the target server. Response: [Update]
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
pub struct UnpauseAllContainers {
  /// Name or id
  pub server: String,
}

//

/// Stops all containers on the target server. Response: [Update]
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
pub struct StopAllContainers {
  /// Name or id
  pub server: String,
}

//

/// Prunes the docker containers on the target server. Response: [Update].
///
/// 1. Runs `docker container prune -f`.
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
pub struct PruneContainers {
  /// Id or name
  pub server: String,
}

// ============================
// = NETWORK / IMAGE / VOLUME =
// ============================

/// Delete a docker network.
/// Response: [Update]
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
pub struct DeleteNetwork {
  /// Id or name.
  pub server: String,
  /// The name of the network to delete.
  pub name: String,
}

//

/// Prunes the docker networks on the target server. Response: [Update].
///
/// 1. Runs `docker network prune -f`.
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
pub struct PruneNetworks {
  /// Id or name
  pub server: String,
}

//

/// Delete a docker image.
/// Response: [Update]
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
pub struct DeleteImage {
  /// Id or name.
  pub server: String,
  /// The name of the image to delete.
  pub name: String,
}

//

/// Prunes the docker images on the target server. Response: [Update].
///
/// 1. Runs `docker image prune -a -f`.
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
pub struct PruneImages {
  /// Id or name
  pub server: String,
}

//

/// Delete a docker volume.
/// Response: [Update]
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
pub struct DeleteVolume {
  /// Id or name.
  pub server: String,
  /// The name of the volume to delete.
  pub name: String,
}

//

/// Prunes the docker volumes on the target server. Response: [Update].
///
/// 1. Runs `docker volume prune -a -f`.
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
pub struct PruneVolumes {
  /// Id or name
  pub server: String,
}

//

/// Prunes the docker builders (build cache) on the target server. Response: [Update].
///
/// 1. Runs `docker builder prune -a -f`.
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
pub struct PruneDockerBuilders {
  /// Id or name
  pub server: String,
}

//

/// Prunes the docker buildx cache on the target server. Response: [Update].
///
/// 1. Runs `docker buildx prune -a -f`.
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
pub struct PruneBuildx {
  /// Id or name
  pub server: String,
}

//

/// Prunes the docker system on the target server, including volumes. Response: [Update].
///
/// 1. Runs `docker system prune -a -f --volumes`.
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
pub struct PruneSystem {
  /// Id or name
  pub server: String,
}
