use komodo_client::entities::{
  docker::{
    container::ContainerListItem, image::ImageListItem,
    network::NetworkListItem, volume::VolumeListItem,
  },
  stack::ComposeProject,
};
use periphery_client::{
  PeripheryClient,
  api::{GetDockerLists, GetDockerListsResponse},
};

pub async fn get_docker_lists(
  periphery: &PeripheryClient,
) -> anyhow::Result<(
  Vec<ContainerListItem>,
  Vec<NetworkListItem>,
  Vec<ImageListItem>,
  Vec<VolumeListItem>,
  Vec<ComposeProject>,
)> {
  let GetDockerListsResponse {
    containers,
    networks,
    images,
    volumes,
    projects,
  } = periphery.request(GetDockerLists {}).await?;
  // TODO: handle the errors
  let (
    mut containers,
    mut networks,
    mut images,
    mut volumes,
    mut projects,
  ) = (
    containers.unwrap_or_default(),
    networks.unwrap_or_default(),
    images.unwrap_or_default(),
    volumes.unwrap_or_default(),
    projects.unwrap_or_default(),
  );

  containers.sort_by(|a, b| a.name.cmp(&b.name));
  networks.sort_by(|a, b| a.name.cmp(&b.name));
  images.sort_by(|a, b| a.name.cmp(&b.name));
  volumes.sort_by(|a, b| a.name.cmp(&b.name));
  projects.sort_by(|a, b| a.name.cmp(&b.name));

  Ok((containers, networks, images, volumes, projects))
}
