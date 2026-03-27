use std::{
  collections::{HashMap, HashSet},
  future::IntoFuture,
  time::Duration,
};

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::update_one_by_id,
  find::find_collect,
  mongodb::{
    bson::{doc, to_bson, to_document},
    options::FindOneOptions,
  },
};
use formatting::format_serror;
use futures_util::future::join_all;
use interpolate::Interpolator;
use komodo_client::{
  api::{
    execute::{
      BatchExecutionResponse, BatchRunBuild, CancelBuild, Deploy,
      RunBuild,
    },
    write::RefreshBuildCache,
  },
  entities::{
    alert::{Alert, AlertData, SeverityLevel},
    all_logs_success,
    build::{Build, BuildConfig},
    builder::{Builder, BuilderConfig},
    deployment::DeploymentState,
    komodo_timestamp, optional_string,
    permission::PermissionLevel,
    repo::Repo,
    update::{Log, Update},
    user::auto_redeploy_user,
  },
};
use mogh_resolver::Resolve;
use periphery_client::api;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
  alert::send_alerts,
  api::write::WriteArgs,
  helpers::{
    build_git_token,
    builder::{cleanup_builder_instance, connect_builder_periphery},
    channel::build_cancel_channel,
    query::{
      VariablesAndSecrets, get_deployment_state,
      get_variables_and_secrets,
    },
    registry_token,
    update::{init_execution_update, update_update},
  },
  permission::get_check_permissions,
  resource::{self, refresh_build_state_cache},
  state::{action_states, db_client},
};

use super::{ExecuteArgs, ExecuteRequest};

impl super::BatchExecute for BatchRunBuild {
  type Resource = Build;
  fn single_request(build: String) -> ExecuteRequest {
    ExecuteRequest::RunBuild(RunBuild { build })
  }
}

impl Resolve<ExecuteArgs> for BatchRunBuild {
  #[instrument(
    "BatchRunBuild",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      pattern = self.pattern,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, task_id, .. }: &ExecuteArgs,
  ) -> mogh_error::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchRunBuild>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for RunBuild {
  #[instrument(
    "RunBuild",
    skip_all,
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      build = self.build,
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
    let mut build = get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut repo = if !build.config.files_on_host
      && !build.config.linked_repo.is_empty()
    {
      crate::resource::get::<Repo>(&build.config.linked_repo)
        .await?
        .into()
    } else {
      None
    };

    let VariablesAndSecrets {
      mut variables,
      secrets,
    } = get_variables_and_secrets().await?;

    // Add the $VERSION to variables. Use with [[$VERSION]]
    variables.insert(
      String::from("$VERSION"),
      build.config.version.to_string(),
    );

    if build.config.builder_id.is_empty() {
      return Err(anyhow!("Must attach builder to RunBuild").into());
    }

    // get the action state for the build (or insert default).
    let action_state =
      action_states().build.get_or_insert_default(&build.id).await;

    // This will set action state back to default when dropped.
    // Will also check to ensure build not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.building = true)?;

    if build.config.auto_increment_version {
      build.config.version.increment();
    }

    let mut update = update.clone();

    update.version = build.config.version;
    update_update(update.clone()).await?;

    let git_token =
      build_git_token(&mut build, repo.as_mut()).await?;

    let registry_tokens =
      validate_account_extract_registry_tokens(&build).await?;

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    let mut cancel_recv =
      build_cancel_channel().receiver.resubscribe();
    let build_id = build.id.clone();

    let builder =
      resource::get::<Builder>(&build.config.builder_id).await?;

    let is_server_builder =
      matches!(&builder.config, BuilderConfig::Server(_));

    tokio::spawn(async move {
      let poll = async {
        loop {
          let (incoming_build_id, mut update) = tokio::select! {
            _ = cancel_clone.cancelled() => return Ok(()),
            id = cancel_recv.recv() => id?
          };
          if incoming_build_id == build_id {
            if is_server_builder {
              update.push_error_log("Cancel acknowledged", "Build cancellation is not possible on server builders at this time. Use an AWS builder to enable this feature.");
            } else {
              update.push_simple_log("Cancel acknowledged", "The build cancellation has been queued, it may still take some time.");
            }
            update.finalize();
            let id = update.id.clone();
            if let Err(e) = update_update(update).await {
              warn!("Failed to modify Update {id} on db | {e:#}");
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
    let (periphery, cleanup_data) = match connect_builder_periphery(
      build.name.clone(),
      Some(build.config.version),
      builder,
      &mut update,
    )
    .await
    {
      Ok(builder) => builder,
      Err(e) => {
        warn!(
          "Failed to get Builder for Build {} | {e:#}",
          build.name
        );
        update.logs.push(Log::error(
          "Get Builder",
          format_serror(&e.context("Failed to get Builder").into()),
        ));
        return handle_early_return(
          update, build.id, build.name, false,
        )
        .await;
      }
    };

    // INTERPOLATE VARIABLES
    let secret_replacers = if !build.config.skip_secret_interp {
      let mut interpolator =
        Interpolator::new(Some(&variables), &secrets);

      interpolator.interpolate_build(&mut build)?;

      if let Some(repo) = repo.as_mut() {
        interpolator.interpolate_repo(repo)?;
      }

      interpolator.push_logs(&mut update.logs);

      interpolator.secret_replacers
    } else {
      Default::default()
    };

    let commit_message = if !build.config.files_on_host
      && (!build.config.repo.is_empty()
        || !build.config.linked_repo.is_empty())
    {
      // PULL OR CLONE REPO
      let res = tokio::select! {
        res = periphery
          .request(api::git::PullOrCloneRepo {
            args: repo.as_ref().map(Into::into).unwrap_or((&build).into()),
            git_token,
            environment: Default::default(),
            env_file_path: Default::default(),
            on_clone: None,
            on_pull: None,
            skip_secret_interp: Default::default(),
            replacers: Default::default(),
          }) => res,
        _ = cancel.cancelled() => {
          debug!("Build cancelled during clone, cleaning up builder");
          update.push_error_log("Build cancelled", String::from("user cancelled build during repo clone"));
          cleanup_builder_instance(periphery, cleanup_data, &mut update)
            .await;
          info!("Builder cleaned up");
          return handle_early_return(update, build.id, build.name, true).await
        },
      };

      let commit_message = match res {
        Ok(res) => {
          debug!("Finished repo clone");
          update.logs.extend(res.res.logs);
          update.commit_hash =
            res.res.commit_hash.unwrap_or_default().to_string();
          res.res.commit_message.unwrap_or_default()
        }
        Err(e) => {
          warn!("Failed build at clone repo | {e:#}");
          update.push_error_log(
            "Clone Repo",
            format_serror(&e.context("Failed to clone repo").into()),
          );
          Default::default()
        }
      };

      update_update(update.clone()).await?;

      Some(commit_message)
    } else {
      None
    };

    if all_logs_success(&update.logs) {
      // RUN BUILD
      let res = tokio::select! {
        res = periphery
          .request(api::build::Build {
            build: build.clone(),
            repo,
            registry_tokens,
            replacers: secret_replacers.into_iter().collect(),
            // To push a commit hash tagged image
            commit_hash: optional_string(&update.commit_hash),
            // Unused for now
            additional_tags: Default::default(),
          }) => res.context("Failed at call to Periphery to build"),
        _ = cancel.cancelled() => {
          info!("Build cancelled during build, cleaning up builder");
          update.push_error_log("Build cancelled", String::from("User cancelled build during docker build"));
          cleanup_builder_instance(periphery, cleanup_data, &mut update)
            .await;
          return handle_early_return(update, build.id, build.name, true).await
        },
      };

      match res {
        Ok(logs) => {
          debug!("finished build");
          update.logs.extend(logs);
        }
        Err(e) => {
          warn!("Error in build | {e:#}");
          update.push_error_log(
            "Build Error",
            format_serror(&e.context("Failed to build").into()),
          )
        }
      };
    }

    update.finalize();

    let db = db_client();

    if update.success {
      let _ = db
        .builds
        .update_one(
          doc! { "name": &build.name },
          doc! { "$set": {
            "config.version": to_bson(&build.config.version)
              .context("failed at converting version to bson")?,
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
    cleanup_builder_instance(periphery, cleanup_data, &mut update)
      .await;

    // Need to manually update the update before cache refresh,
    // and before broadcast with add_update.
    // The Err case of to_document should be unreachable,
    // but will fail to update cache in that case.
    if let Ok(update_doc) = to_document(&update) {
      let _ = update_one_by_id(
        &db.updates,
        &update.id,
        database::mungos::update::Update::Set(update_doc),
        None,
      )
      .await;
      refresh_build_state_cache().await;
    }

    update_update(update.clone()).await?;

    let Build { id, name, .. } = build;

    if update.success {
      // don't hold response up for user
      tokio::spawn(async move {
        handle_post_build_redeploy(&id).await;
      });
    } else {
      let name = name.clone();
      let target = update.target.clone();
      let version = update.version;
      tokio::spawn(async move {
        let alert = Alert {
          id: Default::default(),
          target,
          ts: komodo_timestamp(),
          resolved_ts: Some(komodo_timestamp()),
          resolved: true,
          level: SeverityLevel::Warning,
          data: AlertData::BuildFailed { id, name, version },
        };
        send_alerts(&[alert]).await
      });
    }

    if let Err(e) = (RefreshBuildCache { build: name })
      .resolve(&WriteArgs { user: user.clone() })
      .await
    {
      update.push_error_log(
        "Refresh build cache",
        format_serror(&e.error.into()),
      );
    }

    Ok(update.clone())
  }
}

#[instrument("HandleEarlyReturn", skip(update))]
async fn handle_early_return(
  mut update: Update,
  build_id: String,
  build_name: String,
  is_cancel: bool,
) -> mogh_error::Result<Update> {
  update.finalize();
  // Need to manually update the update before cache refresh,
  // and before broadcast with add_update.
  // The Err case of to_document should be unreachable,
  // but will fail to update cache in that case.
  if let Ok(update_doc) = to_document(&update) {
    let _ = update_one_by_id(
      &db_client().updates,
      &update.id,
      database::mungos::update::Update::Set(update_doc),
      None,
    )
    .await;
    refresh_build_state_cache().await;
  }
  update_update(update.clone()).await?;
  if !update.success && !is_cancel {
    let target = update.target.clone();
    let version = update.version;
    tokio::spawn(async move {
      let alert = Alert {
        id: Default::default(),
        target,
        ts: komodo_timestamp(),
        resolved_ts: Some(komodo_timestamp()),
        resolved: true,
        level: SeverityLevel::Warning,
        data: AlertData::BuildFailed {
          id: build_id,
          name: build_name,
          version,
        },
      };
      send_alerts(&[alert]).await
    });
  }
  Ok(update.clone())
}

pub async fn validate_cancel_build(
  request: &ExecuteRequest,
) -> anyhow::Result<()> {
  if let ExecuteRequest::CancelBuild(req) = request {
    let build = resource::get::<Build>(&req.build).await?;

    let db = db_client();

    let (latest_build, latest_cancel) = tokio::try_join!(
      db.updates
        .find_one(doc! {
          "operation": "RunBuild",
          "target.id": &build.id,
        },)
        .with_options(
          FindOneOptions::builder()
            .sort(doc! { "start_ts": -1 })
            .build()
        )
        .into_future(),
      db.updates
        .find_one(doc! {
          "operation": "CancelBuild",
          "target.id": &build.id,
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
          return Err(anyhow!("Build has already been cancelled"));
        }
      }
      (None, _) => return Err(anyhow!("No build in progress")),
      _ => {}
    };
  }
  Ok(())
}

impl Resolve<ExecuteArgs> for CancelBuild {
  #[instrument(
    "CancelBuild",
    skip(user, update),
    fields(
      task_id = task_id.to_string(),
      operator = user.id,
      update_id = update.id,
      build = self.build,
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
    let build = get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // make sure the build is building
    if !action_states()
      .build
      .get(&build.id)
      .await
      .and_then(|s| s.get().ok().map(|s| s.building))
      .unwrap_or_default()
    {
      return Err(anyhow!("Build is not building.").into());
    }

    let mut update = update.clone();

    update.push_simple_log(
      "cancel triggered",
      "the build cancel has been triggered",
    );
    update_update(update.clone()).await?;

    build_cancel_channel()
      .sender
      .lock()
      .await
      .send((build.id, update.clone()))?;

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
          "Failed to set CancelBuild Update status Complete after timeout | {e:#}"
        )
      }
    });

    Ok(update)
  }
}

#[instrument("PostBuildRedeploy")]
async fn handle_post_build_redeploy(build_id: &str) {
  let Ok(redeploy_deployments) = find_collect(
    &db_client().deployments,
    doc! {
      "config.image.params.build_id": build_id,
      "config.redeploy_on_build": true
    },
    None,
  )
  .await
  else {
    return;
  };

  let futures =
    redeploy_deployments
      .into_iter()
      .map(|deployment| async move {
        let state = get_deployment_state(&deployment.id)
          .await
          .unwrap_or_default();
        if state == DeploymentState::Running {
          let req = super::ExecuteRequest::Deploy(Deploy {
            deployment: deployment.id.clone(),
            stop_signal: None,
            stop_time: None,
          });
          let user = auto_redeploy_user().to_owned();
          let res = async {
            let update = init_execution_update(&req, &user).await?;
            Deploy {
              deployment: deployment.id.clone(),
              stop_signal: None,
              stop_time: None,
            }
            .resolve(&ExecuteArgs {
              user,
              update,
              task_id: Uuid::new_v4(),
            })
            .await
          }
          .await;
          Some((deployment.id.clone(), res))
        } else {
          None
        }
      });

  for res in join_all(futures).await {
    let Some((id, res)) = res else {
      continue;
    };
    if let Err(e) = res {
      warn!(
        "failed post build redeploy for deployment {id}: {:#}",
        e.error
      );
    }
  }
}

/// This will make sure that a build with non-none image registry has an account attached,
/// and will check the core config for a token matching requirements.
/// Otherwise it is left to periphery.
#[instrument("ValidateRegistryTokens")]
async fn validate_account_extract_registry_tokens(
  Build {
    config: BuildConfig { image_registry, .. },
    ..
  }: &Build,
  // Maps (domain, account) -> token
) -> mogh_error::Result<Vec<(String, String, String)>> {
  let mut res = HashMap::with_capacity(image_registry.capacity());

  for (domain, account) in image_registry
    .iter()
    .map(|r| (r.domain.as_str(), r.account.as_str()))
    // This ensures uniqueness / prevents redundant logins
    .collect::<HashSet<_>>()
  {
    if domain.is_empty() {
      continue;
    }
    if account.is_empty() {
      return Err(
        anyhow!(
          "Must attach account to use registry provider {domain}"
        )
        .into(),
      );
    }
    let Some(registry_token) = registry_token(domain, account).await.with_context(
      || format!("Failed to get registry token in call to db. Stopping run. | {domain} | {account}"),
    )? else {
      continue;
    };

    res.insert(
      (domain.to_string(), account.to_string()),
      registry_token,
    );
  }

  Ok(
    res
      .into_iter()
      .map(|((domain, account), token)| (domain, account, token))
      .collect(),
  )
}
