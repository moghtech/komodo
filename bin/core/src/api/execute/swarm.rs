use formatting::format_serror;
use komodo_client::{
  api::execute::{
    RemoveSwarmConfigs, RemoveSwarmNodes, RemoveSwarmSecrets,
    RemoveSwarmServices, RemoveSwarmStacks,
  },
  entities::{permission::PermissionLevel, swarm::Swarm},
};
use resolver_api::Resolve;

use crate::{
  api::execute::ExecuteArgs,
  helpers::{swarm::swarm_request, update::update_update},
  permission::get_check_permissions,
};

impl Resolve<ExecuteArgs> for RemoveSwarmNodes {
  #[instrument(
    "RemoveSwarmNodes",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      swarm = self.swarm,
      nodes = serde_json::to_string(&self.nodes).unwrap_or_else(|e| e.to_string()),
      force = self.force,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    update_update(update.clone()).await?;

    let mut update = update.clone();

    match swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::RemoveSwarmNodes {
        nodes: self.nodes,
        force: self.force,
      },
    )
    .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Remove Swarm Nodes",
        format_serror(
          &e.context("Failed to remove swarm nodes").into(),
        ),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RemoveSwarmStacks {
  #[instrument(
    "RemoveSwarmStacks",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      swarm = self.swarm,
      stacks = serde_json::to_string(&self.stacks).unwrap_or_else(|e| e.to_string()),
      detach = self.detach,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    update_update(update.clone()).await?;

    let mut update = update.clone();

    match swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::RemoveSwarmStacks {
        stacks: self.stacks,
        detach: self.detach,
      },
    )
    .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Remove Swarm Stacks",
        format_serror(
          &e.context("Failed to remove swarm stacks").into(),
        ),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RemoveSwarmServices {
  #[instrument(
    "RemoveSwarmServices",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      swarm = self.swarm,
      services = serde_json::to_string(&self.services).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    update_update(update.clone()).await?;

    let mut update = update.clone();

    match swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::RemoveSwarmServices {
        services: self.services,
      },
    )
    .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Remove Swarm Services",
        format_serror(
          &e.context("Failed to remove swarm services").into(),
        ),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RemoveSwarmConfigs {
  #[instrument(
    "RemoveSwarmConfigs",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      swarm = self.swarm,
      configs = serde_json::to_string(&self.configs).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    update_update(update.clone()).await?;

    let mut update = update.clone();

    match swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::RemoveSwarmConfigs {
        configs: self.configs,
      },
    )
    .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Remove Swarm Configs",
        format_serror(
          &e.context("Failed to remove swarm configs").into(),
        ),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RemoveSwarmSecrets {
  #[instrument(
    "RemoveSwarmSecrets",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      swarm = self.swarm,
      secrets = serde_json::to_string(&self.secrets).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let swarm = get_check_permissions::<Swarm>(
      &self.swarm,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    update_update(update.clone()).await?;

    let mut update = update.clone();

    match swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::RemoveSwarmSecrets {
        secrets: self.secrets,
      },
    )
    .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Remove Swarm Secrets",
        format_serror(
          &e.context("Failed to remove swarm secrets").into(),
        ),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
