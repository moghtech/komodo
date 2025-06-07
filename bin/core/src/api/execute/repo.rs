use std::{collections::HashSet, future::IntoFuture, time::Duration};

use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::{
  api::{execute::*, write::RefreshRepoCache},
  entities::{
    alert::{Alert, AlertData, SeverityLevel},
    builder::{Builder, BuilderConfig},
    komodo_timestamp,
    permission::PermissionLevel,
    repo::Repo,
    server::Server,
    update::{Log, Update},
  },
};
use mungos::{
  by_id::update_one_by_id,
  mongodb::{
    bson::{doc, to_document},
    options::FindOneOptions,
  },
};
use periphery_client::api;
use resolver_api::Resolve;
use tokio_util::sync::CancellationToken;

use crate::{
  alert::send_alerts,
  api::write::WriteArgs,
  helpers::{
    builder::{cleanup_builder_instance, get_builder_periphery},
    channel::repo_cancel_channel,
    git_token,
    interpolate::{
      add_interp_update_log,
      interpolate_variables_secrets_into_string,
      interpolate_variables_secrets_into_system_command,
    },
    periphery_client,
    query::get_variables_and_secrets,
    update::update_update,
  },
  permission::get_check_permissions,
  resource::{self, refresh_repo_state_cache},
  state::{action_states, db_client},
};

use super::{ExecuteArgs, ExecuteRequest};

impl super::BatchExecute for BatchCloneRepo {
  type Resource = Repo;
  fn single_request(repo: String) -> ExecuteRequest {
    ExecuteRequest::CloneRepo(CloneRepo { repo })
  }
}

impl Resolve<ExecuteArgs> for BatchCloneRepo {
  #[instrument(name = "BatchCloneRepo", skip( user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchCloneRepo>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for CloneRepo {
  #[instrument(name = "CloneRepo", skip( user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let mut repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the repo (or insert default).
    let action_state =
      action_states().repo.get_or_insert_default(&repo.id).await;

    // This will set action state back to default when dropped.
    // Will also check to ensure repo not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.cloning = true)?;

    let mut update = update.clone();
    update_update(update.clone()).await?;

    if repo.config.server_id.is_empty() {
      return Err(anyhow!("repo has no server attached").into());
    }

    let git_token = git_token(
      &repo.config.git_provider,
      &repo.config.git_account,
      |https| repo.config.git_https = https,
    )
    .await
    .with_context(
      || format!("Failed to get git token in call to db. This is a database error, not a token exisitence error. Stopping run. | {} | {}", repo.config.git_provider, repo.config.git_account),
    )?;

    let server =
      resource::get::<Server>(&repo.config.server_id).await?;

    let periphery = periphery_client(&server)?;

    // interpolate variables / secrets, returning the sanitizing replacers to send to
    // periphery so it may sanitize the final command for safe logging (avoids exposing secret values)
    let secret_replacers =
      interpolate(&mut repo, &mut update).await?;

    let logs = match periphery
      .request(api::git::CloneRepo {
        args: (&repo).into(),
        git_token,
        environment: repo.config.env_vars()?,
        env_file_path: repo.config.env_file_path,
        skip_secret_interp: repo.config.skip_secret_interp,
        replacers: secret_replacers.into_iter().collect(),
      })
      .await
    {
      Ok(res) => res.logs,
      Err(e) => {
        vec![Log::error(
          "clone repo",
          format_serror(&e.context("failed to clone repo").into()),
        )]
      }
    };

    update.logs.extend(logs);
    update.finalize();

    if update.success {
      update_last_pulled_time(&repo.name).await;
    }

    if let Err(e) = (RefreshRepoCache { repo: repo.id })
      .resolve(&WriteArgs { user: user.clone() })
      .await
      .map_err(|e| e.error)
      .context("Failed to refresh repo cache")
    {
      update.push_error_log(
        "Refresh Repo cache",
        format_serror(&e.into()),
      );
    };

    handle_server_update_return(update).await
  }
}

impl super::BatchExecute for BatchPullRepo {
  type Resource = Repo;
  fn single_request(repo: String) -> ExecuteRequest {
    ExecuteRequest::PullRepo(PullRepo { repo })
  }
}

impl Resolve<ExecuteArgs> for BatchPullRepo {
  #[instrument(name = "BatchPullRepo", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchPullRepo>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for PullRepo {
  #[instrument(name = "PullRepo", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let mut repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the repo (or insert default).
    let action_state =
      action_states().repo.get_or_insert_default(&repo.id).await;

    // This will set action state back to default when dropped.
    // Will also check to ensure repo not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.pulling = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    if repo.config.server_id.is_empty() {
      return Err(anyhow!("repo has no server attached").into());
    }

    let git_token = git_token(
      &repo.config.git_provider,
      &repo.config.git_account,
      |https| repo.config.git_https = https,
    )
    .await
    .with_context(
      || format!("Failed to get git token in call to db. This is a database error, not a token exisitence error. Stopping run. | {} | {}", repo.config.git_provider, repo.config.git_account),
    )?;

    let server =
      resource::get::<Server>(&repo.config.server_id).await?;

    let periphery = periphery_client(&server)?;

    // interpolate variables / secrets, returning the sanitizing replacers to send to
    // periphery so it may sanitize the final command for safe logging (avoids exposing secret values)
    let secret_replacers =
      interpolate(&mut repo, &mut update).await?;

    let logs = match periphery
      .request(api::git::PullRepo {
        args: (&repo).into(),
        git_token,
        environment: repo.config.env_vars()?,
        env_file_path: repo.config.env_file_path,
        skip_secret_interp: repo.config.skip_secret_interp,
        replacers: secret_replacers.into_iter().collect(),
      })
      .await
    {
      Ok(res) => {
        update.commit_hash = res.commit_hash.unwrap_or_default();
        res.logs
      }
      Err(e) => {
        vec![Log::error(
          "pull repo",
          format_serror(&e.context("failed to pull repo").into()),
        )]
      }
    };

    update.logs.extend(logs);

    update.finalize();

    if update.success {
      update_last_pulled_time(&repo.name).await;
    }

    if let Err(e) = (RefreshRepoCache { repo: repo.id })
      .resolve(&WriteArgs { user: user.clone() })
      .await
      .map_err(|e| e.error)
      .context("Failed to refresh repo cache")
    {
      update.push_error_log(
        "Refresh Repo cache",
        format_serror(&e.into()),
      );
    };

    handle_server_update_return(update).await
  }
}

#[instrument(skip_all, fields(update_id = update.id))]
async fn handle_server_update_return(
  update: Update,
) -> serror::Result<Update> {
  // Need to manually update the update before cache refresh,
  // and before broadcast with add_update.
  // The Err case of to_document should be unreachable,
  // but will fail to update cache in that case.
  if let Ok(update_doc) = to_document(&update) {
    let _ = update_one_by_id(
      &db_client().updates,
      &update.id,
      mungos::update::Update::Set(update_doc),
      None,
    )
    .await;
    refresh_repo_state_cache().await;
  }
  update_update(update.clone()).await?;
  Ok(update)
}

#[instrument]
async fn update_last_pulled_time(repo_name: &str) {
  let res = db_client()
    .repos
    .update_one(
      doc! { "name": repo_name },
      doc! { "$set": { "info.last_pulled_at": komodo_timestamp() } },
    )
    .await;
  if let Err(e) = res {
    warn!(
      "failed to update repo last_pulled_at | repo: {repo_name} | {e:#}",
    );
  }
}

impl super::BatchExecute for BatchBuildRepo {
  type Resource = Repo;
  fn single_request(repo: String) -> ExecuteRequest {
    ExecuteRequest::CloneRepo(CloneRepo { repo })
  }
}

impl Resolve<ExecuteArgs> for BatchBuildRepo {
  #[instrument(name = "BatchBuildRepo", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchBuildRepo>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for BuildRepo {
  #[instrument(name = "BuildRepo", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let mut repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    if repo.config.builder_id.is_empty() {
      return Err(anyhow!("Must attach builder to BuildRepo").into());
    }

    // get the action state for the repo (or insert default).
    let action_state =
      action_states().repo.get_or_insert_default(&repo.id).await;

    // This will set action state back to default when dropped.
    // Will also check to ensure repo not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.building = true)?;

    let mut update = update.clone();
    update_update(update.clone()).await?;

    let git_token = git_token(
      &repo.config.git_provider,
      &repo.config.git_account,
      |https| repo.config.git_https = https,
    )
    .await
    .with_context(
      || format!("Failed to get git token in call to db. This is a database error, not a token exisitence error. Stopping run. | {} | {}", repo.config.git_provider, repo.config.git_account),
    )?;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    let mut cancel_recv =
      repo_cancel_channel().receiver.resubscribe();
    let repo_id = repo.id.clone();

    let builder =
      resource::get::<Builder>(&repo.config.builder_id).await?;

    let is_server_builder =
      matches!(&builder.config, BuilderConfig::Server(_));

    tokio::spawn(async move {
      let poll = async {
        loop {
          let (incoming_repo_id, mut update) = tokio::select! {
            _ = cancel_clone.cancelled() => return Ok(()),
            id = cancel_recv.recv() => id?
          };
          if incoming_repo_id == repo_id {
            if is_server_builder {
              update.push_error_log("Cancel acknowledged", "Repo Build cancellation is not possible on server builders at this time. Use an AWS builder to enable this feature.");
            } else {
              update.push_simple_log("Cancel acknowledged", "The repo build cancellation has been queued, it may still take some time.");
            }
            update.finalize();
            let id = update.id.clone();
            if let Err(e) = update_update(update).await {
              warn!("failed to modify Update {id} on db | {e:#}");
            }
            if !is_server_builder {
              cancel_clone.cancel();
            }
            return Ok(());
          }
        }
        #[allow(unreachable_code)]
        anyhow::Ok(())
      };
      tokio::select! {
        _ = cancel_clone.cancelled() => {}
        _ = poll => {}
      }
    });

    // GET BUILDER PERIPHERY

    let (periphery, cleanup_data) = match get_builder_periphery(
      repo.name.clone(),
      None,
      builder,
      &mut update,
    )
    .await
    {
      Ok(builder) => builder,
      Err(e) => {
        warn!("failed to get builder for repo {} | {e:#}", repo.name);
        update.logs.push(Log::error(
          "get builder",
          format_serror(&e.context("failed to get builder").into()),
        ));
        return handle_builder_early_return(
          update, repo.id, repo.name, false,
        )
        .await;
      }
    };

    // CLONE REPO

    // interpolate variables / secrets, returning the sanitizing replacers to send to
    // periphery so it may sanitize the final command for safe logging (avoids exposing secret values)
    let secret_replacers =
      interpolate(&mut repo, &mut update).await?;

    let res = tokio::select! {
      res = periphery
        .request(api::git::CloneRepo {
          args: (&repo).into(),
          git_token,
          environment: repo.config.env_vars()?,
          env_file_path: repo.config.env_file_path,
          skip_secret_interp: repo.config.skip_secret_interp,
          replacers: secret_replacers.into_iter().collect()
        }) => res,
      _ = cancel.cancelled() => {
        debug!("build cancelled during clone, cleaning up builder");
        update.push_error_log("build cancelled", String::from("user cancelled build during repo clone"));
        cleanup_builder_instance(cleanup_data, &mut update)
          .await;
        info!("builder cleaned up");
        return handle_builder_early_return(update, repo.id, repo.name, true).await
      },
    };

    let commit_message = match res {
      Ok(res) => {
        debug!("finished repo clone");
        update.logs.extend(res.logs);
        update.commit_hash = res.commit_hash.unwrap_or_default();
        res.commit_message.unwrap_or_default()
      }
      Err(e) => {
        update.push_error_log(
          "clone repo",
          format_serror(&e.context("failed to clone repo").into()),
        );
        Default::default()
      }
    };

    update.finalize();

    let db = db_client();

    if update.success {
      let _ = db
        .repos
        .update_one(
          doc! { "name": &repo.name },
          doc! { "$set": {
            "info.last_built_at": komodo_timestamp(),
            "info.built_hash": &update.commit_hash,
            "info.built_message": commit_message
          }},
        )
        .await;
    }

    // stop the cancel listening task from going forever
    cancel.cancel();

    // If building on temporary cloud server (AWS),
    // this will terminate the server.
    cleanup_builder_instance(cleanup_data, &mut update).await;

    // Need to manually update the update before cache refresh,
    // and before broadcast with add_update.
    // The Err case of to_document should be unreachable,
    // but will fail to update cache in that case.
    if let Ok(update_doc) = to_document(&update) {
      let _ = update_one_by_id(
        &db.updates,
        &update.id,
        mungos::update::Update::Set(update_doc),
        None,
      )
      .await;
      refresh_repo_state_cache().await;
    }

    update_update(update.clone()).await?;

    if !update.success {
      warn!("repo build unsuccessful, alerting...");
      let target = update.target.clone();
      tokio::spawn(async move {
        let alert = Alert {
          id: Default::default(),
          target,
          ts: komodo_timestamp(),
          resolved_ts: Some(komodo_timestamp()),
          resolved: true,
          level: SeverityLevel::Warning,
          data: AlertData::RepoBuildFailed {
            id: repo.id,
            name: repo.name,
          },
        };
        send_alerts(&[alert]).await
      });
    }

    Ok(update)
  }
}

#[instrument(skip(update))]
async fn handle_builder_early_return(
  mut update: Update,
  repo_id: String,
  repo_name: String,
  is_cancel: bool,
) -> serror::Result<Update> {
  update.finalize();
  // Need to manually update the update before cache refresh,
  // and before broadcast with add_update.
  // The Err case of to_document should be unreachable,
  // but will fail to update cache in that case.
  if let Ok(update_doc) = to_document(&update) {
    let _ = update_one_by_id(
      &db_client().updates,
      &update.id,
      mungos::update::Update::Set(update_doc),
      None,
    )
    .await;
    refresh_repo_state_cache().await;
  }
  update_update(update.clone()).await?;
  if !update.success && !is_cancel {
    warn!("repo build unsuccessful, alerting...");
    let target = update.target.clone();
    tokio::spawn(async move {
      let alert = Alert {
        id: Default::default(),
        target,
        ts: komodo_timestamp(),
        resolved_ts: Some(komodo_timestamp()),
        resolved: true,
        level: SeverityLevel::Warning,
        data: AlertData::RepoBuildFailed {
          id: repo_id,
          name: repo_name,
        },
      };
      send_alerts(&[alert]).await
    });
  }
  Ok(update)
}

#[instrument(skip_all)]
pub async fn validate_cancel_repo_build(
  request: &ExecuteRequest,
) -> anyhow::Result<()> {
  if let ExecuteRequest::CancelRepoBuild(req) = request {
    let repo = resource::get::<Repo>(&req.repo).await?;

    let db = db_client();

    let (latest_build, latest_cancel) = tokio::try_join!(
      db.updates
        .find_one(doc! {
          "operation": "BuildRepo",
          "target.id": &repo.id,
        },)
        .with_options(
          FindOneOptions::builder()
            .sort(doc! { "start_ts": -1 })
            .build()
        )
        .into_future(),
      db.updates
        .find_one(doc! {
          "operation": "CancelRepoBuild",
          "target.id": &repo.id,
        },)
        .with_options(
          FindOneOptions::builder()
            .sort(doc! { "start_ts": -1 })
            .build()
        )
        .into_future()
    )?;

    match (latest_build, latest_cancel) {
      (Some(build), Some(cancel)) => {
        if cancel.start_ts > build.start_ts {
          return Err(anyhow!(
            "Repo build has already been cancelled"
          ));
        }
      }
      (None, _) => return Err(anyhow!("No repo build in progress")),
      _ => {}
    };
  }
  Ok(())
}

impl Resolve<ExecuteArgs> for CancelRepoBuild {
  #[instrument(name = "CancelRepoBuild", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // make sure the build is building
    if !action_states()
      .repo
      .get(&repo.id)
      .await
      .and_then(|s| s.get().ok().map(|s| s.building))
      .unwrap_or_default()
    {
      return Err(anyhow!("Repo is not building.").into());
    }

    let mut update = update.clone();

    update.push_simple_log(
      "cancel triggered",
      "the repo build cancel has been triggered",
    );
    update_update(update.clone()).await?;

    repo_cancel_channel()
      .sender
      .lock()
      .await
      .send((repo.id, update.clone()))?;

    // Make sure cancel is set to complete after some time in case
    // no reciever is there to do it. Prevents update stuck in InProgress.
    let update_id = update.id.clone();
    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_secs(60)).await;
      if let Err(e) = update_one_by_id(
        &db_client().updates,
        &update_id,
        doc! { "$set": { "status": "Complete" } },
        None,
      )
      .await
      {
        warn!(
          "failed to set CancelRepoBuild Update status Complete after timeout | {e:#}"
        )
      }
    });

    Ok(update)
  }
}

async fn interpolate(
  repo: &mut Repo,
  update: &mut Update,
) -> anyhow::Result<HashSet<(String, String)>> {
  if !repo.config.skip_secret_interp {
    let vars_and_secrets = get_variables_and_secrets().await?;

    let mut global_replacers = HashSet::new();
    let mut secret_replacers = HashSet::new();

    interpolate_variables_secrets_into_string(
      &vars_and_secrets,
      &mut repo.config.environment,
      &mut global_replacers,
      &mut secret_replacers,
    )?;

    interpolate_variables_secrets_into_system_command(
      &vars_and_secrets,
      &mut repo.config.on_clone,
      &mut global_replacers,
      &mut secret_replacers,
    )?;

    interpolate_variables_secrets_into_system_command(
      &vars_and_secrets,
      &mut repo.config.on_pull,
      &mut global_replacers,
      &mut secret_replacers,
    )?;

    add_interp_update_log(
      update,
      &global_replacers,
      &secret_replacers,
    );

    Ok(secret_replacers)
  } else {
    Ok(Default::default())
  }
}
