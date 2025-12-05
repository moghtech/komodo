use std::path::PathBuf;

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::{doc, to_document};
use formatting::format_serror;
use komodo_client::{
  api::write::*,
  entities::{
    FileContents, NoData, Operation, RepoExecutionArgs,
    all_logs_success,
    permission::PermissionLevel,
    repo::Repo,
    stack::{Stack, StackInfo},
    update::Update,
    user::stack_user,
  },
};
use periphery_client::api::compose::{
  GetComposeContentsOnHost, GetComposeContentsOnHostResponse,
  WriteComposeContentsToHost,
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::{
    query::get_swarm_or_server,
    stack_git_token, swarm_or_server_request,
    update::{add_update, make_update},
  },
  permission::get_check_permissions,
  resource,
  stack::{
    remote::{RemoteComposeContents, get_repo_compose_contents},
    services::extract_services_into_res,
  },
  state::db_client,
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
  ) -> serror::Result<Stack> {
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
  ) -> serror::Result<Stack> {
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
  ) -> serror::Result<Stack> {
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
  ) -> serror::Result<Stack> {
    Ok(resource::update::<Stack>(&self.id, self.config, user).await?)
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
  ) -> serror::Result<Update> {
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
  ) -> serror::Result<Update> {
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

    if stack.config.files_on_host {
      write_stack_file_contents_on_host(
        stack, file_path, contents, update,
      )
      .await
    } else {
      write_stack_file_contents_git(
        stack,
        &file_path,
        &contents,
        &user.username,
        update,
      )
      .await
    }
  }
}

#[instrument("WriteStackFileContentsOnHost", skip_all)]
async fn write_stack_file_contents_on_host(
  stack: Stack,
  file_path: String,
  contents: String,
  mut update: Update,
) -> serror::Result<Update> {
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
) -> serror::Result<Update> {
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
      update.push_error_log("Pull Repo", format_serror(&e.into()));
      update.finalize();
      return Ok(update);
    }
  };

  if !all_logs_success(&update.logs) {
    update.finalize();
    update.id = add_update(update.clone()).await?;
    return Ok(update);
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
  ) -> serror::Result<NoData> {
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
            &mut services,
          ) {
            warn!(
              "failed to extract stack services, things won't works correctly. stack: {} | {e:#}",
              stack.name
            );
          }
        }

        (services, Some(contents), Some(errors), None, None)
      } else {
        (vec![], None, None, None, None)
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
