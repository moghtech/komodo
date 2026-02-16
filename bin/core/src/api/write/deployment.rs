use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use database::mungos::{by_id::update_one_by_id, mongodb::bson::doc};
use futures_util::{StreamExt as _, stream::FuturesOrdered};
use komodo_client::{
  api::{execute::Deploy, write::*},
  entities::{
    Operation, ResourceTarget, SwarmOrServer,
    alert::{Alert, AlertData, SeverityLevel},
    deployment::{
      Deployment, DeploymentImage, DeploymentInfo, DeploymentState,
      PartialDeploymentConfig, RestartMode, extract_registry_domain,
    },
    docker::container::RestartPolicyNameEnum,
    komodo_timestamp, optional_string,
    permission::PermissionLevel,
    server::{Server, ServerState},
    to_container_compatible_name,
    update::Update,
    user::{auto_redeploy_user, system_user},
  },
};
use mogh_cache::SetCache;
use mogh_resolver::Resolve;
use periphery_client::api::{self, container::InspectContainer};

use crate::{
  alert::send_alerts,
  api::execute::{self, ExecuteRequest, ExecutionResult},
  helpers::{
    periphery_client,
    query::{get_deployment_state, get_swarm_or_server},
    registry_token,
    update::{add_update, make_update, poll_update_until_complete},
  },
  permission::get_check_permissions,
  resource::{
    self, list_full_for_user_using_pattern,
    setup_deployment_execution,
  },
  state::{
    action_states, db_client, deployment_status_cache,
    image_digest_cache, server_status_cache,
  },
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateDeployment {
  #[instrument(
    "CreateDeployment",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Deployment> {
    resource::create::<Deployment>(
      &self.name,
      self.config,
      None,
      user,
    )
    .await
  }
}

impl Resolve<WriteArgs> for CopyDeployment {
  #[instrument(
    "CopyDeployment",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.name,
      copy_deployment = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Deployment> {
    let Deployment { config, .. } =
      get_check_permissions::<Deployment>(
        &self.id,
        user,
        PermissionLevel::Read.into(),
      )
      .await?;
    resource::create::<Deployment>(
      &self.name,
      config.into(),
      None,
      user,
    )
    .await
  }
}

impl Resolve<WriteArgs> for CreateDeploymentFromContainer {
  #[instrument(
    "CreateDeploymentFromContainer",
    skip_all,
    fields(
      operator = user.id,
      server = self.server,
      deployment = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Deployment> {
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
    let container = periphery_client(&server)
      .await?
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

    resource::create::<Deployment>(&self.name, config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteDeployment {
  #[instrument(
    "DeleteDeployment",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.id
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Deployment> {
    Ok(resource::delete::<Deployment>(&self.id, user).await?)
  }
}

impl Resolve<WriteArgs> for UpdateDeployment {
  #[instrument(
    "UpdateDeployment",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.id,
      update = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Deployment> {
    // If the update changes image,
    // also update the stored latest image digest.
    let image_update = self
      .config
      .image
      .as_ref()
      .map(|image| image.as_image().is_some())
      .unwrap_or_default();

    let deployment =
      resource::update::<Deployment>(&self.id, self.config, user)
        .await?;

    if image_update {
      tokio::spawn(async move {
        let _ = (CheckDeploymentForUpdate {
          deployment: self.id,
          skip_auto_update: false,
          wait_for_auto_update: false,
        })
        .resolve(&WriteArgs {
          user: system_user().to_owned(),
        })
        .await;
      });
    }

    Ok(deployment)
  }
}

impl Resolve<WriteArgs> for RenameDeployment {
  #[instrument(
    "RenameDeployment",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.id,
      new_name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
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
      let log = periphery_client(&server)
        .await?
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

//

impl Resolve<WriteArgs> for CheckDeploymentForUpdate {
  #[instrument(
    "CheckDeploymentForUpdate",
    skip_all,
    fields(
      operator = user.id,
      deployment = self.deployment,
      skip_auto_update = self.skip_auto_update,
      wait_for_auto_update = self.wait_for_auto_update,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Self::Response> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // deployment should be able to do this.
    let (deployment, swarm_or_server) = setup_deployment_execution(
      &self.deployment,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    swarm_or_server.verify_has_target()?;

    check_deployment_for_update_inner(
      deployment,
      &swarm_or_server,
      self.skip_auto_update,
      self.wait_for_auto_update,
    )
    .await
    .map_err(Into::into)
  }
}

/// If it goes down the "update available" path,
/// only send alert if deployment id is not in this cache.
/// If alert is sent, add ID to cache.
/// If later it goes down non "update available" path,
/// remove the id from cache, so next time it does another alert
/// will be sent.
fn deployment_alert_sent_cache() -> &'static SetCache<String> {
  static CACHE: OnceLock<SetCache<String>> = OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// Checks remote registry for latest image digest,
/// and saves it to database associated with the deployment.
///
/// Returns true if update is available and auto deploy is false.
/// If auto deploy is true, this will deploy.
#[instrument(
  "CheckDeploymentForUpdateInner",
  skip_all,
  fields(
    deployment = deployment.id,
    skip_auto_update,
    wait_for_auto_update,
  )
)]
pub async fn check_deployment_for_update_inner(
  deployment: Deployment,
  swarm_or_server: &SwarmOrServer,
  skip_auto_update: bool,
  // Otherwise spawns task to run in background
  wait_for_auto_update: bool,
) -> anyhow::Result<CheckDeploymentForUpdateResponse> {
  let alert_cache = deployment_alert_sent_cache();

  let (image, account, token) = match &deployment.config.image {
    DeploymentImage::Image { image } => {
      if image.contains('@') {
        // Images with a hardcoded digest can't have update.
        return Ok(CheckDeploymentForUpdateResponse {
          deployment: deployment.id,
          update_available: false,
        });
      }
      let domain = extract_registry_domain(&image)?;
      let account =
        optional_string(&deployment.config.image_registry_account);
      let token = if let Some(account) = &account {
        registry_token(&domain, account).await?
      } else {
        None
      };
      (image, account, token)
    }
    DeploymentImage::Build { .. } => {
      alert_cache.remove(&deployment.id).await;
      // This method not used for build based deployments
      // as deployed version vs built version can be inferred from Updates.
      return Ok(CheckDeploymentForUpdateResponse {
        deployment: deployment.id,
        update_available: false,
      });
    }
  };

  let latest_digest = image_digest_cache()
    .get(&swarm_or_server, &image, account, token)
    .await?;

  resource::update_info::<Deployment>(
    &deployment.id,
    &DeploymentInfo {
      latest_image_digest: latest_digest.clone(),
    },
  )
  .await?;

  let Some((state, Some(current_digests))) =
    deployment_status_cache()
      .get(&deployment.id)
      .await
      .map(|s| (s.curr.state, s.curr.image_digests.clone()))
  else {
    alert_cache.remove(&deployment.id).await;
    return Ok(CheckDeploymentForUpdateResponse {
      deployment: deployment.id,
      update_available: false,
    });
  };

  // If not running or latest digest matches current, early return
  if !matches!(state, DeploymentState::Running)
    || !latest_digest.update_available(&current_digests)
  {
    alert_cache.remove(&deployment.id).await;
    return Ok(CheckDeploymentForUpdateResponse {
      deployment: deployment.id,
      update_available: false,
    });
  }

  if !skip_auto_update && deployment.config.auto_update {
    // Trigger deploy + alert

    // Conservatively remove from alert cache so 'skip_auto_update'
    // doesn't cause alerts not to be sent on subsequent calls.
    alert_cache.remove(&deployment.id).await;

    let swarm_id = swarm_or_server.swarm_id().map(str::to_string);
    let swarm_name = swarm_or_server.swarm_name().map(str::to_string);
    let server_id = swarm_or_server.server_id().map(str::to_string);
    let server_name =
      swarm_or_server.server_name().map(str::to_string);
    let id = deployment.id.clone();
    let name = deployment.name.clone();
    let image = image.clone();

    let run = async move {
      match execute::inner_handler(
        ExecuteRequest::Deploy(Deploy {
          deployment: name.clone(),
          stop_signal: None,
          stop_time: None,
        }),
        auto_redeploy_user().to_owned(),
      )
      .await
      {
        Ok(res) => {
          let ExecutionResult::Single(update) = res else {
            unreachable!()
          };
          let Ok(update) =
            poll_update_until_complete(&update.id).await
          else {
            return;
          };
          if update.success {
            let ts = komodo_timestamp();
            let alert = Alert {
              id: Default::default(),
              ts,
              resolved: true,
              resolved_ts: ts.into(),
              level: SeverityLevel::Ok,
              target: ResourceTarget::Deployment(id.clone()),
              data: AlertData::DeploymentAutoUpdated {
                id,
                name,
                swarm_id,
                swarm_name,
                server_id,
                server_name,
                image,
              },
            };
            let res = db_client().alerts.insert_one(&alert).await;
            if let Err(e) = res {
              error!(
                "Failed to record DeploymentAutoUpdated to db | {e:#}"
              );
            }
            send_alerts(&[alert]).await;
          }
        }
        Err(e) => {
          warn!("Failed to auto update Deployment {name} | {e:#}",)
        }
      }
    };

    if wait_for_auto_update {
      run.await
    } else {
      tokio::spawn(run);
    }
  } else {
    // Avoid spamming alerts
    if alert_cache.contains(&deployment.id).await {
      return Ok(CheckDeploymentForUpdateResponse {
        deployment: deployment.id,
        update_available: true,
      });
    }
    alert_cache.insert(deployment.id.clone()).await;
    let ts = komodo_timestamp();
    let alert = Alert {
      id: Default::default(),
      ts,
      resolved: true,
      resolved_ts: ts.into(),
      level: SeverityLevel::Ok,
      target: ResourceTarget::Deployment(deployment.id.clone()),
      data: AlertData::DeploymentImageUpdateAvailable {
        id: deployment.id.clone(),
        name: deployment.name.clone(),
        swarm_id: swarm_or_server.swarm_id().map(str::to_string),
        swarm_name: swarm_or_server.swarm_name().map(str::to_string),
        server_id: swarm_or_server.server_id().map(str::to_string),
        server_name: swarm_or_server
          .server_name()
          .map(str::to_string),
        image: image.clone(),
      },
    };
    let res = db_client().alerts.insert_one(&alert).await;
    if let Err(e) = res {
      error!(
        "Failed to record DeploymentImageUpdateAvailable to db | {e:#}"
      );
    }
    send_alerts(&[alert]).await;
  }

  Ok(CheckDeploymentForUpdateResponse {
    deployment: deployment.id,
    update_available: !deployment.config.auto_update,
  })
}

//

impl Resolve<WriteArgs> for BatchCheckDeploymentForUpdate {
  #[instrument(
    "BatchCheckDeploymentForUpdate",
    skip_all,
    fields(
      operator = user.id,
      pattern = self.pattern,
      skip_auto_update = self.skip_auto_update,
      wait_for_auto_update = self.wait_for_auto_update,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let deployments = list_full_for_user_using_pattern::<Deployment>(
      &self.pattern,
      Default::default(),
      user,
      PermissionLevel::Execute.into(),
      &[],
    )
    .await?;

    let res = deployments
      .into_iter()
      .map(|deployment| async move {
        let swarm_or_server = get_swarm_or_server(
          &deployment.config.swarm_id,
          &deployment.config.server_id,
        )
        .await?;
        swarm_or_server.verify_has_target().map_err(|e| e.error)?;
        check_deployment_for_update_inner(
          deployment,
          &swarm_or_server,
          self.skip_auto_update,
          self.wait_for_auto_update,
        )
        .await
      })
      .collect::<FuturesOrdered<_>>()
      .collect::<Vec<_>>()
      .await
      .into_iter()
      .filter_map(|res| {
        res
          .inspect_err(|e| {
            warn!(
              "Failed to check deployment for update in batch run | {e:#}"
            )
          })
          .ok()
      })
      .collect();
    Ok(res)
  }
}
