use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

use anyhow::{Context, anyhow};
use database::{
  bson::to_bson,
  mungos::{
    by_id::update_one_by_id,
    mongodb::bson::{doc, to_document},
  },
};
use formatting::format_serror;
use futures_util::{StreamExt as _, stream::FuturesOrdered};
use komodo_client::{
  api::{execute::DeployStack, write::*},
  entities::{
    FileContents, NoData, Operation, RepoExecutionArgs,
    ResourceTarget, SwarmOrServer,
    alert::{Alert, AlertData, SeverityLevel},
    all_logs_success, komodo_timestamp,
    permission::PermissionLevel,
    repo::Repo,
    stack::{Stack, StackInfo, StackServiceWithUpdate, StackState},
    update::Update,
    user::{auto_redeploy_user, stack_user, system_user},
  },
};
use mogh_cache::SetCache;
use mogh_resolver::Resolve;
use periphery_client::api::compose::{
  GetComposeContentsOnHost, GetComposeContentsOnHostResponse,
  WriteComposeContentsToHost,
};

use crate::{
  alert::send_alerts,
  api::execute::{self, ExecuteRequest, ExecutionResult},
  config::core_config,
  helpers::{
    query::get_swarm_or_server,
    stack_git_token, swarm_or_server_request,
    update::{add_update, make_update, poll_update_until_complete},
  },
  permission::get_check_permissions,
  resource::{self, list_full_for_user_using_pattern},
  stack::{
    remote::{RemoteComposeContents, get_repo_compose_contents},
    services::{
      extract_services_from_stack, extract_services_into_res,
    },
    setup_stack_execution,
  },
  state::{db_client, image_digest_cache, stack_status_cache},
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateStack {
  #[instrument(
    "CreateStack",
    skip_all,
    fields(
      operator = user.id,
      stack = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Stack> {
    resource::create::<Stack>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyStack {
  #[instrument(
    "CopyStack",
    skip_all,
    fields(
      operator = user.id,
      stack = self.name,
      copy_stack = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Stack> {
    let Stack { config, .. } = get_check_permissions::<Stack>(
      &self.id,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    resource::create::<Stack>(&self.name, config.into(), None, user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteStack {
  #[instrument(
    "DeleteStack",
    skip_all,
    fields(
      operator = user.id,
      stack = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Stack> {
    Ok(resource::delete::<Stack>(&self.id, user).await?)
  }
}

impl Resolve<WriteArgs> for UpdateStack {
  #[instrument(
    "UpdateStack",
    skip_all,
    fields(
      operator = user.id,
      stack = self.id,
      update = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Stack> {
    let compose_update = self.config.linked_repo.is_some()
      || self.config.repo.is_some()
      || self.config.files_on_host.is_some()
      || self.config.file_contents.is_some();

    let stack =
      resource::update::<Stack>(&self.id, self.config, user).await?;

    if compose_update {
      tokio::spawn(async move {
        let _ = (CheckStackForUpdate {
          stack: self.id,
          skip_auto_update: false,
          wait_for_auto_update: false,
          skip_cache_refresh: true,
        })
        .resolve(&WriteArgs {
          user: system_user().to_owned(),
        })
        .await;
      });
    }

    Ok(stack)
  }
}

impl Resolve<WriteArgs> for RenameStack {
  #[instrument(
    "RenameStack",
    skip_all,
    fields(
      operator = user.id,
      stack = self.id,
      new_name = self.name
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    Ok(resource::rename::<Stack>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for WriteStackFileContents {
  #[instrument(
    "WriteStackFileContents",
    skip_all,
    fields(
      operator = user.id,
      stack = self.stack,
      path = self.file_path,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    let WriteStackFileContents {
      stack,
      file_path,
      contents,
    } = self;
    let stack = get_check_permissions::<Stack>(
      &stack,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if !stack.config.files_on_host
      && stack.config.repo.is_empty()
      && stack.config.linked_repo.is_empty()
    {
      return Err(anyhow!(
        "Stack is not configured to use Files on Host, Git Repo, or Linked Repo, can't write file contents"
      ).into());
    }

    let mut update =
      make_update(&stack, Operation::WriteStackContents, user);

    update.push_simple_log("File contents to write", &contents);

    let id = stack.id.clone();

    let update = if stack.config.files_on_host {
      write_stack_file_contents_on_host(
        stack, file_path, contents, update,
      )
      .await?
    } else {
      write_stack_file_contents_git(
        stack,
        &file_path,
        &contents,
        &user.username,
        update,
      )
      .await?
    };

    tokio::spawn(async move {
      let _ = (CheckStackForUpdate {
        stack: id,
        skip_auto_update: false,
        wait_for_auto_update: false,
        skip_cache_refresh: true,
      })
      .resolve(&WriteArgs {
        user: system_user().to_owned(),
      })
      .await;
    });

    Ok(update)
  }
}

#[instrument("WriteStackFileContentsOnHost", skip_all)]
async fn write_stack_file_contents_on_host(
  stack: Stack,
  file_path: String,
  contents: String,
  mut update: Update,
) -> mogh_error::Result<Update> {
  let swarm_or_server = get_swarm_or_server(
    &stack.config.swarm_id,
    &stack.config.server_id,
  )
  .await?;

  let res = swarm_or_server_request(
    &swarm_or_server,
    WriteComposeContentsToHost {
      name: stack.name,
      run_directory: stack.config.run_directory,
      file_path,
      contents,
    },
  )
  .await;

  match res {
    Ok(log) => {
      update.logs.push(log);
    }
    Err(e) => {
      update.push_error_log(
        "Write File Contents",
        format_serror(&e.into()),
      );
    }
  }

  if !all_logs_success(&update.logs) {
    update.finalize();
    update.id = add_update(update.clone()).await?;
    return Ok(update);
  }

  // Finish with a cache refresh
  if let Err(e) = (RefreshStackCache { stack: stack.id })
    .resolve(&WriteArgs {
      user: stack_user().to_owned(),
    })
    .await
    .map_err(|e| e.error)
    .context(
      "Failed to refresh stack cache after writing file contents",
    )
  {
    update.push_error_log(
      "Refresh stack cache",
      format_serror(&e.into()),
    );
  }

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

#[instrument("WriteStackFileContentsGit", skip_all)]
async fn write_stack_file_contents_git(
  mut stack: Stack,
  file_path: &str,
  contents: &str,
  username: &str,
  mut update: Update,
) -> mogh_error::Result<Update> {
  let mut repo = if !stack.config.linked_repo.is_empty() {
    crate::resource::get::<Repo>(&stack.config.linked_repo)
      .await?
      .into()
  } else {
    None
  };
  let git_token = stack_git_token(&mut stack, repo.as_mut()).await?;

  let mut repo_args: RepoExecutionArgs = if let Some(repo) = &repo {
    repo.into()
  } else {
    (&stack).into()
  };
  let root = repo_args.unique_path(&core_config().repo_directory)?;
  repo_args.destination = Some(root.display().to_string());

  let file_path = stack
    .config
    .run_directory
    .parse::<PathBuf>()
    .context("Run directory is not a valid path")?
    .join(file_path);
  let full_path =
    root.join(&file_path).components().collect::<PathBuf>();

  if let Some(parent) = full_path.parent() {
    tokio::fs::create_dir_all(parent).await.with_context(|| {
      format!(
        "Failed to initialize stack file parent directory {parent:?}"
      )
    })?;
  }

  // Probe the remote first so an empty remote (or missing branch) can be
  // bootstrapped locally instead of failing on `git pull`, and an
  // unreachable remote surfaces an actionable error before any local work.
  let remote_state =
    match git::check_remote_state(&repo_args, git_token.as_deref())
      .await
      .context("Failed to check remote state")
    {
      Ok(state) => state,
      Err(e) => {
        update
          .push_error_log("Check Remote", format_serror(&e.into()));
        update.finalize();
        update.id = add_update(update.clone()).await?;
        return Ok(update);
      }
    };

  if let git::RemoteState::Unreachable { stderr } = &remote_state {
    update.push_error_log(
      "Check Remote",
      format!(
        "Remote not reachable: {}/{}\n\n{}\n\n\
         Create the repo or fix credentials, then refresh this stack.",
        repo_args.provider,
        repo_args.repo.as_deref().unwrap_or(""),
        stderr.trim(),
      ),
    );
    update.finalize();
    update.id = add_update(update.clone()).await?;
    return Ok(update);
  }

  // Ensure the folder is initialized as git repo.
  // This allows a new file to be committed on a branch that may not exist.
  if !root.join(".git").exists() {
    git::init_folder_as_repo(
      &root,
      &repo_args,
      git_token.as_deref(),
      &mut update.logs,
    )
    .await;

    if !all_logs_success(&update.logs) {
      update.finalize();
      update.id = add_update(update.clone()).await?;
      return Ok(update);
    }
  }

  // Save this for later -- repo_args moved next.
  let branch = repo_args.branch.clone();

  match remote_state {
    git::RemoteState::BranchExists => {
      // Pull latest changes to repo to ensure linear commit history
      match git::pull_or_clone(
        repo_args,
        &core_config().repo_directory,
        git_token,
      )
      .await
      .context("Failed to pull latest changes before commit")
      {
        Ok((res, _)) => update.logs.extend(res.logs),
        Err(e) => {
          update
            .push_error_log("Pull Repo", format_serror(&e.into()));
          update.finalize();
          return Ok(update);
        }
      };

      if !all_logs_success(&update.logs) {
        update.finalize();
        update.id = add_update(update.clone()).await?;
        return Ok(update);
      }
    }
    git::RemoteState::Empty => {
      update.push_simple_log(
        "Check Remote",
        format!(
          "Remote is empty. Bootstrapping branch '{branch}' from \
           this initial commit."
        ),
      );
    }
    git::RemoteState::BranchMissing { available_branches } => {
      update.push_simple_log(
        "Check Remote",
        format!(
          "Branch '{branch}' does not exist on remote. Available \
           branches: [{}]. Bootstrapping branch '{branch}' from \
           this initial commit.",
          available_branches.join(", "),
        ),
      );
    }
    git::RemoteState::Unreachable { .. } => unreachable!(),
  }

  if let Err(e) = tokio::fs::write(&full_path, &contents)
    .await
    .with_context(|| {
      format!(
        "Failed to write compose file contents to {full_path:?}"
      )
    })
  {
    update.push_error_log("Write File", format_serror(&e.into()));
  } else {
    update.push_simple_log(
      "Write File",
      format!("File written to {full_path:?}"),
    );
  };

  if !all_logs_success(&update.logs) {
    update.finalize();
    update.id = add_update(update.clone()).await?;

    return Ok(update);
  }

  let commit_res = git::commit_file(
    &format!("{username}: Write Stack File"),
    &root,
    &file_path,
    &branch,
  )
  .await;

  update.logs.extend(commit_res.logs);

  // Finish with a cache refresh
  if let Err(e) = (RefreshStackCache { stack: stack.id })
    .resolve(&WriteArgs {
      user: stack_user().to_owned(),
    })
    .await
    .map_err(|e| e.error)
    .context(
      "Failed to refresh stack cache after writing file contents",
    )
  {
    update.push_error_log(
      "Refresh stack cache",
      format_serror(&e.into()),
    );
  }

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

impl Resolve<WriteArgs> for RefreshStackCache {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<NoData> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // stack should be able to do this.
    let stack = get_check_permissions::<Stack>(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
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

    let file_contents_empty = stack.config.file_contents.is_empty();
    let repo_empty =
      stack.config.repo.is_empty() && repo.as_ref().is_none();

    if !stack.config.files_on_host
      && file_contents_empty
      && repo_empty
    {
      // Nothing to do without one of these
      return Ok(NoData {});
    }

    let mut missing_files = Vec::new();
    let service_image_digests = stack
      .info
      .latest_services
      .iter()
      .filter_map(|s| {
        let digest = s.image_digest.clone()?;
        Some((s.service_name.clone(), digest))
      })
      .collect::<HashMap<_, _>>();

    let (
      latest_services,
      remote_contents,
      remote_errors,
      latest_hash,
      latest_message,
    ) = if stack.config.files_on_host {
      // =============
      // FILES ON HOST
      // =============
      if let Ok(swarm_or_server) = get_swarm_or_server(
        &stack.config.swarm_id,
        &stack.config.server_id,
      )
      .await
      {
        let GetComposeContentsOnHostResponse { contents, errors } =
          match swarm_or_server_request(
            &swarm_or_server,
            GetComposeContentsOnHost {
              file_paths: stack.all_file_dependencies(),
              name: stack.name.clone(),
              run_directory: stack.config.run_directory.clone(),
            },
          )
          .await
          {
            Ok(res) => res,
            Err(e) => GetComposeContentsOnHostResponse {
              contents: Default::default(),
              errors: vec![FileContents {
                path: stack.config.run_directory.clone(),
                contents: format_serror(&e.into()),
              }],
            },
          };
        let project_name = stack.project_name(true);

        let mut services = Vec::new();

        for contents in &contents {
          // Don't include additional files in service parsing
          if !stack.is_compose_file(&contents.path) {
            continue;
          }
          if let Err(e) = extract_services_into_res(
            &project_name,
            &contents.contents,
            &service_image_digests,
            &mut services,
          ) {
            warn!(
              stack = stack.id,
              stack_name = stack.name,
              "Failed to extract stack services | {e:#}",
            );
          }
        }

        (services, Some(contents), Some(errors), None, None)
      } else {
        // This path is reached if the swarm / server is not available.
        // It carries over the last successful poll.
        (
          stack.info.latest_services,
          stack.info.remote_contents,
          stack.info.remote_errors,
          // Files on host can set hash / message back to None.
          None,
          None,
        )
      }
    } else if !repo_empty {
      // ================
      // REPO BASED STACK
      // ================
      let RemoteComposeContents {
        successful: remote_contents,
        errored: remote_errors,
        hash: latest_hash,
        message: latest_message,
        ..
      } = get_repo_compose_contents(
        &stack,
        repo.as_ref(),
        Some(&mut missing_files),
      )
      .await?;

      let project_name = stack.project_name(true);

      let mut services = Vec::new();

      for contents in &remote_contents {
        // Don't include additional files in service parsing
        if !stack.is_compose_file(&contents.path) {
          continue;
        }
        if let Err(e) = extract_services_into_res(
          &project_name,
          &contents.contents,
          &service_image_digests,
          &mut services,
        ) {
          warn!(
            "failed to extract stack services, things won't works correctly. stack: {} | {e:#}",
            stack.name
          );
        }
      }

      (
        services,
        Some(remote_contents),
        Some(remote_errors),
        latest_hash,
        latest_message,
      )
    } else {
      // =============
      // UI BASED FILE
      // =============
      let mut services = Vec::new();
      if let Err(e) = extract_services_into_res(
        // this should latest (not deployed), so make the project name fresh.
        &stack.project_name(true),
        &stack.config.file_contents,
        &service_image_digests,
        &mut services,
      ) {
        warn!(
          "Failed to extract Stack services for {}, things may not work correctly. | {e:#}",
          stack.name
        );
        services.extend(stack.info.latest_services.clone());
      };
      (services, None, None, None, None)
    };

    let info = StackInfo {
      missing_files,
      deployed_services: stack.info.deployed_services.clone(),
      deployed_project_name: stack.info.deployed_project_name.clone(),
      deployed_contents: stack.info.deployed_contents.clone(),
      deployed_config: stack.info.deployed_config.clone(),
      deployed_hash: stack.info.deployed_hash.clone(),
      deployed_message: stack.info.deployed_message.clone(),
      latest_services,
      remote_contents,
      remote_errors,
      latest_hash,
      latest_message,
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

    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for CheckStackForUpdate {
  #[instrument(
    "CheckStackForUpdate",
    skip_all,
    fields(
      operator = user.id,
      stack = self.stack,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Self::Response> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // stack should be able to do this.
    let (stack, swarm_or_server) = setup_stack_execution(
      &self.stack,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    swarm_or_server.verify_has_target()?;

    check_stack_for_update_inner(
      stack.id,
      &swarm_or_server,
      self.skip_auto_update,
      self.wait_for_auto_update,
      self.skip_cache_refresh,
    )
    .await
    .map_err(Into::into)
  }
}

/// If it goes down the "update available" path,
/// only send alert if stack id + service is not in this cache.
/// If alert is sent, add ID to cache.
/// If later it goes down non "update available" path,
/// remove the id from cache, so next time it does another alert
/// will be sent.
fn stack_alert_sent_cache() -> &'static SetCache<(String, String)> {
  static CACHE: OnceLock<SetCache<(String, String)>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

/// First refresh stack cache, then save
/// latest available 'image_digest' for stack services
/// to database.
#[instrument(
  "CheckStackForUpdateInner",
  skip_all,
  fields(
    stack = stack,
  )
)]
pub async fn check_stack_for_update_inner(
  // ID or name.
  stack: String,
  swarm_or_server: &SwarmOrServer,
  skip_auto_update: bool,
  // Otherwise spawns task to run in background
  wait_for_auto_update: bool,
  skip_cache_refresh: bool,
) -> anyhow::Result<CheckStackForUpdateResponse> {
  if !skip_cache_refresh {
    (RefreshStackCache {
      stack: stack.clone(),
    })
    .resolve(&WriteArgs {
      user: system_user().to_owned(),
    })
    .await
    .map_err(|e| e.error)
    .context("Failed to refresh stack cache before update check")?;
  }

  // Query again after refresh
  let mut stack = resource::get::<Stack>(&stack).await?;

  let cache = image_digest_cache();

  for service in &mut stack.info.latest_services {
    // Prefer the image coming from deployed services
    // so it will be after any interpolation.
    let image = stack
      .info
      .deployed_services
      .as_ref()
      .and_then(|services| {
        services.iter().find_map(|deployed| {
          (deployed.service_name == service.service_name)
            .then_some(&deployed.image)
        })
      })
      .unwrap_or(&service.image);

    if image.is_empty() ||
      // Images with a hardcoded digest can't have update.
      image.contains('@')
    {
      service.image_digest = None;
      continue;
    }
    match cache.get(swarm_or_server, image, None, None).await {
      Ok(digest) => service.image_digest = Some(digest),
      Err(e) => {
        warn!(
          "Failed to check for update | Stack: {} | Service: {} | Error: {e:#}",
          stack.name, service.service_name
        );
        service.image_digest = None;
        continue;
      }
    };
  }

  let latest_services = to_bson(&stack.info.latest_services)
    .context("Failed to serialize stack latest services to BSON")?;

  update_one_by_id(
    &db_client().stacks,
    &stack.id,
    doc! { "$set": { "info.latest_services": latest_services } },
    None,
  )
  .await?;

  let alert_cache = stack_alert_sent_cache();

  let Some(status) = stack_status_cache().get(&stack.id).await else {
    alert_cache
      .retain(|(stack_id, _)| stack_id != &stack.id)
      .await;
    return Ok(CheckStackForUpdateResponse {
      services: extract_services_from_stack(&stack)
        .into_iter()
        .map(|service| StackServiceWithUpdate {
          service: service.service_name,
          image: service.image,
          update_available: false,
        })
        .collect(),
      stack: stack.id,
    });
  };

  let StackState::Running = status.curr.state else {
    alert_cache
      .retain(|(stack_id, _)| stack_id != &stack.id)
      .await;
    return Ok(CheckStackForUpdateResponse {
      stack: stack.id,
      services: status
        .curr
        .services
        .iter()
        .map(|service| StackServiceWithUpdate {
          service: service.service.clone(),
          image: service.image.clone(),
          update_available: false,
        })
        .collect(),
    });
  };

  let mut services = Vec::new();

  for service in status.curr.services.iter() {
    let mut service_with_update = StackServiceWithUpdate {
      service: service.service.clone(),
      image: service.image.clone(),
      update_available: false,
    };

    let Some(current_digests) = &service.image_digests else {
      services.push(service_with_update);
      continue;
    };

    let Some(latest_digest) =
      stack.info.latest_services.iter().find_map(|s| {
        if s.service_name == service.service {
          s.image_digest.clone()
        } else {
          None
        }
      })
    else {
      services.push(service_with_update);
      continue;
    };

    service_with_update.update_available =
      latest_digest.update_available(current_digests);

    if service_with_update.update_available
      && (skip_auto_update || !stack.config.auto_update)
      && !alert_cache
        .contains(&(stack.id.clone(), service.service.clone()))
        .await
    {
      // Send service update available alert
      alert_cache
        .insert((stack.id.clone(), service.service.clone()))
        .await;
      let ts = komodo_timestamp();
      let alert = Alert {
        id: Default::default(),
        ts,
        resolved: true,
        resolved_ts: ts.into(),
        level: SeverityLevel::Ok,
        target: ResourceTarget::Stack(stack.id.clone()),
        data: AlertData::StackImageUpdateAvailable {
          id: stack.id.clone(),
          name: stack.name.clone(),
          swarm_name: swarm_or_server
            .swarm_name()
            .map(str::to_string),
          swarm_id: swarm_or_server.swarm_id().map(str::to_string),
          server_name: swarm_or_server
            .server_name()
            .map(str::to_string),
          server_id: swarm_or_server.server_id().map(str::to_string),
          service: service.service.clone(),
          image: service.image.clone(),
        },
      };
      let res = db_client().alerts.insert_one(&alert).await;
      if let Err(e) = res {
        error!(
          "Failed to record StackImageUpdateAvailable to db | {e:#}"
        );
      }
      send_alerts(&[alert]).await;
    }

    services.push(service_with_update);
  }

  let services_with_update = services
    .iter()
    .filter(|service| service.update_available)
    .cloned()
    .collect::<Vec<_>>();

  if skip_auto_update
    || !stack.config.auto_update
    || services_with_update.is_empty()
  {
    return Ok(CheckStackForUpdateResponse {
      stack: stack.id,
      services,
    });
  }

  // Conservatively remove from alert cache so 'skip_auto_update'
  // doesn't cause alerts not to be sent on subsequent calls.
  alert_cache
    .retain(|(stack_id, _)| stack_id != &stack.id)
    .await;

  let deploy_services = if stack.config.auto_update_all_services
    // Swarm stacks don't support individual service deploy
    || !stack.config.swarm_id.is_empty()
  {
    Vec::new()
  } else {
    services_with_update
      .iter()
      .map(|service| service.service.clone())
      .collect()
  };

  let swarm_id = swarm_or_server.swarm_id().map(str::to_string);
  let swarm_name = swarm_or_server.swarm_name().map(str::to_string);
  let server_id = swarm_or_server.server_id().map(str::to_string);
  let server_name = swarm_or_server.server_name().map(str::to_string);

  let stack_id = stack.id.clone();

  let run = async move {
    match execute::inner_handler(
      ExecuteRequest::DeployStack(DeployStack {
        stack: stack.id.clone(),
        services: deploy_services,
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
        let Ok(update) = poll_update_until_complete(&update.id).await
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
            target: ResourceTarget::Stack(stack.id.clone()),
            data: AlertData::StackAutoUpdated {
              id: stack.id.clone(),
              name: stack.name.clone(),
              swarm_id,
              swarm_name,
              server_id,
              server_name,
              images: services_with_update
                .iter()
                .map(|service| service.image.clone())
                .collect(),
            },
          };
          let res = db_client().alerts.insert_one(&alert).await;
          if let Err(e) = res {
            error!("Failed to record StackAutoUpdated to db | {e:#}");
          }
          send_alerts(&[alert]).await;
        }
      }
      Err(e) => {
        warn!("Failed to auto update Stack {} | {e:#}", stack.name)
      }
    }
  };

  if wait_for_auto_update {
    run.await
  } else {
    tokio::spawn(run);
  }

  Ok(CheckStackForUpdateResponse {
    stack: stack_id,
    services,
  })
}

//

impl Resolve<WriteArgs> for BatchCheckStackForUpdate {
  #[instrument(
    "BatchCheckStackForUpdate",
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
    let stacks = list_full_for_user_using_pattern::<Stack>(
      &self.pattern,
      Default::default(),
      user,
      PermissionLevel::Execute.into(),
      &[],
    )
    .await?;

    let res = stacks
      .into_iter()
      .map(|stack| async move {
        let swarm_or_server = get_swarm_or_server(
          &stack.config.swarm_id,
          &stack.config.server_id,
        )
        .await?;
        swarm_or_server.verify_has_target().map_err(|e| e.error)?;
        check_stack_for_update_inner(
          stack.id,
          &swarm_or_server,
          self.skip_auto_update,
          self.wait_for_auto_update,
          self.skip_cache_refresh,
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
              "Failed to check stack for update in batch run | {e:#}"
            )
          })
          .ok()
      })
      .collect();
    Ok(res)
  }
}
