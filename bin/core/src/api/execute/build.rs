use std::{collections::HashSet, future::IntoFuture, time::Duration};

use anyhow::{anyhow, Context};
use formatting::format_serror;
use futures::future::join_all;
use komodo_client::{
  api::execute::{
    BatchExecutionResponse, BatchRunBuild, CancelBuild, Deploy,
    RunBuild,
  },
  entities::{
    alert::{Alert, AlertData, SeverityLevel},
    all_logs_success,
    build::{Build, BuildConfig, ImageRegistryConfig},
    builder::{Builder, BuilderConfig},
    deployment::DeploymentState,
    komodo_timestamp,
    permission::PermissionLevel,
    update::{Log, Update},
    user::auto_redeploy_user,
  },
};
use mungos::{
  by_id::update_one_by_id,
  find::find_collect,
  mongodb::{
    bson::{doc, to_bson, to_document},
    options::FindOneOptions,
  },
};
use periphery_client::api;
use resolver_api::Resolve;
use tokio_util::sync::CancellationToken;

use crate::{
  alert::send_alerts,
  helpers::{
    builder::{cleanup_builder_instance, get_builder_periphery},
    channel::build_cancel_channel,
    git_token,
    interpolate::{
      add_interp_update_log,
      interpolate_variables_secrets_into_extra_args,
      interpolate_variables_secrets_into_string,
      interpolate_variables_secrets_into_system_command,
    },
    query::{get_deployment_state, get_variables_and_secrets},
    registry_token,
    update::{init_execution_update, update_update},
  },
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
  #[instrument(name = "BatchRunBuild", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchRunBuild>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for RunBuild {
  #[instrument(name = "RunBuild", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let mut build = resource::get_check_permissions::<Build>(
      &self.build,
      &user,
      PermissionLevel::Execute,
    )
    .await?;
    let mut vars_and_secrets = get_variables_and_secrets().await?;

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

    // Add the $VERSION to variables. Use with [[$VERSION]]
    if !vars_and_secrets.variables.contains_key("$VERSION") {
      vars_and_secrets.variables.insert(
        String::from("$VERSION"),
        build.config.version.to_string(),
      );
    }

    let git_token = git_token(
      &build.config.git_provider,
      &build.config.git_account,
      |https| build.config.git_https = https,
    )
    .await
    .with_context(
      || format!("Failed to get git token in call to db. This is a database error, not a token exisitence error. Stopping run. | {} | {}", build.config.git_provider, build.config.git_account),
    )?;

    let registry_token =
      validate_account_extract_registry_token(&build).await?;

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
          "failed to get builder for build {} | {e:#}",
          build.name
        );
        update.logs.push(Log::error(
          "get builder",
          format_serror(&e.context("failed to get builder").into()),
        ));
        return handle_early_return(
          update, build.id, build.name, false,
        )
        .await;
      }
    };

    // CLONE REPO
    let secret_replacers = if !build.config.skip_secret_interp {
      // Interpolate variables / secrets into pre build command
      let mut global_replacers = HashSet::new();
      let mut secret_replacers = HashSet::new();

      interpolate_variables_secrets_into_system_command(
        &vars_and_secrets,
        &mut build.config.pre_build,
        &mut global_replacers,
        &mut secret_replacers,
      )?;

      add_interp_update_log(
        &mut update,
        &global_replacers,
        &secret_replacers,
      );

      secret_replacers
    } else {
      Default::default()
    };

    let res = tokio::select! {
      res = periphery
        .request(api::git::CloneRepo {
          args: (&build).into(),
          git_token,
          environment: Default::default(),
          env_file_path: Default::default(),
          skip_secret_interp: Default::default(),
          replacers: secret_replacers.into_iter().collect(),
        }) => res,
      _ = cancel.cancelled() => {
        debug!("build cancelled during clone, cleaning up builder");
        update.push_error_log("build cancelled", String::from("user cancelled build during repo clone"));
        cleanup_builder_instance(periphery, cleanup_data, &mut update)
          .await;
        info!("builder cleaned up");
        return handle_early_return(update, build.id, build.name, true).await
      },
    };

    let commit_message = match res {
      Ok(res) => {
        debug!("finished repo clone");
        update.logs.extend(res.logs);
        update.commit_hash =
          res.commit_hash.unwrap_or_default().to_string();
        res.commit_message.unwrap_or_default()
      }
      Err(e) => {
        warn!("failed build at clone repo | {e:#}");
        update.push_error_log(
          "clone repo",
          format_serror(&e.context("failed to clone repo").into()),
        );
        Default::default()
      }
    };

    update_update(update.clone()).await?;

    if all_logs_success(&update.logs) {
      let secret_replacers = if !build.config.skip_secret_interp {
        // Interpolate variables / secrets into build args
        let mut global_replacers = HashSet::new();
        let mut secret_replacers = HashSet::new();

        interpolate_variables_secrets_into_string(
          &vars_and_secrets,
          &mut build.config.build_args,
          &mut global_replacers,
          &mut secret_replacers,
        )?;

        interpolate_variables_secrets_into_string(
          &vars_and_secrets,
          &mut build.config.secret_args,
          &mut global_replacers,
          &mut secret_replacers,
        )?;

        interpolate_variables_secrets_into_extra_args(
          &vars_and_secrets,
          &mut build.config.extra_args,
          &mut global_replacers,
          &mut secret_replacers,
        )?;

        add_interp_update_log(
          &mut update,
          &global_replacers,
          &secret_replacers,
        );

        secret_replacers
      } else {
        Default::default()
      };

      let res = tokio::select! {
        res = periphery
          .request(api::build::Build {
            build: build.clone(),
            registry_token,
            replacers: secret_replacers.into_iter().collect(),
            // Push a commit hash tagged image
            additional_tags: if update.commit_hash.is_empty() {
              Default::default()
            } else {
              vec![update.commit_hash.clone()]
            },
          }) => res.context("failed at call to periphery to build"),
        _ = cancel.cancelled() => {
          info!("build cancelled during build, cleaning up builder");
          update.push_error_log("build cancelled", String::from("user cancelled build during docker build"));
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
          warn!("error in build | {e:#}");
          update.push_error_log(
            "build",
            format_serror(&e.context("failed to build").into()),
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
        mungos::update::Update::Set(update_doc),
        None,
      )
      .await;
      refresh_build_state_cache().await;
    }

    update_update(update.clone()).await?;

    if update.success {
      // don't hold response up for user
      tokio::spawn(async move {
        handle_post_build_redeploy(&build.id).await;
      });
    } else {
      warn!("build unsuccessful, alerting...");
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
            id: build.id,
            name: build.name,
            version,
          },
        };
        send_alerts(&[alert]).await
      });
    }

    Ok(update.clone())
  }
}

#[instrument(skip(update))]
async fn handle_early_return(
  mut update: Update,
  build_id: String,
  build_name: String,
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
    refresh_build_state_cache().await;
  }
  update_update(update.clone()).await?;
  if !update.success && !is_cancel {
    warn!("build unsuccessful, alerting...");
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
  #[instrument(name = "CancelBuild", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let build = resource::get_check_permissions::<Build>(
      &self.build,
      &user,
      PermissionLevel::Execute,
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
        warn!("failed to set CancelBuild Update status Complete after timeout | {e:#}")
      }
    });

    Ok(update)
  }
}

#[instrument]
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
        let state =
          get_deployment_state(&deployment).await.unwrap_or_default();
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
            .resolve(&ExecuteArgs { user, update })
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
async fn validate_account_extract_registry_token(
  Build {
    config:
      BuildConfig {
        image_registry:
          ImageRegistryConfig {
            domain, account, ..
          },
        ..
      },
    ..
  }: &Build,
) -> serror::Result<Option<String>> {
  if domain.is_empty() {
    return Ok(None);
  }
  if account.is_empty() {
    return Err(
      anyhow!(
        "Must attach account to use registry provider {domain}"
      )
      .into(),
    );
  }

  let registry_token = registry_token(domain, account).await.with_context(
    || format!("Failed to get registry token in call to db. Stopping run. | {domain} | {account}"),
  )?;

  Ok(registry_token)
}
