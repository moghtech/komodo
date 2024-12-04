use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context};
use formatting::{colored, format_serror, Color};
use komodo_client::{
  api::{execute::RunSync, write::RefreshResourceSyncPending},
  entities::{
    self,
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
    server_template::ServerTemplate,
    stack::Stack,
    sync::ResourceSync,
    update::{Log, Update},
    user::sync_user,
    ResourceTargetVariant,
  },
};
use mongo_indexed::doc;
use mungos::{
  by_id::update_one_by_id,
  mongodb::bson::{oid::ObjectId, to_document},
};
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs,
  helpers::{query::get_id_to_tags, update::update_update},
  resource::{self, refresh_resource_sync_state_cache},
  state::{action_states, db_client},
  sync::{
    deploy::{
      build_deploy_cache, deploy_from_cache, SyncDeployParams,
    },
    execute::{get_updates_for_execution, ExecuteResourceSync},
    remote::RemoteResources,
    AllResourcesById, ResourceSyncTrait,
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
    let sync = resource::get_check_permissions::<
      entities::sync::ResourceSync,
    >(&sync, &user, PermissionLevel::Execute)
    .await?;

    // get the action state for the sync (or insert default).
    let action_state = action_states()
      .resource_sync
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
    } = crate::sync::remote::get_remote_resources(&sync)
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
              ResourceTargetVariant::ServerTemplate => all_resources
                .templates
                .get(&name_or_id)
                .map(|t| t.name.clone()),
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
      all_resources: &all_resources,
    })
    .await?;

    let delete = sync.config.managed || sync.config.delete;

    let (servers_to_create, servers_to_update, servers_to_delete) =
      get_updates_for_execution::<Server>(
        resources.servers,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (
      deployments_to_create,
      deployments_to_update,
      deployments_to_delete,
    ) = get_updates_for_execution::<Deployment>(
      resources.deployments,
      delete,
      &all_resources,
      match_resource_type,
      match_resources.as_deref(),
      &id_to_tags,
      &sync.config.match_tags,
    )
    .await?;
    let (stacks_to_create, stacks_to_update, stacks_to_delete) =
      get_updates_for_execution::<Stack>(
        resources.stacks,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (builds_to_create, builds_to_update, builds_to_delete) =
      get_updates_for_execution::<Build>(
        resources.builds,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (repos_to_create, repos_to_update, repos_to_delete) =
      get_updates_for_execution::<Repo>(
        resources.repos,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (
      procedures_to_create,
      procedures_to_update,
      procedures_to_delete,
    ) = get_updates_for_execution::<Procedure>(
      resources.procedures,
      delete,
      &all_resources,
      match_resource_type,
      match_resources.as_deref(),
      &id_to_tags,
      &sync.config.match_tags,
    )
    .await?;
    let (actions_to_create, actions_to_update, actions_to_delete) =
      get_updates_for_execution::<Action>(
        resources.actions,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (builders_to_create, builders_to_update, builders_to_delete) =
      get_updates_for_execution::<Builder>(
        resources.builders,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (alerters_to_create, alerters_to_update, alerters_to_delete) =
      get_updates_for_execution::<Alerter>(
        resources.alerters,
        delete,
        &all_resources,
        match_resource_type,
        match_resources.as_deref(),
        &id_to_tags,
        &sync.config.match_tags,
      )
      .await?;
    let (
      server_templates_to_create,
      server_templates_to_update,
      server_templates_to_delete,
    ) = get_updates_for_execution::<ServerTemplate>(
      resources.server_templates,
      delete,
      &all_resources,
      match_resource_type,
      match_resources.as_deref(),
      &id_to_tags,
      &sync.config.match_tags,
    )
    .await?;
    let (
      resource_syncs_to_create,
      resource_syncs_to_update,
      resource_syncs_to_delete,
    ) = get_updates_for_execution::<entities::sync::ResourceSync>(
      resources.resource_syncs,
      delete,
      &all_resources,
      match_resource_type,
      match_resources.as_deref(),
      &id_to_tags,
      &sync.config.match_tags,
    )
    .await?;

    let (
      variables_to_create,
      variables_to_update,
      variables_to_delete,
    ) = if match_resource_type.is_none()
      && match_resources.is_none()
      && sync.config.match_tags.is_empty()
    {
      crate::sync::variables::get_updates_for_execution(
        resources.variables,
        // Delete doesn't work with variables when match tags are set
        sync.config.match_tags.is_empty() && delete,
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
      && sync.config.match_tags.is_empty()
    {
      crate::sync::user_groups::get_updates_for_execution(
        resources.user_groups,
        // Delete doesn't work with user groups when match tags are set
        sync.config.match_tags.is_empty() && delete,
        &all_resources,
      )
      .await?
    } else {
      Default::default()
    };

    if deploy_cache.is_empty()
      && resource_syncs_to_create.is_empty()
      && resource_syncs_to_update.is_empty()
      && resource_syncs_to_delete.is_empty()
      && server_templates_to_create.is_empty()
      && server_templates_to_update.is_empty()
      && server_templates_to_delete.is_empty()
      && servers_to_create.is_empty()
      && servers_to_update.is_empty()
      && servers_to_delete.is_empty()
      && deployments_to_create.is_empty()
      && deployments_to_update.is_empty()
      && deployments_to_delete.is_empty()
      && stacks_to_create.is_empty()
      && stacks_to_update.is_empty()
      && stacks_to_delete.is_empty()
      && builds_to_create.is_empty()
      && builds_to_update.is_empty()
      && builds_to_delete.is_empty()
      && builders_to_create.is_empty()
      && builders_to_update.is_empty()
      && builders_to_delete.is_empty()
      && alerters_to_create.is_empty()
      && alerters_to_update.is_empty()
      && alerters_to_delete.is_empty()
      && repos_to_create.is_empty()
      && repos_to_update.is_empty()
      && repos_to_delete.is_empty()
      && procedures_to_create.is_empty()
      && procedures_to_update.is_empty()
      && procedures_to_delete.is_empty()
      && actions_to_create.is_empty()
      && actions_to_update.is_empty()
      && actions_to_delete.is_empty()
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
      ResourceSync::execute_sync_updates(
        resource_syncs_to_create,
        resource_syncs_to_update,
        resource_syncs_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      ServerTemplate::execute_sync_updates(
        server_templates_to_create,
        server_templates_to_update,
        server_templates_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      Server::execute_sync_updates(
        servers_to_create,
        servers_to_update,
        servers_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      Alerter::execute_sync_updates(
        alerters_to_create,
        alerters_to_update,
        alerters_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      Action::execute_sync_updates(
        actions_to_create,
        actions_to_update,
        actions_to_delete,
      )
      .await,
    );

    // Dependent on server
    maybe_extend(
      &mut update.logs,
      Builder::execute_sync_updates(
        builders_to_create,
        builders_to_update,
        builders_to_delete,
      )
      .await,
    );
    maybe_extend(
      &mut update.logs,
      Repo::execute_sync_updates(
        repos_to_create,
        repos_to_update,
        repos_to_delete,
      )
      .await,
    );

    // Dependant on builder
    maybe_extend(
      &mut update.logs,
      Build::execute_sync_updates(
        builds_to_create,
        builds_to_update,
        builds_to_delete,
      )
      .await,
    );

    // Dependant on server / build
    maybe_extend(
      &mut update.logs,
      Deployment::execute_sync_updates(
        deployments_to_create,
        deployments_to_update,
        deployments_to_delete,
      )
      .await,
    );
    // stack only depends on server, but maybe will depend on build later.
    maybe_extend(
      &mut update.logs,
      Stack::execute_sync_updates(
        stacks_to_create,
        stacks_to_update,
        stacks_to_delete,
      )
      .await,
    );

    // Dependant on everything
    maybe_extend(
      &mut update.logs,
      Procedure::execute_sync_updates(
        procedures_to_create,
        procedures_to_update,
        procedures_to_delete,
      )
      .await,
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
      refresh_resource_sync_state_cache().await;
    }
    update_update(update.clone()).await?;

    Ok(update)
  }
}

fn maybe_extend(logs: &mut Vec<Log>, log: Option<Log>) {
  if let Some(log) = log {
    logs.push(log);
  }
}
