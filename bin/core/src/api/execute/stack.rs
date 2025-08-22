use anyhow::Context;
use database::mungos::mongodb::bson::{doc, to_document};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::{
  api::{execute::*, write::RefreshStackCache},
  entities::{
    permission::PermissionLevel,
    repo::Repo,
    server::Server,
    stack::{Stack, StackInfo},
    update::{Log, Update},
  },
};
use periphery_client::api::compose::*;
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  helpers::{
    periphery_client,
    query::{VariablesAndSecrets, get_variables_and_secrets},
    stack_git_token,
    update::{add_update_without_send, update_update},
  },
  monitor::update_cache_for_server,
  permission::get_check_permissions,
  resource,
  stack::{execute::execute_compose, get_stack_and_server},
  state::{action_states, db_client},
};

use super::{ExecuteArgs, ExecuteRequest};

impl super::BatchExecute for BatchDeployStack {
  type Resource = Stack;
  fn single_request(stack: String) -> ExecuteRequest {
    ExecuteRequest::DeployStack(DeployStack {
      stack,
      services: Vec::new(),
      stop_time: None,
    })
  }
}

impl Resolve<ExecuteArgs> for BatchDeployStack {
  #[instrument(name = "BatchDeployStack", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDeployStack>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for DeployStack {
  #[instrument(name = "DeployStack", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (mut stack, server) = get_stack_and_server(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
      true,
    )
    .await?;

    let mut repo = if !stack.config.files_on_host
      && !stack.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&stack.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    // get the action state for the stack (or insert default).
    let action_state =
      action_states().stack.get_or_insert_default(&stack.id).await;

    // Will check to ensure stack not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.deploying = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    if !self.services.is_empty() {
      update.logs.push(Log::simple(
        "Service/s",
        format!(
          "Execution requested for Stack service/s {}",
          self.services.join(", ")
        ),
      ))
    }

    let git_token =
      stack_git_token(&mut stack, repo.as_mut()).await?;

    let registry_token = crate::helpers::registry_token(
      &stack.config.registry_provider,
      &stack.config.registry_account,
    ).await.with_context(
      || format!("Failed to get registry token in call to db. Stopping run. | {} | {}", stack.config.registry_provider, stack.config.registry_account),
    )?;

    // interpolate variables / secrets, returning the sanitizing replacers to send to
    // periphery so it may sanitize the final command for safe logging (avoids exposing secret values)
    let secret_replacers = if !stack.config.skip_secret_interp {
      let VariablesAndSecrets { variables, secrets } =
        get_variables_and_secrets().await?;

      let mut interpolator =
        Interpolator::new(Some(&variables), &secrets);

      interpolator.interpolate_stack(&mut stack)?;
      if let Some(repo) = repo.as_mut()
        && !repo.config.skip_secret_interp
      {
        interpolator.interpolate_repo(repo)?;
      }
      interpolator.push_logs(&mut update.logs);

      interpolator.secret_replacers
    } else {
      Default::default()
    };

    let ComposeUpResponse {
      logs,
      deployed,
      services,
      file_contents,
      missing_files,
      remote_errors,
      compose_config,
      commit_hash,
      commit_message,
    } = periphery_client(&server)?
      .request(ComposeUp {
        stack: stack.clone(),
        services: self.services,
        repo,
        git_token,
        registry_token,
        replacers: secret_replacers.into_iter().collect(),
      })
      .await?;

    update.logs.extend(logs);

    let update_info = async {
      let latest_services = if services.is_empty() {
        // maybe better to do something else here for services.
        stack.info.latest_services.clone()
      } else {
        services
      };

      // This ensures to get the latest project name,
      // as it may have changed since the last deploy.
      let project_name = stack.project_name(true);

      let (
        deployed_services,
        deployed_contents,
        deployed_config,
        deployed_hash,
        deployed_message,
      ) = if deployed {
        (
          Some(latest_services.clone()),
          Some(file_contents.clone()),
          compose_config,
          commit_hash.clone(),
          commit_message.clone(),
        )
      } else {
        (
          stack.info.deployed_services,
          stack.info.deployed_contents,
          stack.info.deployed_config,
          stack.info.deployed_hash,
          stack.info.deployed_message,
        )
      };

      let info = StackInfo {
        missing_files,
        deployed_project_name: project_name.into(),
        deployed_services,
        deployed_contents,
        deployed_config,
        deployed_hash,
        deployed_message,
        latest_services,
        remote_contents: stack
          .config
          .file_contents
          .is_empty()
          .then_some(file_contents),
        remote_errors: stack
          .config
          .file_contents
          .is_empty()
          .then_some(remote_errors),
        latest_hash: commit_hash,
        latest_message: commit_message,
      };

      let info = to_document(&info)
        .context("failed to serialize stack info to bson")?;

      db_client()
        .stacks
        .update_one(
          doc! { "name": &stack.name },
          doc! { "$set": { "info": info } },
        )
        .await
        .context("failed to update stack info on db")?;
      anyhow::Ok(())
    };

    // This will be weird with single service deploys. Come back to it.
    if let Err(e) = update_info.await {
      update.push_error_log(
        "refresh stack info",
        format_serror(
          &e.context("failed to refresh stack info on db").into(),
        ),
      )
    }

    // Ensure cached stack state up to date by updating server cache
    update_cache_for_server(&server).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl super::BatchExecute for BatchDeployStackIfChanged {
  type Resource = Stack;
  fn single_request(stack: String) -> ExecuteRequest {
    ExecuteRequest::DeployStackIfChanged(DeployStackIfChanged {
      stack,
      stop_time: None,
    })
  }
}

impl Resolve<ExecuteArgs> for BatchDeployStackIfChanged {
  #[instrument(name = "BatchDeployStackIfChanged", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDeployStackIfChanged>(
        &self.pattern,
        user,
      )
      .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for DeployStackIfChanged {
  #[instrument(name = "DeployStackIfChanged", skip(user, update), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let stack = get_check_permissions::<Stack>(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;
    RefreshStackCache {
      stack: stack.id.clone(),
    }
    .resolve(&WriteArgs { user: user.clone() })
    .await?;
    let stack = resource::get::<Stack>(&stack.id).await?;
    let changed = match (
      &stack.info.deployed_contents,
      &stack.info.remote_contents,
    ) {
      (Some(deployed_contents), Some(latest_contents)) => {
        let changed = || {
          for latest in latest_contents {
            let Some(deployed) = deployed_contents
              .iter()
              .find(|c| c.path == latest.path)
            else {
              return true;
            };
            if latest.contents != deployed.contents {
              return true;
            }
          }
          false
        };
        changed()
      }
      (None, _) => true,
      _ => false,
    };

    let mut update = update.clone();

    if !changed {
      update.push_simple_log(
        "Diff compose files",
        String::from("Deploy cancelled after no changes detected."),
      );
      update.finalize();
      return Ok(update);
    }

    // Don't actually send it here, let the handler send it after it can set action state.
    // This is usually done in crate::helpers::update::init_execution_update.
    update.id = add_update_without_send(&update).await?;

    DeployStack {
      stack: stack.name,
      services: Vec::new(),
      stop_time: self.stop_time,
    }
    .resolve(&ExecuteArgs {
      user: user.clone(),
      update,
    })
    .await
  }
}

impl super::BatchExecute for BatchPullStack {
  type Resource = Stack;
  fn single_request(stack: String) -> ExecuteRequest {
    ExecuteRequest::PullStack(PullStack {
      stack,
      services: Vec::new(),
    })
  }
}

impl Resolve<ExecuteArgs> for BatchPullStack {
  #[instrument(name = "BatchPullStack", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchPullStack>(&self.pattern, user)
        .await?,
    )
  }
}

pub async fn pull_stack_inner(
  mut stack: Stack,
  services: Vec<String>,
  server: &Server,
  mut repo: Option<Repo>,
  mut update: Option<&mut Update>,
) -> anyhow::Result<ComposePullResponse> {
  if let Some(update) = update.as_mut()
    && !services.is_empty()
  {
    update.logs.push(Log::simple(
      "Service/s",
      format!(
        "Execution requested for Stack service/s {}",
        services.join(", ")
      ),
    ))
  }

  let git_token = stack_git_token(&mut stack, repo.as_mut()).await?;

  let registry_token = crate::helpers::registry_token(
      &stack.config.registry_provider,
      &stack.config.registry_account,
    ).await.with_context(
      || format!("Failed to get registry token in call to db. Stopping run. | {} | {}", stack.config.registry_provider, stack.config.registry_account),
    )?;

  // interpolate variables / secrets
  let secret_replacers = if !stack.config.skip_secret_interp {
    let VariablesAndSecrets { variables, secrets } =
      get_variables_and_secrets().await?;

    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolator.interpolate_stack(&mut stack)?;
    if let Some(repo) = repo.as_mut()
      && !repo.config.skip_secret_interp
    {
      interpolator.interpolate_repo(repo)?;
    }
    if let Some(update) = update {
      interpolator.push_logs(&mut update.logs);
    }
    interpolator.secret_replacers
  } else {
    Default::default()
  };

  let res = periphery_client(server)?
    .request(ComposePull {
      stack,
      services,
      repo,
      git_token,
      registry_token,
      replacers: secret_replacers.into_iter().collect(),
    })
    .await?;

  // Ensure cached stack state up to date by updating server cache
  update_cache_for_server(server).await;

  Ok(res)
}

impl Resolve<ExecuteArgs> for PullStack {
  #[instrument(name = "PullStack", skip(user, update), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (stack, server) = get_stack_and_server(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
      true,
    )
    .await?;

    let repo = if !stack.config.files_on_host
      && !stack.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&stack.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    // get the action state for the stack (or insert default).
    let action_state =
      action_states().stack.get_or_insert_default(&stack.id).await;

    // Will check to ensure stack not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pulling = true)?;

    let mut update = update.clone();
    update_update(update.clone()).await?;

    let res = pull_stack_inner(
      stack,
      self.services,
      &server,
      repo,
      Some(&mut update),
    )
    .await?;

    update.logs.extend(res.logs);
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StartStack {
  #[instrument(name = "StartStack", skip(user, update), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<StartStack>(
      &self.stack,
      self.services,
      user,
      |state| state.starting = true,
      update.clone(),
      (),
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for RestartStack {
  #[instrument(name = "RestartStack", skip(user, update), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<RestartStack>(
      &self.stack,
      self.services,
      user,
      |state| {
        state.restarting = true;
      },
      update.clone(),
      (),
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for PauseStack {
  #[instrument(name = "PauseStack", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<PauseStack>(
      &self.stack,
      self.services,
      user,
      |state| state.pausing = true,
      update.clone(),
      (),
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for UnpauseStack {
  #[instrument(name = "UnpauseStack", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<UnpauseStack>(
      &self.stack,
      self.services,
      user,
      |state| state.unpausing = true,
      update.clone(),
      (),
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for StopStack {
  #[instrument(name = "StopStack", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<StopStack>(
      &self.stack,
      self.services,
      user,
      |state| state.stopping = true,
      update.clone(),
      self.stop_time,
    )
    .await
    .map_err(Into::into)
  }
}

impl super::BatchExecute for BatchDestroyStack {
  type Resource = Stack;
  fn single_request(stack: String) -> ExecuteRequest {
    ExecuteRequest::DestroyStack(DestroyStack {
      stack,
      services: Vec::new(),
      remove_orphans: false,
      stop_time: None,
    })
  }
}

impl Resolve<ExecuteArgs> for BatchDestroyStack {
  #[instrument(name = "BatchDestroyStack", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    super::batch_execute::<BatchDestroyStack>(&self.pattern, user)
      .await
      .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for DestroyStack {
  #[instrument(name = "DestroyStack", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    execute_compose::<DestroyStack>(
      &self.stack,
      self.services,
      user,
      |state| state.destroying = true,
      update.clone(),
      (self.stop_time, self.remove_orphans),
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for RunStackService {
  #[instrument(name = "RunStackService", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let (mut stack, server) = get_stack_and_server(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
      true,
    )
    .await?;

    let mut repo = if !stack.config.files_on_host
      && !stack.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&stack.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    let action_state =
      action_states().stack.get_or_insert_default(&stack.id).await;

    let _action_guard =
      action_state.update(|state| state.deploying = true)?;

    let mut update = update.clone();
    update_update(update.clone()).await?;

    let git_token = stack_git_token(&mut stack, repo.as_mut()).await?;

    let registry_token = crate::helpers::registry_token(
      &stack.config.registry_provider,
      &stack.config.registry_account,
    ).await.with_context(
      || format!("Failed to get registry token in call to db. Stopping run. | {} | {}", stack.config.registry_provider, stack.config.registry_account),
    )?;

    let secret_replacers = if !stack.config.skip_secret_interp {
      let VariablesAndSecrets { variables, secrets } =
        get_variables_and_secrets().await?;

      let mut interpolator =
        Interpolator::new(Some(&variables), &secrets);

      interpolator.interpolate_stack(&mut stack)?;
      if let Some(repo) = repo.as_mut()
        && !repo.config.skip_secret_interp
      {
        interpolator.interpolate_repo(repo)?;
      }
      interpolator.push_logs(&mut update.logs);

      interpolator.secret_replacers
    } else {
      Default::default()
    };

  let log = periphery_client(&server)?
      .request(ComposeRun {
        stack,
        repo,
        git_token,
        registry_token,
        replacers: secret_replacers.into_iter().collect(),
        service: self.service,
        command: self.command,
        no_tty: self.no_tty,
        no_deps: self.no_deps,
        service_ports: self.service_ports,
        env: self.env,
        workdir: self.workdir,
        user: self.user,
        entrypoint: self.entrypoint,
        pull: self.pull,
      })
      .await?;

    update.logs.push(log);
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
