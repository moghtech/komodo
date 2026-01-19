use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ResourceTarget, SearchCombinator, U64,
  docker::{
    container::{Container, ContainerListItem},
    image::{Image, ImageHistoryResponseItem, ImageListItem},
    network::{Network, NetworkListItem},
    volume::{Volume, VolumeListItem},
  },
  stack::ComposeProject,
  update::Log,
};

use super::KomodoReadRequest;

//

/// Gets a summary of data relating to all containers.
/// Response: [GetDockerContainersSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetDockerContainersSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetDockerContainersSummary {}

/// Response for [GetDockerContainersSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetDockerContainersSummaryResponse {
  /// The total number of Containers
  pub total: u32,
  /// The number of Containers with Running state
  pub running: u32,
  /// The number of Containers with Stopped or Paused or Created state
  pub stopped: u32,
  /// The number of Containers with Restarting or Dead state
  pub unhealthy: u32,
  /// The number of Containers with Unknown state
  pub unknown: u32,
}

//

/// List all docker containers on the target servers.
/// Response: [ListDockerContainersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListAllDockerContainersResponse)]
#[error(mogh_error::Error)]
pub struct ListAllDockerContainers {
  /// Filter by server id or name.
  #[serde(default)]
  pub servers: Vec<String>,

  /// Filter by container name.
  #[serde(default)]
  pub containers: Vec<String>,
}

#[typeshare]
pub type ListAllDockerContainersResponse = Vec<ContainerListItem>;

//

/// List all docker containers on the target server.
/// Response: [ListDockerContainersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerContainersResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerContainers {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListDockerContainersResponse = Vec<ContainerListItem>;

//

/// Inspect a docker container on the server. Response: [Container].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDockerContainerResponse)]
#[error(mogh_error::Error)]
pub struct InspectDockerContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The container name
  pub container: String,
}

#[typeshare]
pub type InspectDockerContainerResponse = Container;

//

/// Find the attached resource for a container. Either Deployment or Stack. Response: [GetResourceMatchingContainerResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetResourceMatchingContainerResponse)]
#[error(mogh_error::Error)]
pub struct GetResourceMatchingContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The container name
  pub container: String,
}

/// Response for [GetResourceMatchingContainer]. Resource is either Deployment, Stack, or None.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetResourceMatchingContainerResponse {
  pub resource: Option<ResourceTarget>,
}

//

/// Get the container log's tail, split by stdout/stderr.
/// Response: [Log].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetContainerLogResponse)]
#[error(mogh_error::Error)]
pub struct GetContainerLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The container name
  pub container: String,
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
pub type GetContainerLogResponse = Log;

//

/// Search the container log's tail using `grep`. All lines go to stdout.
/// Response: [Log].
///
/// Note. This call will hit the underlying server directly for most up to date log.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(SearchContainerLogResponse)]
#[error(mogh_error::Error)]
pub struct SearchContainerLog {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The container name
  pub container: String,
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
pub type SearchContainerLogResponse = Log;

//

/// List all docker compose projects on the target server.
/// Response: [ListComposeProjectsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListComposeProjectsResponse)]
#[error(mogh_error::Error)]
pub struct ListComposeProjects {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListComposeProjectsResponse = Vec<ComposeProject>;

//

/// List the docker networks on the server. Response: [ListDockerNetworksResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerNetworksResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerNetworks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListDockerNetworksResponse = Vec<NetworkListItem>;

//

/// Inspect a docker network on the server. Response: [InspectDockerNetworkResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDockerNetworkResponse)]
#[error(mogh_error::Error)]
pub struct InspectDockerNetwork {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The network name
  pub network: String,
}

#[typeshare]
pub type InspectDockerNetworkResponse = Network;

//

/// List the docker images locally cached on the target server.
/// Response: [ListDockerImagesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerImagesResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerImages {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListDockerImagesResponse = Vec<ImageListItem>;

//

/// Inspect a docker image on the server. Response: [Image].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDockerImageResponse)]
#[error(mogh_error::Error)]
pub struct InspectDockerImage {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The image name
  pub image: String,
}

#[typeshare]
pub type InspectDockerImageResponse = Image;

//

/// Get image history from the server. Response: [ListDockerImageHistoryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerImageHistoryResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerImageHistory {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The image name
  pub image: String,
}

#[typeshare]
pub type ListDockerImageHistoryResponse =
  Vec<ImageHistoryResponseItem>;

//

/// List all docker volumes on the target server.
/// Response: [ListDockerVolumesResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerVolumesResponse)]
#[error(mogh_error::Error)]
pub struct ListDockerVolumes {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListDockerVolumesResponse = Vec<VolumeListItem>;

//

/// Inspect a docker volume on the server. Response: [Volume].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectDockerVolumeResponse)]
#[error(mogh_error::Error)]
pub struct InspectDockerVolume {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The volume name
  pub volume: String,
}

#[typeshare]
pub type InspectDockerVolumeResponse = Volume;
