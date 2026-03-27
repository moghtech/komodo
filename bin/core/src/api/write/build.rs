use std::{path::PathBuf, time::Duration};

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::to_document;
use database::{
  mongo_indexed::doc, mungos::mongodb::bson::oid::ObjectId,
};
use formatting::format_serror;
use komodo_client::{
  api::write::*,
  entities::{
    FileContents, NoData, Operation, RepoExecutionArgs,
    all_logs_success,
    build::{Build, BuildInfo},
    builder::{Builder, BuilderConfig},
    permission::PermissionLevel,
    repo::Repo,
    server::ServerState,
    update::Update,
  },
};
use mogh_resolver::Resolve;
use periphery_client::api::build::{
  GetDockerfileContentsOnHost, WriteDockerfileContentsToHost,
};
use tokio::fs;

use crate::{
  config::core_config,
  connection::PeripheryConnectionArgs,
  helpers::{
    git_token, periphery_client,
    query::get_server_with_state,
    update::{add_update, make_update},
  },
  periphery::PeripheryClient,
  permission::get_check_permissions,
  resource,
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateBuild {
  #[instrument(
    "CreateBuild",
    skip_all,
    fields(
      operator = user.id,
      build = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Build> {
    resource::create::<Build>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyBuild {
  #[instrument(
    "CopyBuild",
    skip_all,
    fields(
      operator = user.id,
      build = self.name,
      copy_build = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Build> {
    let Build { mut config, .. } = get_check_permissions::<Build>(
      &self.id,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    // reset version to 0.0.0
    config.version = Default::default();
    resource::create::<Build>(&self.name, config.into(), None, user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteBuild {
  #[instrument(
    "DeleteBuild",
    skip_all,
    fields(
      operator = user.id,
      build = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Build> {
    Ok(resource::delete::<Build>(&self.id, user).await?)
  }
}

impl Resolve<WriteArgs> for UpdateBuild {
  #[instrument(
    "UpdateBuild",
    skip_all,
    fields(
      operator = user.id,
      build = self.id,
      update = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Build> {
    Ok(resource::update::<Build>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameBuild {
  #[instrument(
    "RenameBuild",
    skip_all,
    fields(
      operator = user.id,
      build = self.id,
      new_name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    Ok(resource::rename::<Build>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for WriteBuildFileContents {
  #[instrument(
    "WriteBuildFileContents",
    skip_all,
    fields(
      operator = args.user.id,
      build = self.build,
    )
  )]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    let build = get_check_permissions::<Build>(
      &self.build,
      &args.user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if !build.config.files_on_host
      && build.config.repo.is_empty()
      && build.config.linked_repo.is_empty()
    {
      return Err(anyhow!(
        "Build is not configured to use Files on Host or Git Repo, can't write dockerfile contents"
      ).into());
    }

    let mut update =
      make_update(&build, Operation::WriteDockerfile, &args.user);

    update.push_simple_log("Dockerfile to write", &self.contents);

    if build.config.files_on_host {
      match get_on_host_periphery(&build)
        .await?
        .request(WriteDockerfileContentsToHost {
          name: build.name,
          build_path: build.config.build_path,
          dockerfile_path: build.config.dockerfile_path,
          contents: self.contents,
        })
        .await
        .context("Failed to write dockerfile contents to host")
      {
        Ok(log) => {
          update.logs.push(log);
        }
        Err(e) => {
          update.push_error_log(
            "Write Dockerfile Contents",
            format_serror(&e.into()),
          );
        }
      };

      if !all_logs_success(&update.logs) {
        update.finalize();
        update.id = add_update(update.clone()).await?;

        return Ok(update);
      }

      if let Err(e) =
        (RefreshBuildCache { build: build.id }).resolve(args).await
      {
        update.push_error_log(
          "Refresh build cache",
          format_serror(&e.error.into()),
        );
      }

      update.finalize();
      update.id = add_update(update.clone()).await?;

      Ok(update)
    } else {
      write_dockerfile_contents_git(self, args, build, update).await
    }
  }
}

#[instrument("WriteDockerfileContentsGit", skip_all)]
async fn write_dockerfile_contents_git(
  req: WriteBuildFileContents,
  args: &WriteArgs,
  build: Build,
  mut update: Update,
) -> mogh_error::Result<Update> {
  let WriteBuildFileContents { build: _, contents } = req;

  let mut repo_args: RepoExecutionArgs = if !build
    .config
    .files_on_host
    && !build.config.linked_repo.is_empty()
  {
    (&crate::resource::get::<Repo>(&build.config.linked_repo).await?)
      .into()
  } else {
    (&build).into()
  };
  let root = repo_args.unique_path(&core_config().repo_directory)?;
  repo_args.destination = Some(root.display().to_string());

  let build_path = build
    .config
    .build_path
    .parse::<PathBuf>()
    .context("Invalid build path")?;
  let dockerfile_path = build
    .config
    .dockerfile_path
    .parse::<PathBuf>()
    .context("Invalid dockerfile path")?;

  let full_path = root.join(&build_path).join(&dockerfile_path);

  if let Some(parent) = full_path.parent() {
    fs::create_dir_all(parent).await.with_context(|| {
      format!(
        "Failed to initialize dockerfile parent directory {parent:?}"
      )
    })?;
  }

  let access_token = if let Some(account) = &repo_args.account {
    git_token(&repo_args.provider, account, |https| repo_args.https = https)
    .await
    .with_context(
      || format!("Failed to get git token in call to db. Stopping run. | {} | {account}", repo_args.provider),
    )?
  } else {
    None
  };

  // Ensure the folder is initialized as git repo.
  // This allows a new file to be committed on a branch that may not exist.
  if !root.join(".git").exists() {
    git::init_folder_as_repo(
      &root,
      &repo_args,
      access_token.as_deref(),
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
    access_token,
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

  if let Err(e) = mogh_secret_file::write_async(&full_path, &contents)
    .await
    .with_context(|| {
      format!("Failed to write dockerfile contents to {full_path:?}")
    })
  {
    update
      .push_error_log("Write Dockerfile", format_serror(&e.into()));
  } else {
    update.push_simple_log(
      "Write Dockerfile",
      format!("File written to {full_path:?}"),
    );
  };

  if !all_logs_success(&update.logs) {
    update.finalize();
    update.id = add_update(update.clone()).await?;

    return Ok(update);
  }

  let commit_res = git::commit_file(
    &format!("{}: Commit Dockerfile", args.user.username),
    &root,
    &build_path.join(&dockerfile_path),
    &branch,
  )
  .await;

  update.logs.extend(commit_res.logs);

  if let Err(e) = (RefreshBuildCache { build: build.name })
    .resolve(args)
    .await
  {
    update.push_error_log(
      "Refresh build cache",
      format_serror(&e.error.into()),
    );
  }

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

impl Resolve<WriteArgs> for RefreshBuildCache {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<NoData> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // build should be able to do this.
    let build = get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let repo = if !build.config.files_on_host
      && !build.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&build.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    let RemoteDockerfileContents {
      path,
      contents,
      error,
      hash,
      message,
    } = if build.config.files_on_host {
      // =============
      // FILES ON HOST
      // =============
      match get_on_host_dockerfile(&build).await {
        Ok(FileContents { path, contents }) => {
          RemoteDockerfileContents {
            path: Some(path),
            contents: Some(contents),
            ..Default::default()
          }
        }
        Err(e) => RemoteDockerfileContents {
          error: Some(format_serror(&e.into())),
          ..Default::default()
        },
      }
    } else if let Some(repo) = &repo {
      let Some(res) = get_git_remote(&build, repo.into()).await?
      else {
        // Nothing to do here
        return Ok(NoData {});
      };
      res
    } else if !build.config.repo.is_empty() {
      let Some(res) = get_git_remote(&build, (&build).into()).await?
      else {
        // Nothing to do here
        return Ok(NoData {});
      };
      res
    } else {
      // =============
      // UI BASED FILE
      // =============
      RemoteDockerfileContents::default()
    };

    let info = BuildInfo {
      last_built_at: build.info.last_built_at,
      built_hash: build.info.built_hash,
      built_message: build.info.built_message,
      built_contents: build.info.built_contents,
      remote_path: path,
      remote_contents: contents,
      remote_error: error,
      latest_hash: hash,
      latest_message: message,
    };

    let info = to_document(&info)
      .context("failed to serialize build info to bson")?;

    db_client()
      .builds
      .update_one(
        doc! { "name": &build.name },
        doc! { "$set": { "info": info } },
      )
      .await
      .context("failed to update build info on db")?;

    Ok(NoData {})
  }
}

async fn get_on_host_periphery(
  build: &Build,
) -> anyhow::Result<PeripheryClient> {
  if build.config.builder_id.is_empty() {
    return Err(anyhow!("No builder associated with build"));
  }

  let builder = resource::get::<Builder>(&build.config.builder_id)
    .await
    .context("Failed to get builder")?;

  match builder.config {
    BuilderConfig::Aws(_) => {
      Err(anyhow!("Files on host doesn't work with AWS builder"))
    }
    BuilderConfig::Url(config) => {
      // TODO: Ensure connection is actually established.
      // Builder id no good because it may be active for multiple connections.
      let periphery = PeripheryClient::new(
        PeripheryConnectionArgs::from_url_builder(
          &ObjectId::new().to_hex(),
          &config,
        ),
        config.insecure_tls,
      )
      .await?;
      // Poll for connection to be estalished
      let mut err = None;
      for _ in 0..10 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        match periphery.health_check().await {
          Ok(_) => return Ok(periphery),
          Err(e) => err = Some(e),
        };
      }
      Err(err.context("Missing error")?)
    }
    BuilderConfig::Server(config) => {
      if config.server_id.is_empty() {
        return Err(anyhow!(
          "Builder is type server, but has no server attached"
        ));
      }
      let (server, state) =
        get_server_with_state(&config.server_id).await?;
      if state != ServerState::Ok {
        return Err(anyhow!(
          "Builder server is disabled or not reachable"
        ));
      };
      periphery_client(&server).await
    }
  }
}

/// The successful case will be included as Some(remote_contents).
/// The error case will be included as Some(remote_error)
async fn get_on_host_dockerfile(
  build: &Build,
) -> anyhow::Result<FileContents> {
  get_on_host_periphery(build)
    .await?
    .request(GetDockerfileContentsOnHost {
      name: build.name.clone(),
      build_path: build.config.build_path.clone(),
      dockerfile_path: build.config.dockerfile_path.clone(),
    })
    .await
}

async fn get_git_remote(
  build: &Build,
  mut clone_args: RepoExecutionArgs,
) -> anyhow::Result<Option<RemoteDockerfileContents>> {
  if clone_args.provider.is_empty() {
    // Nothing to do here
    return Ok(None);
  }
  let config = core_config();
  let repo_path = clone_args.unique_path(&config.repo_directory)?;
  clone_args.destination = Some(repo_path.display().to_string());

  let access_token = if let Some(username) = &clone_args.account {
    git_token(&clone_args.provider, username, |https| {
          clone_args.https = https
        })
        .await
        .with_context(
          || format!("Failed to get git token in call to db. Stopping run. | {} | {username}", clone_args.provider),
        )?
  } else {
    None
  };

  let (res, _) = git::pull_or_clone(
    clone_args,
    &config.repo_directory,
    access_token,
  )
  .await
  .context("Failed to clone Build repo")?;

  // Ensure clone / pull successful,
  // propogate error log -> 'errored' and return.
  if let Some(failure) = res.logs.iter().find(|log| !log.success) {
    return Ok(Some(RemoteDockerfileContents {
      path: Some(format!("Failed at: {}", failure.stage)),
      error: Some(failure.combined()),
      ..Default::default()
    }));
  }

  let relative_path = PathBuf::from(&build.config.build_path)
    .join(&build.config.dockerfile_path);

  let full_path = repo_path.join(&relative_path);
  let (contents, error) =
    match fs::read_to_string(&full_path).await.with_context(|| {
      format!("Failed to read dockerfile contents at {full_path:?}")
    }) {
      Ok(contents) => (Some(contents), None),
      Err(e) => (None, Some(format_serror(&e.into()))),
    };
  Ok(Some(RemoteDockerfileContents {
    path: Some(relative_path.display().to_string()),
    contents,
    error,
    hash: res.commit_hash,
    message: res.commit_message,
  }))
}

#[derive(Default)]
pub struct RemoteDockerfileContents {
  pub path: Option<String>,
  pub contents: Option<String>,
  pub error: Option<String>,
  pub hash: Option<String>,
  pub message: Option<String>,
}
