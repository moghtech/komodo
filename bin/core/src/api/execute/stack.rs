use std::collections::HashSet;

use anyhow::Context;
use database::mungos::mongodb::bson::{doc, to_document};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::{
  api::{execute::*, write::RefreshStackCache},
  entities::{
    FileContents,
    permission::PermissionLevel,
    repo::Repo,
    server::Server,
    stack::{
      Stack, StackFileRequires, StackInfo, StackRemoteFileContents,
    },
    update::{Log, Update},
    user::User,
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
    update::{
      add_update_without_send, init_execution_update, update_update,
    },
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
          Some(
            file_contents
              .iter()
              .map(|f| FileContents {
                path: f.path.clone(),
                contents: f.contents.clone(),
              })
              .collect(),
          ),
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

    let action = match (
      &stack.info.deployed_contents,
      &stack.info.remote_contents,
    ) {
      (Some(deployed_contents), Some(latest_contents)) => {
        let services = stack
          .info
          .latest_services
          .iter()
          .map(|s| s.service_name.clone())
          .collect::<Vec<_>>();
        resolve_deploy_if_changed_action(
          deployed_contents,
          latest_contents,
          &services,
        )
      }
      (None, _) => DeployIfChangedAction::FullDeploy,
      _ => DeployIfChangedAction::Services {
        deploy: Vec::new(),
        restart: Vec::new(),
      },
    };

    let mut update = update.clone();

    match action {
      // Existing path pre 1.19.1
      DeployIfChangedAction::FullDeploy => {
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
      DeployIfChangedAction::FullRestart => {
        // For git repo based stacks, need to do a
        // PullStack in order to ensure latest repo contents on the
        // host before restart.
        maybe_pull_stack(&stack, Some(&mut update)).await?;

        // The existing update is initialized to DeployStack,
        // but also has not been created on database.
        // Setup a new update here.
        let req = ExecuteRequest::RestartStack(RestartStack {
          stack: stack.name,
          services: Vec::new(),
        });
        let update = init_execution_update(&req, user).await?;
        let ExecuteRequest::RestartStack(req) = req else {
          unreachable!()
        };
        req
          .resolve(&ExecuteArgs {
            user: user.clone(),
            update,
          })
          .await
      }
      DeployIfChangedAction::Services { deploy, restart } => {
        match (deploy.is_empty(), restart.is_empty()) {
          // Both empty, nothing to do
          (true, true) => {
            update.push_simple_log(
              "Diff compose files",
              String::from(
                "Deploy cancelled after no changes detected.",
              ),
            );
            update.finalize();
            Ok(update)
          }
          // Only restart
          (true, false) => {
            // For git repo based stacks, need to do a
            // PullStack in order to ensure latest repo contents on the
            // host before restart. Only necessary if no "deploys" (deploy already pulls stack).
            maybe_pull_stack(&stack, Some(&mut update)).await?;
            restart_services(stack.name, restart, user).await
          }
          // Only deploy
          (false, true) => {
            deploy_services(stack.name, deploy, user).await
          }
          // Deploy then restart, returning non-db update with executed services.
          (false, false) => {
            update.push_simple_log(
              "Execute Deploys",
              format!("Deploying: {}", deploy.join(", "),),
            );
            let deploy_update =
              deploy_services(stack.name.clone(), deploy, user)
                .await?;
            if !deploy_update.success {
              update.push_error_log(
                "Execute Deploys",
                String::from("There was a failure in service deploy"),
              );
              update.finalize();
              return Ok(update);
            }

            update.push_simple_log(
              "Execute Restarts",
              format!("Restarting: {}", restart.join(", "),),
            );
            let restart_update =
              restart_services(stack.name, restart, user).await?;
            if !restart_update.success {
              update.push_error_log(
                "Execute Restarts",
                String::from(
                  "There was a failure in a service restart",
                ),
              );
            }

            update.finalize();
            Ok(update)
          }
        }
      }
    }
  }
}

async fn deploy_services(
  stack: String,
  services: Vec<String>,
  user: &User,
) -> serror::Result<Update> {
  // The existing update is initialized to DeployStack,
  // but also has not been created on database.
  // Setup a new update here.
  let req = ExecuteRequest::DeployStack(DeployStack {
    stack,
    services,
    stop_time: None,
  });
  let update = init_execution_update(&req, user).await?;
  let ExecuteRequest::DeployStack(req) = req else {
    unreachable!()
  };
  req
    .resolve(&ExecuteArgs {
      user: user.clone(),
      update,
    })
    .await
}

async fn restart_services(
  stack: String,
  services: Vec<String>,
  user: &User,
) -> serror::Result<Update> {
  // The existing update is initialized to DeployStack,
  // but also has not been created on database.
  // Setup a new update here.
  let req =
    ExecuteRequest::RestartStack(RestartStack { stack, services });
  let update = init_execution_update(&req, user).await?;
  let ExecuteRequest::RestartStack(req) = req else {
    unreachable!()
  };
  req
    .resolve(&ExecuteArgs {
      user: user.clone(),
      update,
    })
    .await
}

enum DeployIfChangedAction {
  /// Changes to any compose or env files
  /// always lead to this.
  FullDeploy,
  /// If the above is not met, then changes to
  /// any changed additional file with `requires = "Restart"`
  /// and empty services array will lead to this.
  FullRestart,
  /// If all changed additional files have specific services
  /// they depend on, collect the final necessary
  /// services to deploy / restart.
  /// If eg `deploy` is empty, no services will be redeployed, same for `restart`.
  /// If both are empty, nothing is to be done.
  Services {
    deploy: Vec<String>,
    restart: Vec<String>,
  },
}

fn resolve_deploy_if_changed_action(
  deployed_contents: &[FileContents],
  latest_contents: &[StackRemoteFileContents],
  all_services: &[String],
) -> DeployIfChangedAction {
  let mut full_restart = false;
  let mut deploy = HashSet::<String>::new();
  let mut restart = HashSet::<String>::new();

  for latest in latest_contents {
    let Some(deployed) =
      deployed_contents.iter().find(|c| c.path == latest.path)
    else {
      // If file doesn't exist in deployed contents, do full
      // deploy to align this.
      return DeployIfChangedAction::FullDeploy;
    };
    // Ignore unchanged files
    if latest.contents == deployed.contents {
      continue;
    }
    match (latest.requires, latest.services.is_empty()) {
      (StackFileRequires::Redeploy, true) => {
        // File has requires = "Redeploy" at global level.
        // Can do early return here.
        return DeployIfChangedAction::FullDeploy;
      }
      (StackFileRequires::Redeploy, false) => {
        // Requires redeploy on specific services
        deploy.extend(latest.services.clone());
      }
      (StackFileRequires::Restart, true) => {
        // Services empty -> Full restart
        full_restart = true;
      }
      (StackFileRequires::Restart, false) => {
        restart.extend(latest.services.clone());
      }
      (StackFileRequires::None, _) => {
        // File can be ignored even with changes.
        continue;
      }
    }
  }

  match (full_restart, deploy.is_empty()) {
    // Full restart required with NO deploys needed -> Full Restart
    (true, true) => DeployIfChangedAction::FullRestart,
    // Full restart required WITH deploys needed -> Deploy those, restart all others
    (true, false) => DeployIfChangedAction::Services {
      restart: all_services
        .iter()
        // Only keep ones that don't need deploy
        .filter(|&s| !deploy.contains(s))
        .cloned()
        .collect(),
      deploy: deploy.into_iter().collect(),
    },
    // No full restart needed -> Deploy / restart as. pickedup.
    (false, _) => DeployIfChangedAction::Services {
      deploy: deploy.into_iter().collect(),
      restart: restart.into_iter().collect(),
    },
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

async fn maybe_pull_stack(
  stack: &Stack,
  update: Option<&mut Update>,
) -> anyhow::Result<()> {
  if stack.config.files_on_host
    || (stack.config.repo.is_empty()
      && stack.config.linked_repo.is_empty())
  {
    // Not repo based, no pull necessary
    return Ok(());
  }
  let server =
    resource::get::<Server>(&stack.config.server_id).await?;
  let repo = if stack.config.repo.is_empty()
    && !stack.config.linked_repo.is_empty()
  {
    Some(resource::get::<Repo>(&stack.config.linked_repo).await?)
  } else {
    None
  };
  pull_stack_inner(stack.clone(), Vec::new(), &server, repo, update)
    .await?;
  Ok(())
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

    let git_token =
      stack_git_token(&mut stack, repo.as_mut()).await?;

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
