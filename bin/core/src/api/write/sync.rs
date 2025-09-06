use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::update_one_by_id,
  mongodb::bson::{doc, to_document},
};
use formatting::format_serror;
use komodo_client::{
  api::{read::ExportAllResourcesToToml, write::*},
  entities::{
    self, NoData, Operation, RepoExecutionArgs, ResourceTarget,
    action::Action,
    alert::{Alert, AlertData, SeverityLevel},
    alerter::Alerter,
    all_logs_success,
    build::Build,
    builder::Builder,
    config::core::CoreConfig,
    deployment::Deployment,
    komodo_timestamp,
    permission::PermissionLevel,
    procedure::Procedure,
    repo::Repo,
    server::Server,
    stack::Stack,
    sync::{
      PartialResourceSyncConfig, ResourceSync, ResourceSyncInfo,
      SyncDeployUpdate,
    },
    to_path_compatible_name,
    update::{Log, Update},
    user::sync_user,
  },
};
use octorust::types::{
  ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
};
use resolver_api::Resolve;

use crate::{
  alert::send_alerts,
  api::read::ReadArgs,
  config::core_config,
  helpers::{
    all_resources::AllResourcesById,
    git_token,
    query::get_id_to_tags,
    update::{add_update, make_update, update_update},
  },
  permission::get_check_permissions,
  resource,
  state::{db_client, github_client},
  sync::{
    deploy::SyncDeployParams, remote::RemoteResources,
    view::push_updates_for_view,
  },
};

use super::{WriteArgs, handle_resource_creation_error};

impl Resolve<WriteArgs> for CreateResourceSync {
  #[instrument(name = "CreateResourceSync", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ResourceSync> {
    match resource::create::<ResourceSync>(&self.name, self.config, user).await {
      Ok(sync) => Ok(sync),
      Err(e) => Err(handle_resource_creation_error(e))
    }
  }
}

impl Resolve<WriteArgs> for CopyResourceSync {
  #[instrument(name = "CopyResourceSync", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ResourceSync> {
    let ResourceSync { config, .. } =
      get_check_permissions::<ResourceSync>(
        &self.id,
        user,
        PermissionLevel::Write.into(),
      )
      .await?;
    match resource::create::<ResourceSync>(&self.name, config.into(), user).await {
      Ok(sync) => Ok(sync),
      Err(e) => Err(handle_resource_creation_error(e))
    }
  }
}

impl Resolve<WriteArgs> for DeleteResourceSync {
  #[instrument(name = "DeleteResourceSync", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<ResourceSync> {
    Ok(resource::delete::<ResourceSync>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateResourceSync {
  #[instrument(name = "UpdateResourceSync", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ResourceSync> {
    Ok(
      resource::update::<ResourceSync>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameResourceSync {
  #[instrument(name = "RenameResourceSync", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(
      resource::rename::<ResourceSync>(&self.id, &self.name, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for WriteSyncFileContents {
  #[instrument(name = "WriteSyncFileContents", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Update> {
    let sync = get_check_permissions::<ResourceSync>(
      &self.sync,
      &args.user,
      PermissionLevel::Write.into(),
    )
    .await?;

    let repo = if !sync.config.files_on_host
      && !sync.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&sync.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    if !sync.config.files_on_host
      && sync.config.repo.is_empty()
      && sync.config.linked_repo.is_empty()
    {
      return Err(
        anyhow!(
          "This method is only for 'files on host' or 'repo' based syncs."
        )
        .into(),
      );
    }

    let mut update =
      make_update(&sync, Operation::WriteSyncContents, &args.user);

    update.push_simple_log("File contents", &self.contents);

    if sync.config.files_on_host {
      write_sync_file_contents_on_host(self, args, sync, update).await
    } else {
      write_sync_file_contents_git(self, args, sync, repo, update)
        .await
    }
  }
}

async fn write_sync_file_contents_on_host(
  req: WriteSyncFileContents,
  args: &WriteArgs,
  sync: ResourceSync,
  mut update: Update,
) -> serror::Result<Update> {
  let WriteSyncFileContents {
    sync: _,
    resource_path,
    file_path,
    contents,
  } = req;

  let root = core_config()
    .sync_directory
    .join(to_path_compatible_name(&sync.name));
  let file_path =
    file_path.parse::<PathBuf>().context("Invalid file path")?;
  let resource_path = resource_path
    .parse::<PathBuf>()
    .context("Invalid resource path")?;
  let full_path = root.join(&resource_path).join(&file_path);

  if let Some(parent) = full_path.parent() {
    tokio::fs::create_dir_all(parent).await.with_context(|| {
      format!(
        "Failed to initialize resource file parent directory {parent:?}"
      )
    })?;
  }

  if let Err(e) = tokio::fs::write(&full_path, &contents)
    .await
    .with_context(|| {
      format!(
        "Failed to write resource file contents to {full_path:?}"
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

  if let Err(e) = (RefreshResourceSyncPending { sync: sync.name })
    .resolve(args)
    .await
  {
    update.push_error_log(
      "Refresh failed",
      format_serror(&e.error.into()),
    );
  }

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

async fn write_sync_file_contents_git(
  req: WriteSyncFileContents,
  args: &WriteArgs,
  sync: ResourceSync,
  repo: Option<Repo>,
  mut update: Update,
) -> serror::Result<Update> {
  let WriteSyncFileContents {
    sync: _,
    resource_path,
    file_path,
    contents,
  } = req;

  let mut repo_args: RepoExecutionArgs = if let Some(repo) = &repo {
    repo.into()
  } else {
    (&sync).into()
  };
  let root = repo_args.unique_path(&core_config().repo_directory)?;
  repo_args.destination = Some(root.display().to_string());

  let git_token = if let Some(account) = &repo_args.account {
    git_token(&repo_args.provider, account, |https| repo_args.https = https)
    .await
    .with_context(
      || format!("Failed to get git token in call to db. Stopping run. | {} | {account}", repo_args.provider),
    )?
  } else {
    None
  };

  let file_path =
    file_path.parse::<PathBuf>().with_context(|| {
      format!("File path is not a valid path: {file_path}")
    })?;
  let resource_path =
    resource_path.parse::<PathBuf>().with_context(|| {
      format!("Resource path is not a valid path: {resource_path}")
    })?;
  let full_path = root
    .join(&resource_path)
    .join(&file_path)
    .components()
    .collect::<PathBuf>();

  if let Some(parent) = full_path.parent() {
    tokio::fs::create_dir_all(parent).await.with_context(|| {
      format!(
        "Failed to initialize resource file parent directory {parent:?}"
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
        "Failed to write resource file contents to {full_path:?}"
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
    &format!("{}: Commit Resource File", args.user.username),
    &root,
    &resource_path.join(&file_path),
    &branch,
  )
  .await;

  update.logs.extend(commit_res.logs);

  if let Err(e) = (RefreshResourceSyncPending { sync: sync.name })
    .resolve(args)
    .await
    .map_err(|e| e.error)
    .context(
      "Failed to refresh sync pending after writing file contents",
    )
  {
    update.push_error_log(
      "Refresh sync pending",
      format_serror(&e.into()),
    );
  }

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

impl Resolve<WriteArgs> for CommitSync {
  #[instrument(name = "CommitSync", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Update> {
    let WriteArgs { user } = args;

    let sync = get_check_permissions::<entities::sync::ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    let repo = if !sync.config.files_on_host
      && !sync.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&sync.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    let file_contents_empty = sync.config.file_contents_empty();

    let fresh_sync = !sync.config.files_on_host
      && sync.config.repo.is_empty()
      && repo.is_none()
      && file_contents_empty;

    if !sync.config.managed && !fresh_sync {
      return Err(
        anyhow!("Cannot commit to sync. Enabled 'managed' mode.")
          .into(),
      );
    }

    // Get this here so it can fail before update created.
    let resource_path = if sync.config.files_on_host
      || !sync.config.repo.is_empty()
      || repo.is_some()
    {
      let resource_path = sync
        .config
        .resource_path
        .first()
        .context("Sync does not have resource path configured.")?
        .parse::<PathBuf>()
        .context("Invalid resource path")?;

      if resource_path
        .extension()
        .context("Resource path missing '.toml' extension")?
        != "toml"
      {
        return Err(
          anyhow!("Resource path missing '.toml' extension").into(),
        );
      }
      Some(resource_path)
    } else {
      None
    };

    let res = ExportAllResourcesToToml {
      include_resources: sync.config.include_resources,
      tags: sync.config.match_tags.clone(),
      include_variables: sync.config.include_variables,
      include_user_groups: sync.config.include_user_groups,
    }
    .resolve(&ReadArgs {
      user: sync_user().to_owned(),
    })
    .await?;

    let mut update = make_update(&sync, Operation::CommitSync, user);
    update.id = add_update(update.clone()).await?;

    update.logs.push(Log::simple("Resources", res.toml.clone()));

    if sync.config.files_on_host {
      let Some(resource_path) = resource_path else {
        // Resource path checked above for files_on_host mode.
        unreachable!()
      };
      let file_path = core_config()
        .sync_directory
        .join(to_path_compatible_name(&sync.name))
        .join(&resource_path);
      if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent)
          .await
          .with_context(|| format!("Failed to initialize resource file parent directory {parent:?}"))?;
      };
      if let Err(e) = tokio::fs::write(&file_path, &res.toml)
        .await
        .with_context(|| {
          format!("Failed to write resource file to {file_path:?}",)
        })
      {
        update.push_error_log(
          "Write resource file",
          format_serror(&e.into()),
        );
        update.finalize();
        add_update(update.clone()).await?;
        return Ok(update);
      } else {
        update.push_simple_log(
          "Write contents",
          format!("File contents written to {file_path:?}"),
        );
      }
    } else if let Some(repo) = &repo {
      let Some(resource_path) = resource_path else {
        // Resource path checked above for repo mode.
        unreachable!()
      };
      let args: RepoExecutionArgs = repo.into();
      if let Err(e) =
        commit_git_sync(args, &resource_path, &res.toml, &mut update)
          .await
      {
        update.push_error_log(
          "Write resource file",
          format_serror(&e.into()),
        );
        update.finalize();
        add_update(update.clone()).await?;
        return Ok(update);
      }
    } else if !sync.config.repo.is_empty() {
      let Some(resource_path) = resource_path else {
        // Resource path checked above for repo mode.
        unreachable!()
      };
      let args: RepoExecutionArgs = (&sync).into();
      if let Err(e) =
        commit_git_sync(args, &resource_path, &res.toml, &mut update)
          .await
      {
        update.push_error_log(
          "Write resource file",
          format_serror(&e.into()),
        );
        update.finalize();
        add_update(update.clone()).await?;
        return Ok(update);
      }

      // ===========
      // UI DEFINED
    } else if let Err(e) = db_client()
      .resource_syncs
      .update_one(
        doc! { "name": &sync.name },
        doc! { "$set": { "config.file_contents": res.toml } },
      )
      .await
      .context("failed to update file_contents on db")
    {
      update.push_error_log(
        "Write resource to database",
        format_serror(&e.into()),
      );
      update.finalize();
      add_update(update.clone()).await?;
      return Ok(update);
    }

    if let Err(e) = (RefreshResourceSyncPending { sync: sync.name })
      .resolve(args)
      .await
    {
      update.push_error_log(
        "Refresh sync pending",
        format_serror(&e.error.into()),
      );
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

async fn commit_git_sync(
  mut args: RepoExecutionArgs,
  resource_path: &Path,
  toml: &str,
  update: &mut Update,
) -> anyhow::Result<()> {
  let root = args.unique_path(&core_config().repo_directory)?;
  args.destination = Some(root.display().to_string());

  let access_token = if let Some(account) = &args.account {
    git_token(&args.provider, account, |https| args.https = https)
      .await
      .with_context(
        || format!("Failed to get git token in call to db. Stopping run. | {} | {account}", args.provider),
      )?
  } else {
    None
  };

  let (pull_res, _) = git::pull_or_clone(
    args.clone(),
    &core_config().repo_directory,
    access_token,
  )
  .await?;
  update.logs.extend(pull_res.logs);
  if !all_logs_success(&update.logs) {
    return Ok(());
  }

  let res = git::write_commit_file(
    "Commit Sync",
    &root,
    resource_path,
    toml,
    &args.branch,
  )
  .await?;
  update.logs.extend(res.logs);

  Ok(())
}

impl Resolve<WriteArgs> for RefreshResourceSyncPending {
  #[instrument(
    name = "RefreshResourceSyncPending",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ResourceSync> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // sync should be able to do this.
    let mut sync =
      get_check_permissions::<entities::sync::ResourceSync>(
        &self.sync,
        user,
        PermissionLevel::Execute.into(),
      )
      .await?;

    let repo = if !sync.config.files_on_host
      && !sync.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&sync.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    if !sync.config.managed
      && !sync.config.files_on_host
      && sync.config.file_contents.is_empty()
      && sync.config.repo.is_empty()
      && sync.config.linked_repo.is_empty()
    {
      // Sync not configured, nothing to refresh
      return Ok(sync);
    }

    let res = async {
      let RemoteResources {
        resources,
        files,
        file_errors,
        hash,
        message,
        ..
      } = crate::sync::remote::get_remote_resources(
        &sync,
        repo.as_ref(),
      )
      .await
      .context("failed to get remote resources")?;

      sync.info.remote_contents = files;
      sync.info.remote_errors = file_errors;
      sync.info.pending_hash = hash;
      sync.info.pending_message = message;

      if !sync.info.remote_errors.is_empty() {
        return Err(anyhow!(
          "Remote resources have errors. Cannot compute diffs."
        ));
      }

      let resources = resources?;
      let delete = sync.config.managed || sync.config.delete;
      let all_resources = AllResourcesById::load().await?;

      let (resource_updates, deploy_updates) =
        if sync.config.include_resources {
          let id_to_tags = get_id_to_tags(None).await?;

          let deployments_by_name = all_resources
            .deployments
            .values()
            .map(|deployment| {
              (deployment.name.clone(), deployment.clone())
            })
            .collect::<HashMap<_, _>>();
          let stacks_by_name = all_resources
            .stacks
            .values()
            .map(|stack| (stack.name.clone(), stack.clone()))
            .collect::<HashMap<_, _>>();

          let deploy_updates =
            crate::sync::deploy::get_updates_for_view(
              SyncDeployParams {
                deployments: &resources.deployments,
                deployment_map: &deployments_by_name,
                stacks: &resources.stacks,
                stack_map: &stacks_by_name,
              },
            )
            .await;

          let mut diffs = Vec::new();

          push_updates_for_view::<Server>(
            resources.servers,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Stack>(
            resources.stacks,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Deployment>(
            resources.deployments,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Build>(
            resources.builds,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Repo>(
            resources.repos,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Procedure>(
            resources.procedures,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Action>(
            resources.actions,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Builder>(
            resources.builders,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<Alerter>(
            resources.alerters,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;
          push_updates_for_view::<ResourceSync>(
            resources.resource_syncs,
            delete,
            None,
            None,
            &id_to_tags,
            &sync.config.match_tags,
            &mut diffs,
          )
          .await?;

          (diffs, deploy_updates)
        } else {
          (Vec::new(), SyncDeployUpdate::default())
        };

      let variable_updates = if sync.config.include_variables {
        crate::sync::variables::get_updates_for_view(
          &resources.variables,
          delete,
        )
        .await?
      } else {
        Default::default()
      };

      let user_group_updates = if sync.config.include_user_groups {
        crate::sync::user_groups::get_updates_for_view(
          resources.user_groups,
          delete,
        )
        .await?
      } else {
        Default::default()
      };

      anyhow::Ok((
        resource_updates,
        deploy_updates,
        variable_updates,
        user_group_updates,
      ))
    }
    .await;

    let (
      resource_updates,
      deploy_updates,
      variable_updates,
      user_group_updates,
      pending_error,
    ) = match res {
      Ok(res) => (res.0, res.1, res.2, res.3, None),
      Err(e) => (
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Some(format_serror(&e.into())),
      ),
    };

    let has_updates = !resource_updates.is_empty()
      || !deploy_updates.to_deploy == 0
      || !variable_updates.is_empty()
      || !user_group_updates.is_empty();

    let info = ResourceSyncInfo {
      last_sync_ts: sync.info.last_sync_ts,
      last_sync_hash: sync.info.last_sync_hash,
      last_sync_message: sync.info.last_sync_message,
      remote_contents: sync.info.remote_contents,
      remote_errors: sync.info.remote_errors,
      pending_hash: sync.info.pending_hash,
      pending_message: sync.info.pending_message,
      pending_deploy: deploy_updates,
      resource_updates,
      variable_updates,
      user_group_updates,
      pending_error,
    };

    let info = to_document(&info)
      .context("failed to serialize pending to document")?;

    update_one_by_id(
      &db_client().resource_syncs,
      &sync.id,
      doc! { "$set": { "info": info } },
      None,
    )
    .await?;

    // check to update alert
    let id = sync.id.clone();
    let name = sync.name.clone();
    tokio::task::spawn(async move {
      let db = db_client();
      let Some(existing) = db_client()
        .alerts
        .find_one(doc! {
          "resolved": false,
          "target.type": "ResourceSync",
          "target.id": &id,
        })
        .await
        .context("failed to query db for alert")
        .inspect_err(|e| warn!("{e:#}"))
        .ok()
      else {
        return;
      };
      match (existing, has_updates) {
        // OPEN A NEW ALERT
        (None, true) => {
          let alert = Alert {
            id: Default::default(),
            ts: komodo_timestamp(),
            resolved: false,
            level: SeverityLevel::Ok,
            target: ResourceTarget::ResourceSync(id.clone()),
            data: AlertData::ResourceSyncPendingUpdates { id, name },
            resolved_ts: None,
          };
          db.alerts
            .insert_one(&alert)
            .await
            .context("failed to open existing pending resource sync updates alert")
            .inspect_err(|e| warn!("{e:#}"))
            .ok();
          if sync.config.pending_alert {
            send_alerts(&[alert]).await;
          }
        }
        // CLOSE ALERT
        (Some(existing), false) => {
          update_one_by_id(
            &db.alerts,
            &existing.id,
            doc! {
              "$set": {
                "resolved": true,
                "resolved_ts": komodo_timestamp()
              }
            },
            None,
          )
          .await
          .context("failed to close existing pending resource sync updates alert")
          .inspect_err(|e| warn!("{e:#}"))
          .ok();
        }
        // NOTHING TO DO
        _ => {}
      }
    });

    Ok(crate::resource::get::<ResourceSync>(&sync.id).await?)
  }
}

impl Resolve<WriteArgs> for CreateSyncWebhook {
  #[instrument(name = "CreateSyncWebhook", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<CreateSyncWebhookResponse> {
    let WriteArgs { user } = args;
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let sync = get_check_permissions::<ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if sync.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = sync.config.repo.split('/');
    let owner = split.next().context("Sync repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo =
      split.next().context("Repo repo has no repo after the /")?;

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

    let webhook_secret = if sync.config.webhook_secret.is_empty() {
      webhook_secret
    } else {
      &sync.config.webhook_secret
    };

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = match self.action {
      SyncWebhookAction::Refresh => {
        format!("{host}/listener/github/sync/{}/refresh", sync.id)
      }
      SyncWebhookAction::Sync => {
        format!("{host}/listener/github/sync/{}/sync", sync.id)
      }
    };

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

    if !sync.config.webhook_enabled {
      UpdateResourceSync {
        id: sync.id,
        config: PartialResourceSyncConfig {
          webhook_enabled: Some(true),
          ..Default::default()
        },
      }
      .resolve(args)
      .await
      .map_err(|e| e.error)
      .context("failed to update sync to enable webhook")?;
    }

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for DeleteSyncWebhook {
  #[instrument(name = "DeleteSyncWebhook", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteSyncWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let sync = get_check_permissions::<ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if sync.config.git_provider != "github.com" {
      return Err(
        anyhow!("Can only manage github.com repo webhooks").into(),
      );
    }

    if sync.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = sync.config.repo.split('/');
    let owner = split.next().context("Sync repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo =
      split.next().context("Sync repo has no repo after the /")?;

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
      ..
    } = core_config();

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = match self.action {
      SyncWebhookAction::Refresh => {
        format!("{host}/listener/github/sync/{}/refresh", sync.id)
      }
      SyncWebhookAction::Sync => {
        format!("{host}/listener/github/sync/{}/sync", sync.id)
      }
    };

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
