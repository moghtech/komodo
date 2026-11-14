use std::{collections::HashSet, str::FromStr};

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::{
  doc, oid::ObjectId, to_bson, to_document,
};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::{
  api::{execute::*, write::RefreshStackCache},
  entities::{
    FileContents, SwarmOrServer,
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
use mogh_error::AddStatusCodeError as _;
use mogh_resolver::Resolve;
use periphery_client::api::{
  DeployStackResponse, compose::*, swarm::DeploySwarmStack,
};
use reqwest::StatusCode;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
  api::write::WriteArgs,
  helpers::{
    periphery_client,
    query::{
      VariablesAndSecrets, get_swarm_or_server,
      get_variables_and_secrets,
    },
    stack_git_token,
    swarm::swarm_request,
    update::{
      add_update_without_send, init_execution_update, update_update,
    },
  },
  monitor::{refresh_server_cache, refresh_swarm_cache},
  permission::get_check_permissions,
  resource,
  secret_provider::{
    execution::{
      PeripheryStackSecretPayload, build_external_secrets_log,
      delete_resource_deployment_metadata,
      excluded_secret_paths_for_stack, filter_stack_remote_contents,
      get_resource_deployment_metadata, merge_secret_replacers,
      upsert_stack_deployment_metadata,
    },
    redaction::redact_compose_merged_config_yaml,
    resolver::{
      ResolvedStackSecrets, resolve_stack_secrets,
      stack_uses_external_secrets, validate_repo_no_vault_refs,
    },
  },
  stack::{
    execute::{
      execute_compose, execute_compose_with_stack_and_server,
    },
    setup_stack_execution,
  },
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
  #[instrument(
    "BatchDeployStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      pattern = self.pattern,
      tags = self.tags.join(","),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, task_id, .. }: &ExecuteArgs,
  ) -> mogh_error::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDeployStack>(
        &self.pattern,
        self.tags,
        user,
      )
      .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for DeployStack {
  #[instrument(
    "DeployStack",
    skip_all,
    fields(
      task_id = args.task_id.to_string(),
      operator = args.user.id,
      update_id = args.update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(
    self,
    args: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
    deploy_stack_with_pre_resolved(self, args, None).await
  }
}

async fn interpolate_stack_and_repo_for_execution(
  stack: &mut Stack,
  repo: &mut Option<Repo>,
  user: &User,
  is_swarm: bool,
  pre_resolved_secrets: Option<(ResolvedStackSecrets, Vec<Log>)>,
  mut logs: Option<&mut Vec<Log>>,
) -> anyhow::Result<(
  Vec<(String, String)>,
  Option<ResolvedStackSecrets>,
)> {
  let VariablesAndSecrets { variables, secrets } =
    if !stack.config.skip_secret_interp {
      get_variables_and_secrets().await?
    } else {
      Default::default()
    };

  let mut interpolator = Interpolator::new(
    (!stack.config.skip_secret_interp).then_some(&variables),
    &secrets,
  );

  let (resolved_ext, pre_interp_logs) = match pre_resolved_secrets {
    Some((pre, pre_logs)) => (Some(pre), pre_logs),
    None => {
      let ext = resolve_stack_secrets(
        stack,
        user,
        Some(PermissionLevel::Execute),
        is_swarm,
        &mut interpolator,
        &CancellationToken::new(),
      )
      .await?;
      (ext, Vec::new())
    }
  };

  if let Some(out_logs) = logs.as_deref_mut() {
    out_logs.extend(pre_interp_logs);
  }

  if !stack.config.skip_secret_interp {
    if let Some(ext) = &resolved_ext {
      stack.config.environment.clear();
      interpolator.interpolate_stack(stack)?;
      stack.config.environment =
        ext.remaining_standard_environment.clone();
    } else {
      interpolator.interpolate_stack(stack)?;
    }
    if let Some(repo) = repo.as_mut() {
      validate_repo_no_vault_refs(repo)?;
      if !repo.config.skip_secret_interp {
        interpolator.interpolate_repo(repo)?;
      }
    }
    if let Some(out_logs) = logs.as_deref_mut() {
      interpolator.push_logs(out_logs);
    }
  } else if let Some(repo) = repo.as_ref() {
    validate_repo_no_vault_refs(repo)?;
  }

  if let Some(ext) = &resolved_ext
    && let Some(out_logs) = logs
  {
    out_logs.push(build_external_secrets_log(&ext.provenance));
  }

  let replacers = merge_secret_replacers(
    interpolator.secret_replacers,
    resolved_ext
      .as_ref()
      .map(|e| e.replacers.clone())
      .unwrap_or_default(),
  );

  Ok((replacers, resolved_ext))
}

async fn deploy_stack_with_pre_resolved(
  req: DeployStack,
  ExecuteArgs {
    user,
    update,
    task_id: _,
  }: &ExecuteArgs,
  pre_resolved_secrets: Option<(ResolvedStackSecrets, Vec<Log>)>,
) -> mogh_error::Result<Update> {
  let (mut stack, swarm_or_server) = setup_stack_execution(
    &req.stack,
    user,
    PermissionLevel::Execute.into(),
  )
  .await?;

  swarm_or_server.verify_has_target()?;

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
  let action_guard =
    action_state.update(|state| state.deploying = true)?;

  let mut update = update.clone();

  update_update(update.clone()).await?;

  if !req.services.is_empty() {
    update.logs.push(Log::simple(
      "Service/s",
      format!(
        "Execution requested for Stack service/s {}",
        req.services.join(", ")
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

  let is_swarm = matches!(swarm_or_server, SwarmOrServer::Swarm(_));
  let (secret_replacers, resolved_ext) =
    interpolate_stack_and_repo_for_execution(
      &mut stack,
      &mut repo,
      user,
      is_swarm,
      pre_resolved_secrets,
      Some(&mut update.logs),
    )
    .await?;

  let stack_secret_payload = resolved_ext
    .as_ref()
    .map(PeripheryStackSecretPayload::from)
    .unwrap_or_default();

  let DeployStackResponse {
    logs,
    deployed,
    services,
    mut file_contents,
    missing_files,
    mut remote_errors,
    mut merged_config,
    commit_hash,
    commit_message,
    active_secret_generation,
  } = match &swarm_or_server {
    SwarmOrServer::None => unreachable!(),
    SwarmOrServer::Swarm(swarm) => {
      swarm_request(
        &swarm.config.server_ids,
        DeploySwarmStack {
          stack: stack.clone(),
          repo,
          git_token,
          registry_token,
          replacers: secret_replacers,
        },
      )
      .await?
    }
    SwarmOrServer::Server(server) => {
      let periphery = periphery_client(server).await?;
      if resolved_ext.is_some() {
        periphery
          .require_confidential_external_secret_transport()
          .await?;
      }
      periphery
        .request(ComposeUp {
          stack: stack.clone(),
          services: req.services,
          repo,
          git_token,
          registry_token,
          replacers: secret_replacers,
          external_env_vars: stack_secret_payload.external_env_vars,
          service_env_bindings: stack_secret_payload
            .service_env_bindings,
          file_secret_bindings: stack_secret_payload
            .file_secret_bindings,
          secret_env_keys: stack_secret_payload.secret_env_keys,
        })
        .await?
    }
  };

  update.logs.extend(logs);

  let excluded_paths = excluded_secret_paths_for_stack(&stack).await;
  filter_stack_remote_contents(
    &mut file_contents,
    &mut remote_errors,
    &excluded_paths,
  );
  if let Some(ext) = &resolved_ext
    && let Some(cfg) = merged_config.as_mut()
  {
    *cfg =
      redact_compose_merged_config_yaml(cfg, &ext.secret_env_keys);
  }

  if deployed && let SwarmOrServer::Server(server) = &swarm_or_server
  {
    match &resolved_ext {
      Some(ext) => {
        if let Err(e) = upsert_stack_deployment_metadata(
          &stack,
          &server.id,
          ext,
          active_secret_generation,
        )
        .await
        {
          warn!(
            "Failed to persist SecretDeploymentMetadata for Stack {} | {e:#}",
            stack.id
          );
        }
      }
      None => {
        let _ =
          delete_resource_deployment_metadata("Stack", &stack.id)
            .await;
      }
    }
  }

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
        merged_config,
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
      .context("Failed to serialize stack info to bson")?;

    db_client()
      .stacks
      .update_one(
        doc! { "name": &stack.name },
        doc! { "$set": { "info": info } },
      )
      .await
      .context("Failed to update stack info on db")?;
    anyhow::Ok(())
  };

  // This will be weird with single service deploys. Come back to it.
  if let Err(e) = update_info.await {
    update.push_error_log(
      "Refresh Stack Info",
      format_serror(
        &e.context("Failed to refresh stack info on db").into(),
      ),
    )
  }

  // Ensure cached stack state up to date by updating server cache
  match swarm_or_server {
    SwarmOrServer::None => unreachable!(),
    SwarmOrServer::Swarm(swarm) => {
      refresh_swarm_cache(&swarm, true).await;
    }
    SwarmOrServer::Server(server) => {
      refresh_server_cache(&server, true).await;
    }
  }

  update.finalize();

  // Drop action guard before updating
  // clients to requery action state
  drop(action_guard);
  update_update(update.clone()).await?;

  Ok(update)
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
  #[instrument(
    "BatchDeployStackIfChanged",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      pattern = self.pattern,
      tags = self.tags.join(","),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, task_id, .. }: &ExecuteArgs,
  ) -> mogh_error::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchDeployStackIfChanged>(
        &self.pattern,
        self.tags,
        user,
      )
      .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for DeployStackIfChanged {
  #[instrument(
    "DeployStackIfChanged",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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

    let mut action = match (
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

    // Single-fetch external secret snapshot + keyed HMAC diff against deployed metadata
    let existing_secret_meta =
      get_resource_deployment_metadata("Stack", &stack.id).await?;
    let pre_resolved = if stack_uses_external_secrets(&stack)? {
      let swarm_or_server = get_swarm_or_server(
        &stack.config.swarm_id,
        &stack.config.server_id,
      )
      .await?;
      let is_swarm =
        matches!(swarm_or_server, SwarmOrServer::Swarm(_));
      let VariablesAndSecrets { variables, secrets } =
        if !stack.config.skip_secret_interp {
          get_variables_and_secrets().await?
        } else {
          Default::default()
        };
      let mut pre_interp = Interpolator::new(
        (!stack.config.skip_secret_interp).then_some(&variables),
        &secrets,
      );
      let resolved = resolve_stack_secrets(
        &stack,
        user,
        Some(PermissionLevel::Execute),
        is_swarm,
        &mut pre_interp,
        &CancellationToken::new(),
      )
      .await?;
      let mut pre_logs = Vec::new();
      if !stack.config.skip_secret_interp {
        pre_interp.push_logs(&mut pre_logs);
      }
      if let Some(ext) = resolved {
        let secret_changed = match &existing_secret_meta {
          None => true,
          Some(meta) => {
            meta.keyed_fingerprint_hmac != ext.keyed_fingerprint_hmac
              || meta.fingerprint_key_id != ext.fingerprint_key_id
          }
        };
        if secret_changed {
          action = DeployIfChangedAction::FullDeploy;
        }
        Some((ext, pre_logs))
      } else {
        None
      }
    } else {
      if existing_secret_meta.is_some() {
        // External secrets were removed since last deploy -> force FullDeploy to clean up
        action = DeployIfChangedAction::FullDeploy;
      }
      None
    };

    let mut update = update.clone();

    match action {
      // Existing path pre 1.19.1
      DeployIfChangedAction::FullDeploy => {
        // Don't actually send it here, let the handler send it after it can set action state.
        // This is usually done in crate::helpers::update::init_execution_update.
        update.id = add_update_without_send(&update).await?;

        deploy_stack_with_pre_resolved(
          DeployStack {
            stack: stack.name,
            services: Vec::new(),
            stop_time: self.stop_time,
          },
          &ExecuteArgs {
            user: user.clone(),
            update,
            task_id: *task_id,
          },
          pre_resolved,
        )
        .await
      }
      DeployIfChangedAction::FullRestart => {
        // For git repo based stacks, need to do a
        // PullStack in order to ensure latest repo contents on the
        // host before restart.
        maybe_pull_stack(
          &stack,
          user,
          pre_resolved,
          Some(&mut update),
        )
        .await?;

        let mut update =
          restart_services(stack.name, Vec::new(), user).await?;

        if update.success {
          // Need to update 'info.deployed_contents' with the
          // latest contents so next check doesn't read the same diff.
          update_deployed_contents_with_latest(
            &stack.id,
            stack.info.remote_contents,
            &mut update,
          )
          .await;
        }

        Ok(update)
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
            maybe_pull_stack(
              &stack,
              user,
              pre_resolved,
              Some(&mut update),
            )
            .await?;

            let mut update =
              restart_services(stack.name, restart, user).await?;

            if update.success {
              // Need to update 'info.deployed_contents' with the
              // latest contents so next check doesn't read the same diff.
              update_deployed_contents_with_latest(
                &stack.id,
                stack.info.remote_contents,
                &mut update,
              )
              .await;
            }

            Ok(update)
          }
          // Only deploy
          (false, true) => {
            deploy_services(stack.name, deploy, user, pre_resolved)
              .await
          }
          // Deploy then restart, returning non-db update with executed services.
          (false, false) => {
            update.push_simple_log(
              "Execute Deploys",
              format!("Deploying: {}", deploy.join(", "),),
            );
            // This already updates 'stack.info.deployed_services',
            // restart doesn't require this again.
            let deploy_update = deploy_services(
              stack.name.clone(),
              deploy,
              user,
              pre_resolved,
            )
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

#[instrument(
  "DeployStackServices",
  skip_all,
  fields(
    stack = stack,
    services = format!("{services:?}")
  )
)]
async fn deploy_services(
  stack: String,
  services: Vec<String>,
  user: &User,
  pre_resolved_secrets: Option<(ResolvedStackSecrets, Vec<Log>)>,
) -> mogh_error::Result<Update> {
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
  deploy_stack_with_pre_resolved(
    req,
    &ExecuteArgs {
      user: user.clone(),
      update,
      task_id: Uuid::new_v4(),
    },
    pre_resolved_secrets,
  )
  .await
}

#[instrument(
  "RestartStackServices",
  skip_all,
  fields(
    stack = stack,
    services = format!("{services:?}")
  )
)]
async fn restart_services(
  stack: String,
  services: Vec<String>,
  user: &User,
) -> mogh_error::Result<Update> {
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
      task_id: Uuid::new_v4(),
    })
    .await
}

/// This can safely be called in [DeployStackIfChanged]
/// when there are ONLY changes to config files requiring restart,
/// AFTER the restart has been successfully completed.
///
/// In the case the if changed action is not FullDeploy,
/// the only file diff possible is to config files.
/// Also note either full or service deploy will already update 'deployed_contents'
/// making this method unnecessary in those cases.
///
/// Changes to config files after restart is applied should
/// be taken as the deployed contents, otherwise next changed check
/// will restart service again for no reason.
#[instrument(
  "UpdateStackDeployedContents",
  skip_all,
  fields(stack = id)
)]
async fn update_deployed_contents_with_latest(
  id: &str,
  contents: Option<Vec<StackRemoteFileContents>>,
  update: &mut Update,
) {
  let Some(contents) = contents else {
    return;
  };
  let contents = contents
    .into_iter()
    .map(|f| FileContents {
      path: f.path,
      contents: f.contents,
    })
    .collect::<Vec<_>>();
  if let Err(e) = (async {
    let contents = to_bson(&contents)
      .context("Failed to serialize contents to bson")?;
    let id =
      ObjectId::from_str(id).context("Id is not valid ObjectId")?;
    db_client()
      .stacks
      .update_one(
        doc! { "_id": id },
        doc! { "$set": { "info.deployed_contents": contents } },
      )
      .await
      .context("Failed to update stack 'deployed_contents'")?;
    anyhow::Ok(())
  })
  .await
  {
    update.push_error_log(
      "Update content cache",
      format_serror(&e.into()),
    );
    update.finalize();
    let _ = update_update(update.clone()).await;
  }
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
  #[instrument(
    "BatchPullStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      pattern = self.pattern,
      tags = self.tags.join(","),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, task_id, .. }: &ExecuteArgs,
  ) -> mogh_error::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchPullStack>(
        &self.pattern,
        self.tags,
        user,
      )
      .await?,
    )
  }
}

async fn maybe_pull_stack(
  stack: &Stack,
  user: &User,
  pre_resolved_secrets: Option<(ResolvedStackSecrets, Vec<Log>)>,
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
  pull_stack_inner(
    stack.clone(),
    Vec::new(),
    &server,
    repo,
    user,
    pre_resolved_secrets,
    update,
  )
  .await?;
  Ok(())
}

#[instrument(
  "PullStackInner",
  skip_all,
  fields(
    stack = stack.id,
    services = format!("{services:?}"),
  )
)]
pub async fn pull_stack_inner(
  mut stack: Stack,
  services: Vec<String>,
  server: &Server,
  mut repo: Option<Repo>,
  user: &User,
  pre_resolved_secrets: Option<(ResolvedStackSecrets, Vec<Log>)>,
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

  let (secret_replacers, resolved_ext) =
    interpolate_stack_and_repo_for_execution(
      &mut stack,
      &mut repo,
      user,
      false,
      pre_resolved_secrets,
      update.as_deref_mut().map(|u| &mut u.logs),
    )
    .await?;

  let stack_secret_payload = resolved_ext
    .as_ref()
    .map(PeripheryStackSecretPayload::from)
    .unwrap_or_default();

  let periphery = periphery_client(server).await?;
  if resolved_ext.is_some() {
    periphery
      .require_confidential_external_secret_transport()
      .await?;
  }

  let res = periphery
    .request(ComposePull {
      stack,
      services,
      repo,
      git_token,
      registry_token,
      replacers: secret_replacers,
      external_env_vars: stack_secret_payload.external_env_vars,
      service_env_bindings: stack_secret_payload.service_env_bindings,
      file_secret_bindings: stack_secret_payload.file_secret_bindings,
      secret_env_keys: stack_secret_payload.secret_env_keys,
    })
    .await?;

  // Ensure cached stack state up to date by updating server cache
  refresh_server_cache(server, true).await;

  Ok(res)
}

impl Resolve<ExecuteArgs> for PullStack {
  #[instrument(
    "PullStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
    let (stack, swarm_or_server) = setup_stack_execution(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!(
          "PullStack should not be called for Stack in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

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
    let action_guard =
      action_state.update(|state| state.pulling = true)?;

    let mut update = update.clone();
    update_update(update.clone()).await?;

    let res = pull_stack_inner(
      stack,
      self.services,
      &server,
      repo,
      user,
      None,
      Some(&mut update),
    )
    .await?;

    update.logs.extend(res.logs);
    update.finalize();

    // Drop action guard before updating
    // clients to requery action state
    drop(action_guard);
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StartStack {
  #[instrument(
    "StartStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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
  #[instrument(
    "RestartStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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
  #[instrument(
    "PauseStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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
  #[instrument(
    "UnpauseStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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
  #[instrument(
    "StopStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
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
  #[instrument(
    "BatchDestroyStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      pattern = self.pattern,
      tags = self.tags.join(","),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, task_id, .. }: &ExecuteArgs,
  ) -> mogh_error::Result<BatchExecutionResponse> {
    super::batch_execute::<BatchDestroyStack>(
      &self.pattern,
      self.tags,
      user,
    )
    .await
    .map_err(Into::into)
  }
}

impl Resolve<ExecuteArgs> for DestroyStack {
  #[instrument(
    "DestroyStack",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      services = format!("{:?}", self.services),
      remove_orphans = self.remove_orphans,
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
    let (stack, swarm_or_server) = setup_stack_execution(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    swarm_or_server.verify_has_target()?;

    match swarm_or_server {
      SwarmOrServer::None => unreachable!(),
      SwarmOrServer::Swarm(swarm) => {
        if !self.services.is_empty() {
          return Err(
            anyhow!("Cannot destroy specific Stack services when in Swarm mode.")
              .status_code(StatusCode::BAD_REQUEST)
          );
        }

        // get the action state for the stack (or insert default).
        let action_state = action_states()
          .stack
          .get_or_insert_default(&stack.id)
          .await;

        // Will check to ensure stack not already busy before updating, and return Err if so.
        // The returned guard will set the action state back to default when dropped.
        let action_guard =
          action_state.update(|state| state.destroying = true)?;

        let mut update = update.clone();
        // Send update here for UI to recheck action state
        update_update(update.clone()).await?;

        match swarm_request(
          &swarm.config.server_ids,
          periphery_client::api::swarm::RemoveSwarmStacks {
            stacks: vec![stack.project_name(false)],
            detach: false,
          },
        )
        .await
        {
          Ok(log) => update.logs.push(log),
          Err(e) => update.push_error_log(
            "Destroy Stack",
            format_serror(
              &e.context("Failed to 'docker stack rm' on swarm")
                .into(),
            ),
          ),
        }

        // Ensure cached stack state up to date by updating swarm cache
        refresh_swarm_cache(&swarm, true).await;

        update.finalize();

        // Drop action guard before updating
        // clients to requery action state
        drop(action_guard);
        update_update(update.clone()).await?;

        Ok(update)
      }
      SwarmOrServer::Server(server) => {
        execute_compose_with_stack_and_server::<DestroyStack>(
          stack,
          server,
          self.services,
          |state| state.destroying = true,
          update.clone(),
          (self.stop_time, self.remove_orphans),
        )
        .await
        .map_err(Into::into)
      }
    }
  }
}

impl Resolve<ExecuteArgs> for RunStackService {
  #[instrument(
    "RunStackService",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      stack = self.stack,
      service = self.service,
      request = format!("{self:?}"),
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs {
      user,
      update,
      task_id,
    }: &ExecuteArgs,
  ) -> mogh_error::Result<Update> {
    let (mut stack, swarm_or_server) = setup_stack_execution(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!(
          "RunStackService should not be called for Stack in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

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

    let action_guard =
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

    let (secret_replacers, resolved_ext) =
      interpolate_stack_and_repo_for_execution(
        &mut stack,
        &mut repo,
        user,
        false,
        None,
        Some(&mut update.logs),
      )
      .await?;

    let stack_secret_payload = resolved_ext
      .as_ref()
      .map(PeripheryStackSecretPayload::from)
      .unwrap_or_default();

    let periphery = periphery_client(&server).await?;
    if resolved_ext.is_some() {
      periphery
        .require_confidential_external_secret_transport()
        .await?;
    }

    let log = periphery
      .request(ComposeRun {
        stack,
        repo,
        git_token,
        registry_token,
        replacers: secret_replacers,
        external_env_vars: stack_secret_payload.external_env_vars,
        service_env_bindings: stack_secret_payload
          .service_env_bindings,
        file_secret_bindings: stack_secret_payload
          .file_secret_bindings,
        secret_env_keys: stack_secret_payload.secret_env_keys,
        service: self.service,
        command: self.command,
        no_tty: self.no_tty,
        no_deps: self.no_deps,
        detach: self.detach,
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

    // Drop action guard before updating
    // clients to requery action state
    drop(action_guard);
    update_update(update.clone()).await?;

    Ok(update)
  }
}
