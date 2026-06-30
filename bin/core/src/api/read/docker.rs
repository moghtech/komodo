use std::cmp;

use anyhow::{Context as _, anyhow};
use database::bson::doc;
use komodo_client::{
  api::read::*,
  entities::{
    ResourceTarget,
    deployment::Deployment,
    docker::{
      container::{
        Container, ContainerListItem, ContainerStateStatusEnum,
      },
      image::{Image, ImageHistoryResponseItem},
      network::Network,
      volume::Volume,
    },
    permission::PermissionLevel,
    server::{Server, ServerQuery, ServerState},
    stack::{Stack, StackServiceNames},
    update::Log,
  },
};
use mogh_resolver::Resolve;
use periphery_client::api::{
  self as periphery,
  container::InspectContainer,
  docker::{
    ImageHistory, InspectImage, InspectNetwork, InspectVolume,
  },
};
use wildcard::Wildcard;

use crate::{
  api::read::ReadArgs,
  helpers::periphery_client,
  permission::{get_check_permissions, list_resources_for_user},
  resource,
  stack::compose_container_match_regex,
  state::server_status_cache,
};

impl Resolve<ReadArgs> for GetDockerContainersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetDockerContainersSummaryResponse> {
    let servers = resource::list_full_for_user::<Server>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get servers from db")?;

    let mut res = GetDockerContainersSummaryResponse::default();

    for server in servers {
      let cache = server_status_cache()
        .get_or_insert_default(&server.id)
        .await;

      if let Some(docker) = &cache.docker {
        for container in &docker.containers {
          res.total += 1;
          match container.state {
            ContainerStateStatusEnum::Created
            | ContainerStateStatusEnum::Paused
            | ContainerStateStatusEnum::Exited => res.stopped += 1,
            ContainerStateStatusEnum::Running => res.running += 1,
            ContainerStateStatusEnum::Empty => res.unknown += 1,
            _ => res.unhealthy += 1,
          }
        }
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListAllDockerContainers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListAllDockerContainersResponse> {
    let servers = resource::list_for_user::<Server>(
      ServerQuery::builder()
        .names(self.servers.clone())
        .tags(self.tags)
        .build(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?;

    let mut containers = Vec::<ContainerListItem>::new();
    let mut skipped = 0;
    let limit_usize = self.limit as usize;

    for server in servers {
      let cache = server_status_cache()
        .get_or_insert_default(&server.id)
        .await;
      let Some(docker) = &cache.docker else {
        continue;
      };
      let more = docker
        .containers
        .iter()
        .filter(|container| {
          // Apply state filter if defined.
          (self.state.is_empty() || self.state.contains(&container.state)) &&
          // Apply terms filter if defined
          (self.terms.is_empty()
            // Match when all terms contained within a name.
            || self.terms.iter().all(|term| container.name.contains(term)))
        });
      for container in more {
        if skipped < self.limit * self.page {
          // Eg. page 1 skips until after 100 containers, page 2 after 200.
          skipped += 1;
        } else {
          // push and maybe early return
          containers.push(container.clone());
          if containers.len() >= limit_usize {
            return Ok(containers);
          }
        }
      }
    }

    Ok(containers)
  }
}

impl Resolve<ReadArgs> for ListDockerContainers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerContainersResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.containers.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Container> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect container: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectContainer {
        name: self.container,
      })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetResourceMatchingContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetResourceMatchingContainerResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    // first check deployments
    if let Ok(deployment) =
      resource::get::<Deployment>(&self.container).await
    {
      return Ok(GetResourceMatchingContainerResponse {
        resource: ResourceTarget::Deployment(deployment.id).into(),
      });
    }

    // then check stacks
    let stacks = list_resources_for_user::<Stack>(
      doc! { "config.server_id": &server.id },
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    // check matching stack
    for stack in stacks {
      for StackServiceNames {
        service_name,
        container_name,
        ..
      } in stack
        .info
        .deployed_services
        .unwrap_or(stack.info.latest_services)
      {
        let is_match = match compose_container_match_regex(&container_name)
          .with_context(|| format!("failed to construct container name matching regex for service {service_name}")) 
        {
          Ok(regex) => regex,
          Err(e) => {
            warn!("{e:#}");
            continue;
          }
        }.is_match(&self.container);

        if is_match {
          return Ok(GetResourceMatchingContainerResponse {
            resource: ResourceTarget::Stack(stack.id).into(),
          });
        }
      }
    }

    Ok(GetResourceMatchingContainerResponse { resource: None })
  }
}

const MAX_LOG_LENGTH: u64 = 5000;

impl Resolve<ReadArgs> for GetContainerLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Log> {
    let GetContainerLog {
      server,
      container,
      tail,
      timestamps,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    let res = periphery_client(&server)
      .await?
      .request(periphery::container::GetContainerLog {
        name: container,
        tail: cmp::min(tail, MAX_LOG_LENGTH),
        timestamps,
      })
      .await
      .context("failed at call to periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for SearchContainerLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Log> {
    let SearchContainerLog {
      server,
      container,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    let res = periphery_client(&server)
      .await?
      .request(periphery::container::GetContainerLogSearch {
        name: container,
        terms,
        combinator,
        invert,
        timestamps,
      })
      .await
      .context("failed at call to periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListComposeProjects {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListComposeProjectsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.projects.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for ListDockerNetworks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerNetworksResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.networks.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerNetwork {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Network> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect network: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectNetwork { name: self.network })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImages {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerImagesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.images.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerImage {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Image> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!("Cannot inspect image: server is {:?}", cache.state)
          .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectImage { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerImageHistory {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<ImageHistoryResponseItem>> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot get image history: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(ImageHistory { name: self.image })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListDockerVolumes {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListDockerVolumesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if let Some(docker) = &cache.docker {
      Ok(docker.volumes.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectDockerVolume {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Volume> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!("Cannot inspect volume: server is {:?}", cache.state)
          .into(),
      );
    }
    let res = periphery_client(&server)
      .await?
      .request(InspectVolume { name: self.volume })
      .await?;
    Ok(res)
  }
}
