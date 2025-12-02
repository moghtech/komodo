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
  update::{Update, UpdateListItem},
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
  let (operation, target) = match &request {
    // Swarm
    ExecuteRequest::RemoveSwarmNodes(data) => (
      Operation::RemoveSwarmNodes,
      ResourceTarget::Swarm(
        resource::get::<Swarm>(&data.swarm).await?.id,
      ),
    ),
    ExecuteRequest::RemoveSwarmStacks(data) => (
      Operation::RemoveSwarmStacks,
      ResourceTarget::Swarm(
        resource::get::<Swarm>(&data.swarm).await?.id,
      ),
    ),
    ExecuteRequest::RemoveSwarmServices(data) => (
      Operation::RemoveSwarmServices,
      ResourceTarget::Swarm(
        resource::get::<Swarm>(&data.swarm).await?.id,
      ),
    ),
    ExecuteRequest::RemoveSwarmConfigs(data) => (
      Operation::RemoveSwarmConfigs,
      ResourceTarget::Swarm(
        resource::get::<Swarm>(&data.swarm).await?.id,
      ),
    ),
    ExecuteRequest::RemoveSwarmSecrets(data) => (
      Operation::RemoveSwarmSecrets,
      ResourceTarget::Swarm(
        resource::get::<Swarm>(&data.swarm).await?.id,
      ),
    ),

    // Server
    ExecuteRequest::StartContainer(data) => (
      Operation::StartContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::RestartContainer(data) => (
      Operation::RestartContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PauseContainer(data) => (
      Operation::PauseContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::UnpauseContainer(data) => (
      Operation::UnpauseContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::StopContainer(data) => (
      Operation::StopContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::DestroyContainer(data) => (
      Operation::DestroyContainer,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::StartAllContainers(data) => (
      Operation::StartAllContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::RestartAllContainers(data) => (
      Operation::RestartAllContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PauseAllContainers(data) => (
      Operation::PauseAllContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::UnpauseAllContainers(data) => (
      Operation::UnpauseAllContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::StopAllContainers(data) => (
      Operation::StopAllContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneContainers(data) => (
      Operation::PruneContainers,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::DeleteNetwork(data) => (
      Operation::DeleteNetwork,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneNetworks(data) => (
      Operation::PruneNetworks,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::DeleteImage(data) => (
      Operation::DeleteImage,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneImages(data) => (
      Operation::PruneImages,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::DeleteVolume(data) => (
      Operation::DeleteVolume,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneVolumes(data) => (
      Operation::PruneVolumes,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneDockerBuilders(data) => (
      Operation::PruneDockerBuilders,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneBuildx(data) => (
      Operation::PruneBuildx,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),
    ExecuteRequest::PruneSystem(data) => (
      Operation::PruneSystem,
      ResourceTarget::Server(
        resource::get::<Server>(&data.server).await?.id,
      ),
    ),

    // Deployment
    ExecuteRequest::Deploy(data) => (
      Operation::Deploy,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::BatchDeploy(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::PullDeployment(data) => (
      Operation::PullDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::StartDeployment(data) => (
      Operation::StartDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::RestartDeployment(data) => (
      Operation::RestartDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::PauseDeployment(data) => (
      Operation::PauseDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::UnpauseDeployment(data) => (
      Operation::UnpauseDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::StopDeployment(data) => (
      Operation::StopDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::DestroyDeployment(data) => (
      Operation::DestroyDeployment,
      ResourceTarget::Deployment(
        resource::get::<Deployment>(&data.deployment).await?.id,
      ),
    ),
    ExecuteRequest::BatchDestroyDeployment(_data) => {
      return Ok(Default::default());
    }

    // Build
    ExecuteRequest::RunBuild(data) => (
      Operation::RunBuild,
      ResourceTarget::Build(
        resource::get::<Build>(&data.build).await?.id,
      ),
    ),
    ExecuteRequest::BatchRunBuild(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::CancelBuild(data) => (
      Operation::CancelBuild,
      ResourceTarget::Build(
        resource::get::<Build>(&data.build).await?.id,
      ),
    ),

    // Repo
    ExecuteRequest::CloneRepo(data) => (
      Operation::CloneRepo,
      ResourceTarget::Repo(
        resource::get::<Repo>(&data.repo).await?.id,
      ),
    ),
    ExecuteRequest::BatchCloneRepo(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::PullRepo(data) => (
      Operation::PullRepo,
      ResourceTarget::Repo(
        resource::get::<Repo>(&data.repo).await?.id,
      ),
    ),
    ExecuteRequest::BatchPullRepo(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::BuildRepo(data) => (
      Operation::BuildRepo,
      ResourceTarget::Repo(
        resource::get::<Repo>(&data.repo).await?.id,
      ),
    ),
    ExecuteRequest::BatchBuildRepo(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::CancelRepoBuild(data) => (
      Operation::CancelRepoBuild,
      ResourceTarget::Repo(
        resource::get::<Repo>(&data.repo).await?.id,
      ),
    ),

    // Procedure
    ExecuteRequest::RunProcedure(data) => (
      Operation::RunProcedure,
      ResourceTarget::Procedure(
        resource::get::<Procedure>(&data.procedure).await?.id,
      ),
    ),
    ExecuteRequest::BatchRunProcedure(_) => {
      return Ok(Default::default());
    }

    // Action
    ExecuteRequest::RunAction(data) => (
      Operation::RunAction,
      ResourceTarget::Action(
        resource::get::<Action>(&data.action).await?.id,
      ),
    ),
    ExecuteRequest::BatchRunAction(_) => {
      return Ok(Default::default());
    }

    // Resource Sync
    ExecuteRequest::RunSync(data) => (
      Operation::RunSync,
      ResourceTarget::ResourceSync(
        resource::get::<ResourceSync>(&data.sync).await?.id,
      ),
    ),

    // Stack
    ExecuteRequest::DeployStack(data) => (
      if !data.services.is_empty() {
        Operation::DeployStackService
      } else {
        Operation::DeployStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::BatchDeployStack(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::DeployStackIfChanged(data) => (
      Operation::DeployStack,
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::BatchDeployStackIfChanged(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::StartStack(data) => (
      if !data.services.is_empty() {
        Operation::StartStackService
      } else {
        Operation::StartStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::PullStack(data) => (
      if !data.services.is_empty() {
        Operation::PullStackService
      } else {
        Operation::PullStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::BatchPullStack(_data) => {
      return Ok(Default::default());
    }
    ExecuteRequest::RestartStack(data) => (
      if !data.services.is_empty() {
        Operation::RestartStackService
      } else {
        Operation::RestartStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::PauseStack(data) => (
      if !data.services.is_empty() {
        Operation::PauseStackService
      } else {
        Operation::PauseStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::UnpauseStack(data) => (
      if !data.services.is_empty() {
        Operation::UnpauseStackService
      } else {
        Operation::UnpauseStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::StopStack(data) => (
      if !data.services.is_empty() {
        Operation::StopStackService
      } else {
        Operation::StopStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::DestroyStack(data) => (
      if !data.services.is_empty() {
        Operation::DestroyStackService
      } else {
        Operation::DestroyStack
      },
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),
    ExecuteRequest::BatchDestroyStack(_data) => {
      return Ok(Default::default());
    }

    ExecuteRequest::RunStackService(data) => (
      Operation::RunStackService,
      ResourceTarget::Stack(
        resource::get::<Stack>(&data.stack).await?.id,
      ),
    ),

    // Alerter
    ExecuteRequest::TestAlerter(data) => (
      Operation::TestAlerter,
      ResourceTarget::Alerter(
        resource::get::<Alerter>(&data.alerter).await?.id,
      ),
    ),
    ExecuteRequest::SendAlert(_) => {
      (Operation::SendAlert, ResourceTarget::system())
    }

    // Maintenance
    ExecuteRequest::ClearRepoCache(_data) => {
      (Operation::ClearRepoCache, ResourceTarget::system())
    }
    ExecuteRequest::BackupCoreDatabase(_data) => {
      (Operation::BackupCoreDatabase, ResourceTarget::system())
    }
    ExecuteRequest::GlobalAutoUpdate(_data) => {
      (Operation::GlobalAutoUpdate, ResourceTarget::system())
    }
    ExecuteRequest::RotateAllServerKeys(_data) => {
      (Operation::RotateAllServerKeys, ResourceTarget::system())
    }
    ExecuteRequest::RotateCoreKeys(_data) => {
      (Operation::RotateCoreKeys, ResourceTarget::system())
    }
  };

  let mut update = make_update(target, operation, user);
  update.in_progress();

  // Hold off on even adding update for DeployStackIfChanged
  if !matches!(&request, ExecuteRequest::DeployStackIfChanged(_)) {
    // Don't actually send it here, let the handlers send it after they can set action state.
    update.id = add_update_without_send(&update).await?;
  }

  Ok(update)
}
