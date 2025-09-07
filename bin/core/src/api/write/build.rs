use std::{path::PathBuf, str::FromStr, time::Duration};

use anyhow::{Context, anyhow};
use database::mongo_indexed::doc;
use database::mungos::mongodb::bson::to_document;
use formatting::format_serror;
use komodo_client::{
  api::write::*,
  entities::{
    FileContents, NoData, Operation, RepoExecutionArgs,
    all_logs_success,
    build::{Build, BuildInfo, PartialBuildConfig},
    builder::{Builder, BuilderConfig},
    config::core::CoreConfig,
    permission::PermissionLevel,
    repo::Repo,
    server::ServerState,
    update::Update,
  },
};
use octorust::types::{
  ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
};
use periphery_client::{
  PeripheryClient,
  api::build::{
    GetDockerfileContentsOnHost, WriteDockerfileContentsToHost,
  },
};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  config::core_config,
  helpers::{
    git_token, periphery_client,
    query::get_server_with_state,
    update::{add_update, make_update},
  },
  permission::get_check_permissions,
  resource,
  state::{db_client, github_client},
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateBuild {
  #[instrument(name = "CreateBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    resource::create::<Build>(&self.name, self.config, user).await
  }
}

impl Resolve<WriteArgs> for CopyBuild {
  #[instrument(name = "CopyBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    let Build { mut config, .. } = get_check_permissions::<Build>(
      &self.id,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    // reset version to 0.0.0
    config.version = Default::default();
    resource::create::<Build>(&self.name, config.into(), user).await
  }
}

impl Resolve<WriteArgs> for DeleteBuild {
  #[instrument(name = "DeleteBuild", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Build> {
    Ok(resource::delete::<Build>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateBuild {
  #[instrument(name = "UpdateBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    Ok(resource::update::<Build>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameBuild {
  #[instrument(name = "RenameBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Build>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for WriteBuildFileContents {
  #[instrument(name = "WriteBuildFileContents", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Update> {
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

async fn write_dockerfile_contents_git(
  req: WriteBuildFileContents,
  args: &WriteArgs,
  build: Build,
  mut update: Update,
) -> serror::Result<Update> {
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

  if let Err(e) =
    fs::write(&full_path, &contents).await.with_context(|| {
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
  #[instrument(
    name = "RefreshBuildCache",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<NoData> {
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

    let (
      remote_path,
      remote_contents,
      remote_error,
      latest_hash,
      latest_message,
    ) = if build.config.files_on_host {
      // =============
      // FILES ON HOST
      // =============
      match get_on_host_dockerfile(&build).await {
        Ok(FileContents { path, contents }) => {
          (Some(path), Some(contents), None, None, None)
        }
        Err(e) => {
          (None, None, Some(format_serror(&e.into())), None, None)
        }
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
      (None, None, None, None, None)
    };

    let info = BuildInfo {
      last_built_at: build.info.last_built_at,
      built_hash: build.info.built_hash,
      built_message: build.info.built_message,
      built_contents: build.info.built_contents,
      remote_path,
      remote_contents,
      remote_error,
      latest_hash,
      latest_message,
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
      let periphery = PeripheryClient::new(
        config.address,
        config.passkey,
        Duration::from_secs(3),
      );
      periphery.health_check().await?;
      Ok(periphery)
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
      periphery_client(&server)
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
) -> anyhow::Result<
  Option<(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
  )>,
> {
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
  .context("failed to clone build repo")?;

  let relative_path = PathBuf::from_str(&build.config.build_path)
    .context("Invalid build path")?
    .join(&build.config.dockerfile_path);

  let full_path = repo_path.join(&relative_path);
  let (contents, error) =
    match fs::read_to_string(&full_path).await.with_context(|| {
      format!("Failed to read dockerfile contents at {full_path:?}")
    }) {
      Ok(contents) => (Some(contents), None),
      Err(e) => (None, Some(format_serror(&e.into()))),
    };
  Ok(Some((
    Some(relative_path.display().to_string()),
    contents,
    error,
    res.commit_hash,
    res.commit_message,
  )))
}

impl Resolve<WriteArgs> for CreateBuildWebhook {
  #[instrument(name = "CreateBuildWebhook", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<CreateBuildWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let WriteArgs { user } = args;

    let build = get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if build.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = build.config.repo.split('/');
    let owner = split.next().context("Build repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo =
      split.next().context("Build repo has no repo after the /")?;

    let github_repos = github.repos();

    // First make sure the webhook isn't already created (inactive ones are ignored)
    let webhooks = github_repos
      .list_all_webhooks(owner, repo)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      webhook_secret,
      ..
    } = core_config();

    let webhook_secret = if build.config.webhook_secret.is_empty() {
      webhook_secret
    } else {
      &build.config.webhook_secret
    };

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = format!("{host}/listener/github/build/{}", build.id);

    for webhook in webhooks {
      if webhook.active && webhook.config.url == url {
        return Ok(NoData {});
      }
    }

    // Now good to create the webhook
    let request = ReposCreateWebhookRequest {
      active: Some(true),
      config: Some(ReposCreateWebhookRequestConfig {
        url,
        secret: webhook_secret.to_string(),
        content_type: String::from("json"),
        insecure_ssl: None,
        digest: Default::default(),
        token: Default::default(),
      }),
      events: vec![String::from("push")],
      name: String::from("web"),
    };
    github_repos
      .create_webhook(owner, repo, &request)
      .await
      .context("failed to create webhook")?;

    if !build.config.webhook_enabled {
      UpdateBuild {
        id: build.id,
        config: PartialBuildConfig {
          webhook_enabled: Some(true),
          ..Default::default()
        },
      }
      .resolve(args)
      .await
      .map_err(|e| e.error)
      .context("failed to update build to enable webhook")?;
    }

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for DeleteBuildWebhook {
  #[instrument(name = "DeleteBuildWebhook", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteBuildWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let build = get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if build.config.git_provider != "github.com" {
      return Err(
        anyhow!("Can only manage github.com repo webhooks").into(),
      );
    }

    if build.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't delete webhook").into(),
      );
    }

    let mut split = build.config.repo.split('/');
    let owner = split.next().context("Build repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo =
      split.next().context("Build repo has no repo after the /")?;

    let github_repos = github.repos();

    let webhooks = github_repos
      .list_all_webhooks(owner, repo)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      ..
    } = core_config();

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = format!("{host}/listener/github/build/{}", build.id);

    for webhook in webhooks {
      if webhook.active && webhook.config.url == url {
        github_repos
          .delete_webhook(owner, repo, webhook.id)
          .await
          .context("failed to delete webhook")?;
        return Ok(NoData {});
      }
    }

    // No webhook to delete, all good
    Ok(NoData {})
  }
}
