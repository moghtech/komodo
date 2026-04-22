use std::time::Duration;

use anyhow::Context;
use database::mungos::{
  by_id::{find_one_by_id, update_one_by_id},
  mongodb::bson::to_document,
};
use komodo_client::entities::{
  Operation, ResourceTarget,
  action::Action,
  alerter::Alerter,
  build::Build,
  deployment::Deployment,
  komodo_timestamp,
  procedure::Procedure,
  repo::Repo,
  server::Server,
  stack::Stack,
  swarm::Swarm,
  sync::ResourceSync,
  update::{Update, UpdateListItem, UpdateStatus},
  user::User,
};

use crate::{
  api::execute::ExecuteRequest, resource, state::db_client,
};

use super::channel::update_channel;

pub fn make_update(
  target: impl Into<ResourceTarget>,
  operation: Operation,
  user: &User,
) -> Update {
  Update {
    start_ts: komodo_timestamp(),
    target: target.into(),
    operation,
    operator: user.id.clone(),
    success: true,
    ..Default::default()
  }
}

pub async fn add_update(
  mut update: Update,
) -> anyhow::Result<String> {
  update.id = db_client()
    .updates
    .insert_one(&update)
    .await
    .context("failed to insert update into db")?
    .inserted_id
    .as_object_id()
    .context("inserted_id is not object id")?
    .to_string();
  let id = update.id.clone();
  let update = update_list_item(update).await?;
  let _ = send_update(update).await;
  Ok(id)
}

pub async fn add_update_without_send(
  update: &Update,
) -> anyhow::Result<String> {
  let id = db_client()
    .updates
    .insert_one(update)
    .await
    .context("failed to insert update into db")?
    .inserted_id
    .as_object_id()
    .context("inserted_id is not object id")?
    .to_string();
  Ok(id)
}

pub async fn update_update(update: Update) -> anyhow::Result<()> {
  update_one_by_id(&db_client().updates, &update.id, database::mungos::update::Update::Set(to_document(&update)?), None)
    .await
    .context("failed to update the update on db. the update build process was deleted")?;
  let update = update_list_item(update).await?;
  let _ = send_update(update).await;
  Ok(())
}

async fn update_list_item(
  update: Update,
) -> anyhow::Result<UpdateListItem> {
  let username = if User::is_service_user(&update.operator) {
    update.operator.clone()
  } else {
    find_one_by_id(&db_client().users, &update.operator)
      .await
      .context("failed to query mongo for user")?
      .with_context(|| {
        format!("no user found with id {}", update.operator)
      })?
      .username
  };
  let update = UpdateListItem {
    id: update.id,
    operation: update.operation,
    start_ts: update.start_ts,
    success: update.success,
    operator: update.operator,
    target: update.target,
    status: update.status,
    version: update.version,
    other_data: update.other_data,
    username,
  };
  Ok(update)
}

async fn send_update(update: UpdateListItem) -> anyhow::Result<()> {
  update_channel().sender.lock().await.send(update)?;
  Ok(())
}

pub async fn init_execution_update(
  request: &ExecuteRequest,
  user: &User,
) -> anyhow::Result<Update> {
  macro_rules! init_execution_match {
    (
      resource: [$(($Variant:ident, $ResType:ident, $field:ident)),* $(,)?],
      batch: [$($BatchVariant:ident),* $(,)?],
      stack_service: [$(($StackVariant:ident, $ServiceOp:ident)),* $(,)?],
      system: [$($SysVariant:ident),* $(,)?],
    ) => {
      match &request {
        $(
          ExecuteRequest::$Variant(data) => (
            Operation::$Variant,
            ResourceTarget::$ResType(
              resource::get::<$ResType>(&data.$field).await?.id,
            ),
          ),
        )*
        $(
          ExecuteRequest::$BatchVariant(_data) => {
            return Ok(Default::default());
          }
        )*
        $(
          ExecuteRequest::$StackVariant(data) => (
            if !data.services.is_empty() {
              Operation::$ServiceOp
            } else {
              Operation::$StackVariant
            },
            ResourceTarget::Stack(
              resource::get::<Stack>(&data.stack).await?.id,
            ),
          ),
        )*
        // DeployStackIfChanged doesn't have a service variant
        ExecuteRequest::DeployStackIfChanged(data) => (
          Operation::DeployStack,
          ResourceTarget::Stack(
            resource::get::<Stack>(&data.stack).await?.id,
          ),
        ),
        $(
          ExecuteRequest::$SysVariant(_data) => {
            (Operation::$SysVariant, ResourceTarget::system())
          }
        )*
      }
    };
  }

  let (operation, target) = init_execution_match!(
    resource: [
      // Swarm
      (RemoveSwarmNodes, Swarm, swarm),
      (UpdateSwarmNode, Swarm, swarm),
      (RemoveSwarmStacks, Swarm, swarm),
      (RemoveSwarmServices, Swarm, swarm),
      (CreateSwarmConfig, Swarm, swarm),
      (RotateSwarmConfig, Swarm, swarm),
      (RemoveSwarmConfigs, Swarm, swarm),
      (CreateSwarmSecret, Swarm, swarm),
      (RotateSwarmSecret, Swarm, swarm),
      (RemoveSwarmSecrets, Swarm, swarm),
      // Server
      (StartContainer, Server, server),
      (RestartContainer, Server, server),
      (PauseContainer, Server, server),
      (UnpauseContainer, Server, server),
      (StopContainer, Server, server),
      (DestroyContainer, Server, server),
      (StartAllContainers, Server, server),
      (RestartAllContainers, Server, server),
      (PauseAllContainers, Server, server),
      (UnpauseAllContainers, Server, server),
      (StopAllContainers, Server, server),
      (PruneContainers, Server, server),
      (DeleteNetwork, Server, server),
      (PruneNetworks, Server, server),
      (DeleteImage, Server, server),
      (PruneImages, Server, server),
      (DeleteVolume, Server, server),
      (PruneVolumes, Server, server),
      (PruneDockerBuilders, Server, server),
      (PruneBuildx, Server, server),
      (PruneSystem, Server, server),
      // Deployment
      (Deploy, Deployment, deployment),
      (PullDeployment, Deployment, deployment),
      (StartDeployment, Deployment, deployment),
      (RestartDeployment, Deployment, deployment),
      (PauseDeployment, Deployment, deployment),
      (UnpauseDeployment, Deployment, deployment),
      (StopDeployment, Deployment, deployment),
      (DestroyDeployment, Deployment, deployment),
      // Build
      (RunBuild, Build, build),
      (CancelBuild, Build, build),
      // Repo
      (CloneRepo, Repo, repo),
      (PullRepo, Repo, repo),
      (BuildRepo, Repo, repo),
      (CancelRepoBuild, Repo, repo),
      // Procedure
      (RunProcedure, Procedure, procedure),
      // Action
      (RunAction, Action, action),
      // Resource Sync
      (RunSync, ResourceSync, sync),
      // Stack (simple)
      (RunStackService, Stack, stack),
      // Alerter
      (TestAlerter, Alerter, alerter),
    ],
    batch: [
      BatchDeploy,
      BatchDestroyDeployment,
      BatchRunBuild,
      BatchCloneRepo,
      BatchPullRepo,
      BatchBuildRepo,
      BatchRunProcedure,
      BatchRunAction,
      BatchDeployStack,
      BatchDeployStackIfChanged,
      BatchPullStack,
      BatchDestroyStack,
    ],
    stack_service: [
      (DeployStack, DeployStackService),
      (PullStack, PullStackService),
      (StartStack, StartStackService),
      (RestartStack, RestartStackService),
      (PauseStack, PauseStackService),
      (UnpauseStack, UnpauseStackService),
      (StopStack, StopStackService),
      (DestroyStack, DestroyStackService),
    ],
    system: [
      SendAlert,
      ClearRepoCache,
      BackupCoreDatabase,
      GlobalAutoUpdate,
      RotateAllServerKeys,
      RotateCoreKeys,
    ],
  );

  let mut update = make_update(target, operation, user);
  update.in_progress();

  // Hold off on even adding update for DeployStackIfChanged
  if !matches!(&request, ExecuteRequest::DeployStackIfChanged(_)) {
    // Don't actually send it here, let the handlers send it after they can set action state.
    update.id = add_update_without_send(&update).await?;
  }

  Ok(update)
}

pub async fn poll_update_until_complete(
  update_id: &str,
) -> anyhow::Result<Update> {
  loop {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let update = find_one_by_id(&db_client().updates, update_id)
      .await?
      .context("No update found at given ID")?;
    if matches!(update.status, UpdateStatus::Complete) {
      return Ok(update);
    }
  }
}
