use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  ResourceTarget, SearchCombinator, U64,
  docker::{
    container::{
      Container, ContainerListItem, ContainerStateStatusEnum,
    },
    image::{Image, ImageHistoryResponseItem, ImageListItem},
    network::{Network, NetworkListItem},
    volume::{Volume, VolumeListItem},
  },
  stack::ComposeProject,
  update::Log,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetContainersSummary",
  description = "Gets a summary of data relating to all containers.",
  request_body(content = GetContainersSummary),
  responses(
    (status = 200, description = "The containers summary", body = GetContainersSummaryResponse),
  ),
)]
pub fn get_containers_summary() {}

/// Gets a summary of data relating to all containers.
/// Response: [GetContainersSummaryResponse].
///
/// Pre v2.3.0, called `GetDockerContainersSummary`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetContainersSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetContainersSummary {}

/// Response for [GetContainersSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetContainersSummaryResponse {
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListAllContainers",
  description = "List all containers on the target servers.",
  request_body(content = ListAllContainers),
  responses(
    (status = 200, description = "The list of containers", body = ListAllContainersResponse),
  ),
)]
pub fn list_all_containers() {}

/// List all containers on the target servers.
/// Response: [ListAllContainersResponse].
///
/// Pre v2.3.0, called `ListAllDockerContainers`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListAllContainersResponse)]
#[error(mogh_error::Error)]
pub struct ListAllContainers {
  /// Filter by server id or name.
  #[serde(default)]
  pub servers: Vec<String>,

  /// Filter servers by tag.
  #[serde(default)]
  pub tags: Vec<String>,

  /// Filter by container name.
  /// Returned containers have names which contain all terms.
  #[serde(default)]
  pub terms: Vec<String>,

  /// Filter by container state.
  #[serde(default)]
  pub state: Vec<ContainerStateStatusEnum>,

  /// Retrieve more results by incrementing the page.
  /// `page: 0` is default.
  #[serde(default)]
  pub page: U64,

  /// Set the limit for number of containers per-page.
  /// `limit: 100` is default.
  ///
  /// Note: the page logic relies on this being consistent
  /// across queries for more pages.
  #[serde(default = "default_limit")]
  pub limit: U64,
}

fn default_limit() -> u64 {
  100
}

#[typeshare]
pub type ListAllContainersResponse = Vec<ContainerListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListContainers",
  description = "List all containers on the target server.",
  request_body(content = ListContainers),
  responses(
    (status = 200, description = "The list of containers", body = ListContainersResponse),
  ),
)]
pub fn list_containers() {}

/// List all containers on the target server.
/// Response: [ListContainersResponse].
///
/// Pre v2.3.0, called `ListDockerContainers`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListContainersResponse)]
#[error(mogh_error::Error)]
pub struct ListContainers {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListContainersResponse = Vec<ContainerListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectContainer",
  description = "Inspect a container on the server.",
  request_body(content = InspectContainer),
  responses(
    (status = 200, description = "The container", body = InspectContainerResponse),
  ),
)]
pub fn inspect_container() {}

/// Inspect a container on the server. Response: [Container].
///
/// Pre v2.3.0, called `InspectDockerContainer`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectContainerResponse)]
#[error(mogh_error::Error)]
pub struct InspectContainer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The container name
  pub container: String,
}

#[typeshare]
pub type InspectContainerResponse = Container;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetResourceMatchingContainer",
  description = "Find the attached resource for a container.",
  request_body(content = GetResourceMatchingContainer),
  responses(
    (status = 200, description = "The resource matching the container", body = GetResourceMatchingContainerResponse),
  ),
)]
pub fn get_resource_matching_container() {}

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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetContainerLog",
  description = "Get the container log's tail, split by stdout/stderr.",
  request_body(content = GetContainerLog),
  responses(
    (status = 200, description = "The container log", body = GetContainerLogResponse),
  ),
)]
pub fn get_container_log() {}

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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SearchContainerLog",
  description = "Search the container log's tail using `grep`.",
  request_body(content = SearchContainerLog),
  responses(
    (status = 200, description = "The search results", body = SearchContainerLogResponse),
  ),
)]
pub fn search_container_log() {}

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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListComposeProjects",
  description = "List all compose projects on the target server.",
  request_body(content = ListComposeProjects),
  responses(
    (status = 200, description = "The list of compose projects", body = ListComposeProjectsResponse),
  ),
)]
pub fn list_compose_projects() {}

/// List all compose projects on the target server.
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListNetworks",
  description = "List the container networks on the server.",
  request_body(content = ListNetworks),
  responses(
    (status = 200, description = "The list of networks", body = ListNetworksResponse),
  ),
)]
pub fn list_networks() {}

/// List the container networks on the server. Response: [ListNetworksResponse].
///
/// Pre v2.3.0, called `ListDockerNetworks`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListNetworksResponse)]
#[error(mogh_error::Error)]
pub struct ListNetworks {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListNetworksResponse = Vec<NetworkListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectNetwork",
  description = "Inspect a container network on the server.",
  request_body(content = InspectNetwork),
  responses(
    (status = 200, description = "The network", body = InspectNetworkResponse),
  ),
)]
pub fn inspect_network() {}

/// Inspect a container network on the server. Response: [InspectNetworkResponse].
///
/// Pre v2.3.0, called `InspectDockerNetwork`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectNetworkResponse)]
#[error(mogh_error::Error)]
pub struct InspectNetwork {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The network name
  pub network: String,
}

#[typeshare]
pub type InspectNetworkResponse = Network;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListImages",
  description = "List the container images locally cached on the target server.",
  request_body(content = ListImages),
  responses(
    (status = 200, description = "The list of images", body = ListImagesResponse),
  ),
)]
pub fn list_images() {}

/// List the container images locally cached on the target server.
/// Response: [ListImagesResponse].
///
/// Pre v2.3.0, called `ListDockerImages`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListImagesResponse)]
#[error(mogh_error::Error)]
pub struct ListImages {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListImagesResponse = Vec<ImageListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectImage",
  description = "Inspect a container image on the server.",
  request_body(content = InspectImage),
  responses(
    (status = 200, description = "The image", body = InspectImageResponse),
  ),
)]
pub fn inspect_image() {}

/// Inspect a container image on the server. Response: [Image].
///
/// Pre v2.3.0, called `InspectDockerImage`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectImageResponse)]
#[error(mogh_error::Error)]
pub struct InspectImage {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The image name
  pub image: String,
}

#[typeshare]
pub type InspectImageResponse = Image;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListImageHistory",
  description = "Get image history from the server.",
  request_body(content = ListImageHistory),
  responses(
    (status = 200, description = "The image history", body = ListImageHistoryResponse),
  ),
)]
pub fn list_image_history() {}

/// Get image history from the server. Response: [ListImageHistoryResponse].
///
/// Pre v2.3.0, called `ListDockerImageHistory`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListImageHistoryResponse)]
#[error(mogh_error::Error)]
pub struct ListImageHistory {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The image name
  pub image: String,
}

#[typeshare]
pub type ListImageHistoryResponse = Vec<ImageHistoryResponseItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListVolumes",
  description = "List all container volumes on the target server.",
  request_body(content = ListVolumes),
  responses(
    (status = 200, description = "The list of volumes", body = ListVolumesResponse),
  ),
)]
pub fn list_volumes() {}

/// List all container volumes on the target server.
/// Response: [ListVolumesResponse].
///
/// Pre v2.3.0, called `ListDockerVolumes`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListVolumesResponse)]
#[error(mogh_error::Error)]
pub struct ListVolumes {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListVolumesResponse = Vec<VolumeListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/InspectVolume",
  description = "Inspect a container volume on the server.",
  request_body(content = InspectVolume),
  responses(
    (status = 200, description = "The volume", body = InspectVolumeResponse),
  ),
)]
pub fn inspect_volume() {}

/// Inspect a container volume on the server. Response: [Volume].
///
/// Pre v2.3.0, called `InspectDockerVolume`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(InspectVolumeResponse)]
#[error(mogh_error::Error)]
pub struct InspectVolume {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The volume name
  pub volume: String,
}

#[typeshare]
pub type InspectVolumeResponse = Volume;
