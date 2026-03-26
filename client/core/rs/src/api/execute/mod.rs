use clap::{Parser, Subcommand};
use mogh_resolver::HasResponse;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumString};
use typeshare::typeshare;

mod action;
mod alerter;
mod build;
mod deployment;
mod maintenance;
mod procedure;
mod repo;
mod server;
mod stack;
mod swarm;
mod sync;

pub use action::*;
pub use alerter::*;
pub use build::*;
pub use deployment::*;
pub use maintenance::*;
pub use procedure::*;
pub use repo::*;
pub use server::*;
pub use stack::*;
pub use swarm::*;
pub use sync::*;

use crate::{
  api::write::CommitSync,
  entities::{_Serror, I64, NoData, update::Update},
};

#[cfg(feature = "utoipa")]
pub mod openapi;

pub trait KomodoExecuteRequest: HasResponse {}

/// A wrapper for all Komodo exections.
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  EnumDiscriminants,
  Subcommand,
)]
#[strum_discriminants(name(ExecutionVariant))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(
  not(feature = "utoipa"),
  strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr
  ))
)]
#[cfg_attr(
  feature = "utoipa",
  strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    utoipa::ToSchema
  ))
)]
#[serde(tag = "type", content = "params")]
pub enum Execution {
  /// The "null" execution. Does nothing.
  None(NoData),

  // STACK
  /// Deploy the target stack. (alias: `stack`, `st`)
  #[clap(alias = "stack", alias = "st")]
  DeployStack(DeployStack),
  BatchDeployStack(BatchDeployStack),
  DeployStackIfChanged(DeployStackIfChanged),
  BatchDeployStackIfChanged(BatchDeployStackIfChanged),
  PullStack(PullStack),
  BatchPullStack(BatchPullStack),
  StartStack(StartStack),
  RestartStack(RestartStack),
  PauseStack(PauseStack),
  UnpauseStack(UnpauseStack),
  StopStack(StopStack),
  DestroyStack(DestroyStack),
  BatchDestroyStack(BatchDestroyStack),
  RunStackService(RunStackService),

  // DEPLOYMENT
  /// Deploy the target deployment. (alias: `dp`)
  #[clap(alias = "dp")]
  Deploy(Deploy),
  BatchDeploy(BatchDeploy),
  PullDeployment(PullDeployment),
  StartDeployment(StartDeployment),
  RestartDeployment(RestartDeployment),
  PauseDeployment(PauseDeployment),
  UnpauseDeployment(UnpauseDeployment),
  StopDeployment(StopDeployment),
  DestroyDeployment(DestroyDeployment),
  BatchDestroyDeployment(BatchDestroyDeployment),

  // BUILD
  /// Run the target build. (alias: `build`, `bd`)
  #[clap(alias = "build", alias = "bd")]
  RunBuild(RunBuild),
  BatchRunBuild(BatchRunBuild),
  CancelBuild(CancelBuild),

  // REPO
  /// Clone the target repo
  #[clap(alias = "clone")]
  CloneRepo(CloneRepo),
  BatchCloneRepo(BatchCloneRepo),
  PullRepo(PullRepo),
  BatchPullRepo(BatchPullRepo),
  BuildRepo(BuildRepo),
  BatchBuildRepo(BatchBuildRepo),
  CancelRepoBuild(CancelRepoBuild),

  // PROCEDURE
  /// Run the target procedure. (alias: `procedure`, `pr`)
  #[clap(alias = "procedure", alias = "pr")]
  RunProcedure(RunProcedure),
  BatchRunProcedure(BatchRunProcedure),

  // ACTION
  /// Run the target action. (alias: `action`, `ac`)
  #[clap(alias = "action", alias = "ac")]
  RunAction(RunAction),
  BatchRunAction(BatchRunAction),

  // SYNC
  /// Execute a Resource Sync. (alias: `sync`)
  #[clap(alias = "sync")]
  RunSync(RunSync),
  /// Commit a Resource Sync. (alias: `commit`)
  #[clap(alias = "commit")]
  CommitSync(CommitSync), // This is a special case, its actually a write operation.

  // ALERTER
  TestAlerter(TestAlerter),
  #[clap(alias = "alert")]
  SendAlert(SendAlert),

  // SERVER
  StartContainer(StartContainer),
  RestartContainer(RestartContainer),
  PauseContainer(PauseContainer),
  UnpauseContainer(UnpauseContainer),
  StopContainer(StopContainer),
  DestroyContainer(DestroyContainer),
  StartAllContainers(StartAllContainers),
  RestartAllContainers(RestartAllContainers),
  PauseAllContainers(PauseAllContainers),
  UnpauseAllContainers(UnpauseAllContainers),
  StopAllContainers(StopAllContainers),
  PruneContainers(PruneContainers),
  DeleteNetwork(DeleteNetwork),
  PruneNetworks(PruneNetworks),
  DeleteImage(DeleteImage),
  PruneImages(PruneImages),
  DeleteVolume(DeleteVolume),
  PruneVolumes(PruneVolumes),
  PruneDockerBuilders(PruneDockerBuilders),
  PruneBuildx(PruneBuildx),
  PruneSystem(PruneSystem),

  // SWARM
  RemoveSwarmNodes(RemoveSwarmNodes),
  UpdateSwarmNode(UpdateSwarmNode),
  RemoveSwarmStacks(RemoveSwarmStacks),
  RemoveSwarmServices(RemoveSwarmServices),
  CreateSwarmConfig(CreateSwarmConfig),
  RotateSwarmConfig(RotateSwarmConfig),
  RemoveSwarmConfigs(RemoveSwarmConfigs),
  CreateSwarmSecret(CreateSwarmSecret),
  RotateSwarmSecret(RotateSwarmSecret),
  RemoveSwarmSecrets(RemoveSwarmSecrets),

  // MAINTENANCE
  ClearRepoCache(ClearRepoCache),
  #[clap(
    alias = "backup-database",
    alias = "backup-db",
    alias = "backup"
  )]
  BackupCoreDatabase(BackupCoreDatabase),
  #[clap(alias = "auto-update")]
  GlobalAutoUpdate(GlobalAutoUpdate),
  #[clap(alias = "rotate-keys")]
  RotateAllServerKeys(RotateAllServerKeys),
  RotateCoreKeys(RotateCoreKeys),

  // SLEEP
  Sleep(Sleep),
}

/// Sleeps for the specified time.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Parser)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Sleep {
  #[serde(default)]
  pub duration_ms: I64,
}

#[typeshare]
pub type BatchExecutionResponse = Vec<BatchExecutionResponseItem>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "status", content = "data")]
pub enum BatchExecutionResponseItem {
  Ok(Update),
  Err(BatchExecutionResponseItemErr),
}

impl From<Result<Box<Update>, BatchExecutionResponseItemErr>>
  for BatchExecutionResponseItem
{
  fn from(
    value: Result<Box<Update>, BatchExecutionResponseItemErr>,
  ) -> Self {
    match value {
      Ok(update) => Self::Ok(*update),
      Err(e) => Self::Err(e),
    }
  }
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BatchExecutionResponseItemErr {
  pub name: String,
  pub error: _Serror,
}
