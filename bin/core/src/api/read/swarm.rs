use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    swarm::{Swarm, SwarmActionState, SwarmListItem, SwarmState},
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::{query::get_all_tags, swarm::swarm_request},
  permission::get_check_permissions,
  resource,
  state::{action_states, server_status_cache, swarm_status_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetSwarm {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Swarm> {
    Ok(
      get_check_permissions::<Swarm>(
        &self.swarm,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListSwarms {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<SwarmListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Swarm>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullSwarms {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullSwarmsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Swarm>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetSwarmActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<SwarmActionState> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .swarm
      .get(&swarm.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetSwarmsSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetSwarmsSummaryResponse> {
    let swarms = resource::list_full_for_user::<Swarm>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get swarms from db")?;

    let mut res = GetSwarmsSummaryResponse::default();

    let cache = swarm_status_cache();

    for swarm in swarms {
      res.total += 1;

      match cache
        .get(&swarm.id)
        .await
        .map(|status| status.state)
        .unwrap_or_default()
      {
        SwarmState::Unknown => {
          res.unknown += 1;
        }
        SwarmState::Healthy => {
          res.healthy += 1;
        }
        SwarmState::Unhealthy => {
          res.unhealthy += 1;
        }
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for InspectSwarm {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    let inspect = cache
      .inspect
      .as_ref()
      .cloned()
      .context("SwarmInspectInfo not available")?;
    Ok(inspect)
  }
}

impl Resolve<ReadArgs> for ListSwarmNodes {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmNodesResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.nodes.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmNode {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmNodeResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmNode {
        node: self.node,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmServices {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmServicesResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.services.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmService {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmServiceResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmService {
        service: self.service,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for GetSwarmServiceLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetSwarmServiceLogResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::GetSwarmServiceLog {
        service: self.service,
        tail: self.tail,
        timestamps: self.timestamps,
        no_task_ids: self.no_task_ids,
        no_resolve: self.no_resolve,
        details: self.details,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for SearchSwarmServiceLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<SearchSwarmServiceLogResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::GetSwarmServiceLogSearch {
        service: self.service,
        terms: self.terms,
        combinator: self.combinator,
        invert: self.invert,
        timestamps: self.timestamps,
        no_task_ids: self.no_task_ids,
        no_resolve: self.no_resolve,
        details: self.details,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmTasks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmTasksResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.tasks.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmTask {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmTaskResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmTask {
        task: self.task,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmSecrets {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmSecretsResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.secrets.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmSecret {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmSecretResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmSecret {
        secret: self.secret,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmConfigs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmConfigsResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.configs.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmConfig {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmConfigResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmConfig {
        config: self.config,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmStacks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListSwarmStacksResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let cache =
      swarm_status_cache().get_or_insert_default(&swarm.id).await;
    if let Some(lists) = &cache.lists {
      Ok(lists.stacks.clone())
    } else {
      Ok(Vec::new())
    }
  }
}

impl Resolve<ReadArgs> for InspectSwarmStack {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<InspectSwarmStackResponse> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmStack {
        stack: self.stack,
      },
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListSwarmNetworks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    let cache = server_status_cache();

    for server_id in swarm.config.server_ids {
      let Some(status) = cache.get(&server_id).await else {
        continue;
      };
      let Some(docker) = &status.docker else {
        continue;
      };
      let networks = docker
        .networks
        .iter()
        .filter(|network| {
          network.driver.as_deref() == Some("overlay")
        })
        .cloned()
        .collect::<Vec<_>>();
      return Ok(networks);
    }

    Err(
      anyhow!(
        "Failed to retrieve swarm networks from any manager node."
      )
      .into(),
    )
  }
}
