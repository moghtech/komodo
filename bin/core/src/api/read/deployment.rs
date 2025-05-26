use std::{cmp, collections::HashSet};

use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::*,
  entities::{
    deployment::{
      Deployment, DeploymentActionState, DeploymentConfig,
      DeploymentListItem, DeploymentState,
    },
    docker::container::ContainerStats,
    permission::PermissionLevel,
    server::Server,
    update::Log,
  },
};
use periphery_client::api;
use resolver_api::Resolve;

use crate::{
  helpers::{periphery_client, query::get_all_tags},
  permission::get_check_permissions,
  resource,
  state::{action_states, deployment_status_cache},
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
    let Deployment {
      name,
      config: DeploymentConfig { server_id, .. },
      ..
    } = get_check_permissions::<Deployment>(
      &deployment,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    if server_id.is_empty() {
      return Ok(Log::default());
    }
    let server = resource::get::<Server>(&server_id).await?;
    let res = periphery_client(&server)?
      .request(api::container::GetContainerLog {
        name,
        tail: cmp::min(tail, MAX_LOG_LENGTH),
        timestamps,
      })
      .await
      .context("failed at call to periphery")?;
    Ok(res)
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
    let Deployment {
      name,
      config: DeploymentConfig { server_id, .. },
      ..
    } = get_check_permissions::<Deployment>(
      &deployment,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    if server_id.is_empty() {
      return Ok(Log::default());
    }
    let server = resource::get::<Server>(&server_id).await?;
    let res = periphery_client(&server)?
      .request(api::container::GetContainerLogSearch {
        name,
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
    let res = periphery_client(&server)?
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
          res.unknown += 1;
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
