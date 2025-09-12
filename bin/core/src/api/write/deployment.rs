use anyhow::{Context, anyhow};
use database::mungos::{by_id::update_one_by_id, mongodb::bson::doc};
use komodo_client::{
  api::write::*,
  entities::{
    Operation,
    deployment::{
      Deployment, DeploymentImage, DeploymentState,
      PartialDeploymentConfig, RestartMode,
    },
    docker::container::RestartPolicyNameEnum,
    komodo_timestamp,
    permission::PermissionLevel,
    server::{Server, ServerState},
    to_container_compatible_name,
    update::Update,
  },
};
use periphery_client::api::{self, container::InspectContainer};
use resolver_api::Resolve;

use crate::{
  helpers::{
    periphery_client,
    query::get_deployment_state,
    update::{add_update, make_update},
  },
  permission::get_check_permissions,
  resource,
  state::{action_states, db_client, server_status_cache},
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateDeployment {
  #[instrument(name = "CreateDeployment", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Deployment> {
    resource::create::<Deployment>(&self.name, self.config, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyDeployment {
  #[instrument(name = "CopyDeployment", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Deployment> {
    let Deployment { config, .. } =
      get_check_permissions::<Deployment>(
        &self.id,
        user,
        PermissionLevel::Read.into(),
      )
      .await?;
    resource::create::<Deployment>(&self.name, config.into(), user)
      .await
  }
}

impl Resolve<WriteArgs> for CreateDeploymentFromContainer {
  #[instrument(name = "CreateDeploymentFromContainer", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Deployment> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.inspect().attach(),
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
    let container = periphery_client(&server)?
      .request(InspectContainer {
        name: self.name.clone(),
      })
      .await
      .context("Failed to inspect container")?;

    let mut config = PartialDeploymentConfig {
      server_id: server.id.into(),
      ..Default::default()
    };

    if let Some(container_config) = container.config {
      config.image = container_config
        .image
        .map(|image| DeploymentImage::Image { image });
      config.command = container_config.cmd.join(" ").into();
      config.environment = container_config
        .env
        .into_iter()
        .map(|env| format!("  {env}"))
        .collect::<Vec<_>>()
        .join("\n")
        .into();
      config.labels = container_config
        .labels
        .into_iter()
        .map(|(key, val)| format!("  {key}: {val}"))
        .collect::<Vec<_>>()
        .join("\n")
        .into();
    }
    if let Some(host_config) = container.host_config {
      config.volumes = host_config
        .binds
        .into_iter()
        .map(|bind| format!("  {bind}"))
        .collect::<Vec<_>>()
        .join("\n")
        .into();
      config.network = host_config.network_mode;
      config.ports = host_config
        .port_bindings
        .into_iter()
        .filter_map(|(container, mut host)| {
          let host = host.pop()?.host_port?;
          Some(format!("  {host}:{}", container.replace("/tcp", "")))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into();
      config.restart = host_config.restart_policy.map(|restart| {
        match restart.name {
          RestartPolicyNameEnum::Always => RestartMode::Always,
          RestartPolicyNameEnum::No
          | RestartPolicyNameEnum::Empty => RestartMode::NoRestart,
          RestartPolicyNameEnum::UnlessStopped => {
            RestartMode::UnlessStopped
          }
          RestartPolicyNameEnum::OnFailure => RestartMode::OnFailure,
        }
      });
    }

    resource::create::<Deployment>(&self.name, config, user).await
  }
}

impl Resolve<WriteArgs> for DeleteDeployment {
  #[instrument(name = "DeleteDeployment", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<Deployment> {
    Ok(resource::delete::<Deployment>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateDeployment {
  #[instrument(name = "UpdateDeployment", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Deployment> {
    Ok(
      resource::update::<Deployment>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameDeployment {
  #[instrument(name = "RenameDeployment", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    let deployment = get_check_permissions::<Deployment>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.renaming = true)?;

    let name = to_container_compatible_name(&self.name);

    let container_state =
      get_deployment_state(&deployment.id).await?;

    if container_state == DeploymentState::Unknown {
      return Err(
        anyhow!(
          "Cannot rename Deployment when container status is unknown"
        )
        .into(),
      );
    }

    let mut update =
      make_update(&deployment, Operation::RenameDeployment, user);

    update_one_by_id(
      &db_client().deployments,
      &deployment.id,
      database::mungos::update::Update::Set(
        doc! { "name": &name, "updated_at": komodo_timestamp() },
      ),
      None,
    )
    .await
    .context("Failed to update Deployment name on db")?;

    if container_state != DeploymentState::NotDeployed {
      let server =
        resource::get::<Server>(&deployment.config.server_id).await?;
      let log = periphery_client(&server)?
        .request(api::container::RenameContainer {
          curr_name: deployment.name.clone(),
          new_name: name.clone(),
        })
        .await
        .context("Failed to rename container on server")?;
      update.logs.push(log);
    }

    update.push_simple_log(
      "Rename Deployment",
      format!(
        "Renamed Deployment from {} to {}",
        deployment.name, name
      ),
    );
    update.finalize();
    update.id = add_update(update.clone()).await?;

    Ok(update)
  }
}
