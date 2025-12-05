use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use cache::TimeoutCache;
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::{
  api::execute::*,
  entities::{
    SwarmOrServer, Version,
    build::{Build, ImageRegistryConfig},
    deployment::{
      Deployment, DeploymentImage, extract_registry_domain,
    },
    komodo_timestamp, optional_string,
    permission::PermissionLevel,
    server::Server,
    update::{Log, Update},
    user::User,
  },
};
use periphery_client::api;
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;

use crate::{
  helpers::{
    periphery_client,
    query::{
      VariablesAndSecrets, get_swarm_or_server,
      get_variables_and_secrets,
    },
    registry_token,
    swarm::swarm_request,
    update::update_update,
  },
  monitor::{update_cache_for_server, update_cache_for_swarm},
  permission::get_check_permissions,
  resource,
  state::action_states,
};

use super::{ExecuteArgs, ExecuteRequest};

impl super::BatchExecute for BatchDeploy {
  type Resource = Deployment;
  fn single_request(deployment: String) -> ExecuteRequest {
    ExecuteRequest::Deploy(Deploy {
      deployment,
      stop_signal: None,
      stop_time: None,
    })
  }
}

impl Resolve<ExecuteArgs> for BatchDeploy {
  #[instrument(
    "BatchDeploy",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      pattern = self.pattern,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, id, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDeploy>(&self.pattern, user)
        .await?,
    )
  }
}

#[instrument("SetupDeploy", skip_all)]
async fn setup_deployment_execution(
  deployment: &str,
  user: &User,
) -> anyhow::Result<(Deployment, SwarmOrServer)> {
  let deployment = get_check_permissions::<Deployment>(
    deployment,
    user,
    PermissionLevel::Execute.into(),
  )
  .await?;

  let swarm_or_server = get_swarm_or_server(
    &deployment.config.swarm_id,
    &deployment.config.server_id,
  )
  .await?;

  Ok((deployment, swarm_or_server))
}

impl Resolve<ExecuteArgs> for Deploy {
  #[instrument(
    "Deploy",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
      stop_signal = format!("{:?}", self.stop_signal),
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (mut deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.deploying = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    // This block resolves the attached Build to an actual versioned image
    let (version, registry_token) = match &deployment.config.image {
      DeploymentImage::Build { build_id, version } => {
        let build = resource::get::<Build>(build_id).await?;
        let image_names = build.get_image_names();
        let image_name = image_names
          .first()
          .context("No image name could be created")
          .context("Failed to create image name")?;
        let version = if version.is_none() {
          build.config.version
        } else {
          *version
        };
        let version_str = version.to_string();
        // Potentially add the build image_tag postfix
        let version_str = if build.config.image_tag.is_empty() {
          version_str
        } else {
          format!("{version_str}-{}", build.config.image_tag)
        };
        // replace image with corresponding build image.
        deployment.config.image = DeploymentImage::Image {
          image: format!("{image_name}:{version_str}"),
        };
        let first_registry = build
          .config
          .image_registry
          .first()
          .unwrap_or(ImageRegistryConfig::static_default());
        if first_registry.domain.is_empty() {
          (version, None)
        } else {
          let ImageRegistryConfig {
            domain, account, ..
          } = first_registry;
          if deployment.config.image_registry_account.is_empty() {
            deployment.config.image_registry_account =
              account.to_string();
          }
          let token = if !deployment
            .config
            .image_registry_account
            .is_empty()
          {
            registry_token(domain, &deployment.config.image_registry_account).await.with_context(
              || format!("Failed to get git token in call to db. Stopping run. | {domain} | {}", deployment.config.image_registry_account),
            )?
          } else {
            None
          };
          (version, token)
        }
      }
      DeploymentImage::Image { image } => {
        let domain = extract_registry_domain(image)?;
        let token = if !deployment
          .config
          .image_registry_account
          .is_empty()
        {
          registry_token(&domain, &deployment.config.image_registry_account).await.with_context(
            || format!("Failed to get git token in call to db. Stopping run. | {domain} | {}", deployment.config.image_registry_account),
          )?
        } else {
          None
        };
        (Version::default(), token)
      }
    };

    // interpolate variables / secrets, returning the sanitizing replacers to send to
    // periphery so it may sanitize the final command for safe logging (avoids exposing secret values)
    let secret_replacers = if !deployment.config.skip_secret_interp {
      let VariablesAndSecrets { variables, secrets } =
        get_variables_and_secrets().await?;

      let mut interpolator =
        Interpolator::new(Some(&variables), &secrets);

      interpolator
        .interpolate_deployment(&mut deployment)?
        .push_logs(&mut update.logs);

      interpolator.secret_replacers
    } else {
      Default::default()
    };

    update.version = version;
    update_update(update.clone()).await?;

    match swarm_or_server {
      SwarmOrServer::Swarm(swarm) => {
        match swarm_request(
          &swarm.config.server_ids,
          api::swarm::CreateSwarmService {
            deployment,
            registry_token,
            replacers: secret_replacers.into_iter().collect(),
          },
        )
        .await
        {
          Ok(log) => update.logs.push(log),
          Err(e) => {
            update.push_error_log(
              "Create Service",
              format_serror(&e.into()),
            );
          }
        };
        update_cache_for_swarm(&swarm, true).await;
      }
      SwarmOrServer::Server(server) => {
        match periphery_client(&server)
          .await?
          .request(api::container::RunContainer {
            deployment,
            stop_signal: self.stop_signal,
            stop_time: self.stop_time,
            registry_token,
            replacers: secret_replacers.into_iter().collect(),
          })
          .await
        {
          Ok(log) => update.logs.push(log),
          Err(e) => {
            update.push_error_log(
              "Deploy Container",
              format_serror(&e.into()),
            );
          }
        };
        update_cache_for_server(&server, true).await;
      }
    }

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

/// Wait this long after a pull to allow another pull through
const PULL_TIMEOUT: i64 = 5_000;
type ServerId = String;
type Image = String;
type PullCache = TimeoutCache<(ServerId, Image), Log>;

fn pull_cache() -> &'static PullCache {
  static PULL_CACHE: OnceLock<PullCache> = OnceLock::new();
  PULL_CACHE.get_or_init(Default::default)
}

#[instrument(
  "PullDeploymentInner",
  skip_all,
  fields(
    deployment = deployment.id,
    server = server.id
  )
)]
pub async fn pull_deployment_inner(
  deployment: Deployment,
  server: &Server,
) -> anyhow::Result<Log> {
  let (image, account, token) = match deployment.config.image {
    DeploymentImage::Build { build_id, version } => {
      let build = resource::get::<Build>(&build_id).await?;
      let image_names = build.get_image_names();
      let image_name = image_names
        .first()
        .context("No image name could be created")
        .context("Failed to create image name")?;
      let version = if version.is_none() {
        build.config.version.to_string()
      } else {
        version.to_string()
      };
      // Potentially add the build image_tag postfix
      let version = if build.config.image_tag.is_empty() {
        version
      } else {
        format!("{version}-{}", build.config.image_tag)
      };
      // replace image with corresponding build image.
      let image = format!("{image_name}:{version}");
      let first_registry = build
        .config
        .image_registry
        .first()
        .unwrap_or(ImageRegistryConfig::static_default());
      if first_registry.domain.is_empty() {
        (image, None, None)
      } else {
        let ImageRegistryConfig {
          domain, account, ..
        } = first_registry;
        let account =
          if deployment.config.image_registry_account.is_empty() {
            account
          } else {
            &deployment.config.image_registry_account
          };
        let token = if !account.is_empty() {
          registry_token(domain, account).await.with_context(
              || format!("Failed to get git token in call to db. Stopping run. | {domain} | {account}"),
            )?
        } else {
          None
        };
        (image, optional_string(account), token)
      }
    }
    DeploymentImage::Image { image } => {
      let domain = extract_registry_domain(&image)?;
      let token = if !deployment
        .config
        .image_registry_account
        .is_empty()
      {
        registry_token(&domain, &deployment.config.image_registry_account).await.with_context(
            || format!("Failed to get git token in call to db. Stopping run. | {domain} | {}", deployment.config.image_registry_account),
          )?
      } else {
        None
      };
      (
        image,
        optional_string(&deployment.config.image_registry_account),
        token,
      )
    }
  };

  // Acquire the pull lock for this image on the server
  let lock = pull_cache()
    .get_lock((server.id.clone(), image.clone()))
    .await;

  // Lock the path lock, prevents simultaneous pulls by
  // ensuring simultaneous pulls will wait for first to finish
  // and checking cached results.
  let mut locked = lock.lock().await;

  // Early return from cache if lasted pulled with PULL_TIMEOUT
  if locked.last_ts + PULL_TIMEOUT > komodo_timestamp() {
    return locked.clone_res();
  }

  let res = async {
    let log = match periphery_client(server)
      .await?
      .request(api::docker::PullImage {
        name: image,
        account,
        token,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error("Pull image", format_serror(&e.into())),
    };

    update_cache_for_server(server, true).await;
    anyhow::Ok(log)
  }
  .await;

  // Set the cache with results. Any other calls waiting on the lock will
  // then immediately also use this same result.
  locked.set(&res, komodo_timestamp());

  res
}

impl Resolve<ExecuteArgs> for PullDeployment {
  #[instrument(
    "PullDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("PullDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pulling = true)?;

    let mut update = update.clone();
    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = pull_deployment_inner(deployment, &server).await?;

    update.logs.push(log);
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StartDeployment {
  #[instrument(
    "StartDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("StartDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.starting = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::StartContainer {
        name: deployment.name,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "start container",
        format_serror(&e.context("failed to start container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RestartDeployment {
  #[instrument(
    "RestartDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("RestartDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.restarting = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::RestartContainer {
        name: deployment.name,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "restart container",
        format_serror(
          &e.context("failed to restart container").into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PauseDeployment {
  #[instrument(
    "PauseDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("PauseDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pausing = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::PauseContainer {
        name: deployment.name,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "pause container",
        format_serror(&e.context("failed to pause container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for UnpauseDeployment {
  #[instrument(
    "UnpauseDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("UnpauseDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.unpausing = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::UnpauseContainer {
        name: deployment.name,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "unpause container",
        format_serror(
          &e.context("failed to unpause container").into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StopDeployment {
  #[instrument(
    "StopDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
      signal = format!("{:?}", self.signal),
      time = self.time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("StopDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.stopping = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::StopContainer {
        name: deployment.name,
        signal: self
          .signal
          .unwrap_or(deployment.config.termination_signal)
          .into(),
        time: self
          .time
          .unwrap_or(deployment.config.termination_timeout)
          .into(),
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "stop container",
        format_serror(&e.context("failed to stop container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl super::BatchExecute for BatchDestroyDeployment {
  type Resource = Deployment;
  fn single_request(deployment: String) -> ExecuteRequest {
    ExecuteRequest::DestroyDeployment(DestroyDeployment {
      deployment,
      signal: None,
      time: None,
    })
  }
}

impl Resolve<ExecuteArgs> for BatchDestroyDeployment {
  #[instrument(
    "BatchDestroyDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      pattern = self.pattern,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, id, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDestroyDeployment>(
        &self.pattern,
        user,
      )
      .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for DestroyDeployment {
  #[instrument(
    "DestroyDeployment",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      deployment = self.deployment,
      signal = format!("{:?}", self.signal),
      time = self.time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (deployment, swarm_or_server) =
      setup_deployment_execution(&self.deployment, user).await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!("DestroyDeployment should not be called for Deployment in Swarm Mode")
          .status_code(StatusCode::BAD_REQUEST),
      );
    };

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .deployment
      .get_or_insert_default(&deployment.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.destroying = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let log = match periphery_client(&server)
      .await?
      .request(api::container::RemoveContainer {
        name: deployment.name,
        signal: self
          .signal
          .unwrap_or(deployment.config.termination_signal)
          .into(),
        time: self
          .time
          .unwrap_or(deployment.config.termination_timeout)
          .into(),
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "stop container",
        format_serror(&e.context("failed to stop container").into()),
      ),
    };

    update.logs.push(log);
    update.finalize();
    update_cache_for_server(&server, true).await;
    update_update(update.clone()).await?;

    Ok(update)
  }
}
