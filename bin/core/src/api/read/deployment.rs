use std::{cmp, collections::HashSet};

use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::*,
  entities::{
    SwarmOrServer,
    deployment::{
      Deployment, DeploymentActionState, DeploymentConfig,
      DeploymentListItem, DeploymentState,
    },
    docker::{
      container::{Container, ContainerStats},
      service::SwarmService,
    },
    permission::PermissionLevel,
    server::{Server, ServerState},
    update::Log,
  },
};
use periphery_client::api::{self, container::InspectContainer};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError as _;

use crate::{
  helpers::{
    periphery_client, query::get_all_tags, swarm::swarm_request,
  },
  permission::get_check_permissions,
  resource::{self, setup_deployment_execution},
  state::{
    action_states, deployment_status_cache, server_status_cache,
  },
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetDeployment {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Deployment> {
    Ok(
      get_check_permissions::<Deployment>(
        &self.deployment,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListDeployments {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<DeploymentListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let only_update_available = self.query.specific.update_available;
    let deployments = resource::list_for_user::<Deployment>(
      self.query,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await?;
    let deployments = if only_update_available {
      deployments
        .into_iter()
        .filter(|deployment| deployment.info.update_available)
        .collect()
    } else {
      deployments
    };
    Ok(deployments)
  }
}

impl Resolve<ReadArgs> for ListFullDeployments {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullDeploymentsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Deployment>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetDeploymentContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetDeploymentContainerResponse> {
    let deployment = get_check_permissions::<Deployment>(
      &self.deployment,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let status = deployment_status_cache()
      .get(&deployment.id)
      .await
      .unwrap_or_default();
    let response = GetDeploymentContainerResponse {
      state: status.curr.state,
      container: status.curr.container.clone(),
    };
    Ok(response)
  }
}

const MAX_LOG_LENGTH: u64 = 5000;

impl Resolve<ReadArgs> for GetDeploymentLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Log> {
    let GetDeploymentLog {
      deployment,
      tail,
      timestamps,
    } = self;

    let (deployment, swarm_or_server) = setup_deployment_execution(
      &deployment,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;

    let log = match swarm_or_server {
      SwarmOrServer::Swarm(swarm) => swarm_request(
        &swarm.config.server_ids,
        periphery_client::api::swarm::GetSwarmServiceLog {
          service: deployment.name,
          tail,
          timestamps,
          no_task_ids: false,
          no_resolve: false,
          details: false,
        },
      )
      .await
      .context("Failed to get service log from swarm")?,
      SwarmOrServer::Server(server) => periphery_client(&server)
        .await?
        .request(api::container::GetContainerLog {
          name: deployment.name,
          tail: cmp::min(tail, MAX_LOG_LENGTH),
          timestamps,
        })
        .await
        .context("failed at call to periphery")?,
    };

    Ok(log)
  }
}

impl Resolve<ReadArgs> for SearchDeploymentLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Log> {
    let SearchDeploymentLog {
      deployment,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;

    let (deployment, swarm_or_server) = setup_deployment_execution(
      &deployment,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;

    let log = match swarm_or_server {
      SwarmOrServer::Swarm(swarm) => swarm_request(
        &swarm.config.server_ids,
        periphery_client::api::swarm::GetSwarmServiceLogSearch {
          service: deployment.name,
          terms,
          combinator,
          invert,
          timestamps,
          no_task_ids: false,
          no_resolve: false,
          details: false,
        },
      )
      .await
      .context("Failed to search service log from swarm")?,
      SwarmOrServer::Server(server) => periphery_client(&server)
        .await?
        .request(api::container::GetContainerLogSearch {
          name: deployment.name,
          terms,
          combinator,
          invert,
          timestamps,
        })
        .await
        .context("Failed to search container log from server")?,
    };

    Ok(log)
  }
}

impl Resolve<ReadArgs> for InspectDeploymentContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Container> {
    let InspectDeploymentContainer { deployment } = self;
    let (deployment, swarm_or_server) = setup_deployment_execution(
      &deployment,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!(
          "InspectDeploymentContainer should not be called for Deployment in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;

    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect container: Server is {:?}",
          cache.state
        )
        .into(),
      );
    }

    periphery_client(&server)
      .await?
      .request(InspectContainer {
        name: deployment.name,
      })
      .await
      .context("Failed to inspect container on server")
      .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for InspectDeploymentSwarmService {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<SwarmService> {
    let InspectDeploymentSwarmService { deployment } = self;
    let (deployment, swarm_or_server) = setup_deployment_execution(
      &deployment,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;

    let SwarmOrServer::Swarm(swarm) = swarm_or_server else {
      return Err(
        anyhow!(
          "InspectDeploymentSwarmService should only be called for Deployment in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmService {
        service: deployment.name,
      },
    )
    .await
    .context("Failed to inspect service on swarm")
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for GetDeploymentStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ContainerStats> {
    let Deployment {
      name,
      config: DeploymentConfig { server_id, .. },
      ..
    } = get_check_permissions::<Deployment>(
      &self.deployment,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    if server_id.is_empty() {
      return Err(
        anyhow!("deployment has no server attached").into(),
      );
    }
    let server = resource::get::<Server>(&server_id).await?;
    let res = periphery_client(&server)
      .await?
      .request(api::container::GetContainerStats { name })
      .await
      .context("failed to get stats from periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetDeploymentActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<DeploymentActionState> {
    let deployment = get_check_permissions::<Deployment>(
      &self.deployment,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .deployment
      .get(&deployment.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetDeploymentsSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetDeploymentsSummaryResponse> {
    let deployments = resource::list_full_for_user::<Deployment>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get deployments from db")?;
    let mut res = GetDeploymentsSummaryResponse::default();
    let status_cache = deployment_status_cache();
    for deployment in deployments {
      res.total += 1;
      let status =
        status_cache.get(&deployment.id).await.unwrap_or_default();
      match status.curr.state {
        DeploymentState::Running => {
          res.running += 1;
        }
        DeploymentState::Exited | DeploymentState::Paused => {
          res.stopped += 1;
        }
        DeploymentState::NotDeployed => {
          res.not_deployed += 1;
        }
        DeploymentState::Unknown => {
          if !deployment.template {
            res.unknown += 1;
          }
        }
        _ => {
          res.unhealthy += 1;
        }
      }
    }
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListCommonDeploymentExtraArgs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListCommonDeploymentExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let deployments = resource::list_full_for_user::<Deployment>(
      self.query,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await
    .context("failed to get resources matching query")?;

    // first collect with guaranteed uniqueness
    let mut res = HashSet::<String>::new();

    for deployment in deployments {
      for extra_arg in deployment.config.extra_args {
        res.insert(extra_arg);
      }
    }

    let mut res = res.into_iter().collect::<Vec<_>>();
    res.sort();
    Ok(res)
  }
}
