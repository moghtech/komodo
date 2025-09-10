use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::update_one_by_id,
  mongodb::bson::{doc, oid::ObjectId},
};
use formatting::{Color, colored, format_serror};
use komodo_client::{
  api::{execute::RunSync, write::RefreshResourceSyncPending},
  entities::{
    self, ResourceTargetVariant,
    action::Action,
    alerter::Alerter,
    build::Build,
    builder::Builder,
    deployment::Deployment,
    komodo_timestamp,
    permission::PermissionLevel,
    procedure::Procedure,
    repo::Repo,
    server::Server,
    stack::Stack,
    sync::ResourceSync,
    update::{Log, Update},
    user::sync_user,
  },
};
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  helpers::{
    all_resources::AllResourcesById, query::get_id_to_tags,
    update::update_update,
  },
  permission::get_check_permissions,
  state::{action_states, db_client},
  sync::{
    ResourceSyncTrait,
    deploy::{
      SyncDeployParams, build_deploy_cache, deploy_from_cache,
    },
    execute::{ExecuteResourceSync, get_updates_for_execution},
    remote::RemoteResources,
  },
};

use super::ExecuteArgs;

impl Resolve<ExecuteArgs> for RunSync {
  #[instrument(name = "RunSync", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let RunSync {
      sync,
      resource_type: match_resource_type,
      resources: match_resources,
    } = self;
    let sync = get_check_permissions::<entities::sync::ResourceSync>(
      &sync,
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

    // get the action state for the sync (or insert default).
    let action_state = action_states()
      .sync
      .get_or_insert_default(&sync.id)
      .await;

    // This will set action state back to default when dropped.
    // Will also check to ensure sync not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.syncing = true)?;

    let mut update = update.clone();

    // Send update here for FE to recheck action state
    update_update(update.clone()).await?;

    let RemoteResources {
      resources,
      logs,
      hash,
      message,
      file_errors,
      ..
    } =
      crate::sync::remote::get_remote_resources(&sync, repo.as_ref())
        .await
        .context("failed to get remote resources")?;

    update.logs.extend(logs);
    update_update(update.clone()).await?;

    if !file_errors.is_empty() {
      return Err(
        anyhow!("Found file errors. Cannot execute sync.").into(),
      );
    }

    let resources = resources?;

    let id_to_tags = get_id_to_tags(None).await?;
    let all_resources = AllResourcesById::load().await?;
    // Convert all match_resources to names
    let match_resources = match_resources.map(|resources| {
      resources
        .into_iter()
        .filter_map(|name_or_id| {
          let Some(resource_type) = match_resource_type else {
            return Some(name_or_id);
          };
          match ObjectId::from_str(&name_or_id) {
            Ok(_) => match resource_type {
              ResourceTargetVariant::Alerter => all_resources
                .alerters
                .get(&name_or_id)
                .map(|a| a.name.clone()),
              ResourceTargetVariant::Build => all_resources
                .builds
                .get(&name_or_id)
                .map(|b| b.name.clone()),
              ResourceTargetVariant::Builder => all_resources
                .builders
                .get(&name_or_id)
                .map(|b| b.name.clone()),
              ResourceTargetVariant::Deployment => all_resources
                .deployments
                .get(&name_or_id)
                .map(|d| d.name.clone()),
              ResourceTargetVariant::Procedure => all_resources
                .procedures
                .get(&name_or_id)
                .map(|p| p.name.clone()),
              ResourceTargetVariant::Action => all_resources
                .actions
                .get(&name_or_id)
                .map(|p| p.name.clone()),
              ResourceTargetVariant::Repo => all_resources
                .repos
                .get(&name_or_id)
                .map(|r| r.name.clone()),
              ResourceTargetVariant::Server => all_resources
                .servers
                .get(&name_or_id)
                .map(|s| s.name.clone()),
              ResourceTargetVariant::Stack => all_resources
                .stacks
                .get(&name_or_id)
                .map(|s| s.name.clone()),
              ResourceTargetVariant::ResourceSync => all_resources
                .syncs
                .get(&name_or_id)
                .map(|s| s.name.clone()),
              ResourceTargetVariant::System => None,
            },
            Err(_) => Some(name_or_id),
          }
        })
        .collect::<Vec<_>>()
    });

    let deployments_by_name = all_resources
      .deployments
      .values()
      .filter(|deployment| {
        Deployment::include_resource(
          &deployment.name,
          &deployment.config,
          match_resource_type,
          match_resources.as_deref(),
          &deployment.tags,
          &id_to_tags,
          &sync.config.match_tags,
        )
      })
      .map(|deployment| (deployment.name.clone(), deployment.clone()))
      .collect::<HashMap<_, _>>();
    let stacks_by_name = all_resources
      .stacks
      .values()
      .filter(|stack| {
        Stack::include_resource(
          &stack.name,
          &stack.config,
          match_resource_type,
          match_resources.as_deref(),
          &stack.tags,
          &id_to_tags,
          &sync.config.match_tags,
        )
      })
      .map(|stack| (stack.name.clone(), stack.clone()))
      .collect::<HashMap<_, _>>();

    let deploy_cache = build_deploy_cache(SyncDeployParams {
      deployments: &resources.deployments,
      deployment_map: &deployments_by_name,
      stacks: &resources.stacks,
      stack_map: &stacks_by_name,
    })
    .await?;

    let delete = sync.config.managed || sync.config.delete;

    let server_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Server>(
        resources.servers,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let stack_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Stack>(
        resources.stacks,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let deployment_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Deployment>(
        resources.deployments,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let build_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Build>(
        resources.builds,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let repo_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Repo>(
        resources.repos,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let procedure_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Procedure>(
        resources.procedures,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let action_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Action>(
        resources.actions,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let builder_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Builder>(
        resources.builders,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let alerter_deltas = if sync.config.include_resources {
      get_updates_for_execution::<Alerter>(
        resources.alerters,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };
    let resource_sync_deltas = if sync.config.include_resources {
      get_updates_for_execution::<entities::sync::ResourceSync>(
        resources.resource_syncs,
        delete,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?
    } else {
      Default::default()
    };

    let (
      variables_to_create,
      variables_to_update,
      variables_to_delete,
    ) = if match_resource_type.is_none()
      && match_resources.is_none()
      && sync.config.include_variables
    {
      crate::sync::variables::get_updates_for_execution(
        resources.variables,
        delete,
      )
      .await?
    } else {
      Default::default()
    };
    let (
      user_groups_to_create,
      user_groups_to_update,
      user_groups_to_delete,
    ) = if match_resource_type.is_none()
      && match_resources.is_none()
      && sync.config.include_user_groups
    {
      crate::sync::user_groups::get_updates_for_execution(
        resources.user_groups,
        delete,
      )
      .await?
    } else {
      Default::default()
    };

    if deploy_cache.is_empty()
      && resource_sync_deltas.no_changes()
      && server_deltas.no_changes()
      && deployment_deltas.no_changes()
      && stack_deltas.no_changes()
      && build_deltas.no_changes()
      && builder_deltas.no_changes()
      && alerter_deltas.no_changes()
      && repo_deltas.no_changes()
      && procedure_deltas.no_changes()
      && action_deltas.no_changes()
      && user_groups_to_create.is_empty()
      && user_groups_to_update.is_empty()
      && user_groups_to_delete.is_empty()
      && variables_to_create.is_empty()
      && variables_to_update.is_empty()
      && variables_to_delete.is_empty()
    {
      update.push_simple_log(
        "No Changes",
        format!(
          "{}. exiting.",
          colored("nothing to do", Color::Green)
        ),
      );
      update.finalize();
      update_update(update.clone()).await?;
      return Ok(update);
    }

    // =================

    // No deps
    maybe_extend(
      &mut update.logs,
      crate::sync::variables::run_updates(
        variables_to_create,
        variables_to_update,
        variables_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      crate::sync::user_groups::run_updates(
        user_groups_to_create,
        user_groups_to_update,
        user_groups_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      ResourceSync::execute_sync_updates(resource_sync_deltas).await,
    );
    maybe_extend(
      &mut update.logs,
      Server::execute_sync_updates(server_deltas).await,
    );
    maybe_extend(
      &mut update.logs,
      Alerter::execute_sync_updates(alerter_deltas).await,
    );
    maybe_extend(
      &mut update.logs,
      Action::execute_sync_updates(action_deltas).await,
    );

    // Dependent on server
    maybe_extend(
      &mut update.logs,
      Builder::execute_sync_updates(builder_deltas).await,
    );
    maybe_extend(
      &mut update.logs,
      Repo::execute_sync_updates(repo_deltas).await,
    );

    // Dependant on builder
    maybe_extend(
      &mut update.logs,
      Build::execute_sync_updates(build_deltas).await,
    );

    // Dependant on server / build
    maybe_extend(
      &mut update.logs,
      Deployment::execute_sync_updates(deployment_deltas).await,
    );
    // stack only depends on server, but maybe will depend on build later.
    maybe_extend(
      &mut update.logs,
      Stack::execute_sync_updates(stack_deltas).await,
    );

    // Dependant on everything
    maybe_extend(
      &mut update.logs,
      Procedure::execute_sync_updates(procedure_deltas).await,
    );

    // Execute the deploy cache
    deploy_from_cache(deploy_cache, &mut update.logs).await;

    let db = db_client();

    if let Err(e) = update_one_by_id(
      &db.resource_syncs,
      &sync.id,
      doc! {
        "$set": {
          "info.last_sync_ts": komodo_timestamp(),
          "info.last_sync_hash": hash,
          "info.last_sync_message": message,
        }
      },
      None,
    )
    .await
    {
      warn!(
        "failed to update resource sync {} info after sync | {e:#}",
        sync.name
      )
    }

    if let Err(e) = (RefreshResourceSyncPending { sync: sync.id })
      .resolve(&WriteArgs {
        user: sync_user().to_owned(),
      })
      .await
    {
      warn!(
        "failed to refresh sync {} after run | {:#}",
        sync.name, e.error
      );
      update.push_error_log(
        "refresh sync",
        format_serror(
          &e.error
            .context("failed to refresh sync pending after run")
            .into(),
        ),
      );
    }

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

fn maybe_extend(logs: &mut Vec<Log>, log: Option<Log>) {
  if let Some(log) = log {
    logs.push(log);
  }
}
