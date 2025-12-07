use command::run_komodo_standard_command;
use derive_variants::EnumVariants;
use encoding::{EncodedJsonMessage, EncodedResponse};
use komodo_client::entities::{
  config::{DockerRegistry, GitProvider},
  stats::SystemProcess,
  update::Log,
};
use periphery_client::api::{
  build::*, compose::*, container::*, docker::*, git::*, keys::*,
  poll::*, stats::*, swarm::*, terminal::*, *,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::periphery_config, state::stats_client};

pub mod terminal;

mod build;
mod compose;
mod container;
mod docker;
mod git;
mod keys;
mod poll;
mod swarm;

#[derive(Debug)]
pub struct Args {
  pub core: String,
  /// The execution id.
  /// Unique for every /execute call.
  pub id: Uuid,
}

#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[args(Args)]
#[response(EncodedResponse<EncodedJsonMessage>)]
#[error(anyhow::Error)]
#[variant_derive(Debug)]
#[serde(tag = "type", content = "params")]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
pub enum PeripheryRequest {
  // Stats / Info (Read)
  PollStatus(PollStatus),
  GetHealth(GetHealth),
  GetVersion(GetVersion),
  GetSystemProcesses(GetSystemProcesses),
  GetLatestCommit(GetLatestCommit),

  // Config (Read)
  ListGitProviders(ListGitProviders),
  ListDockerRegistries(ListDockerRegistries),
  ListSecrets(ListSecrets),

  // Repo (Write)
  CloneRepo(CloneRepo),
  PullRepo(PullRepo),
  PullOrCloneRepo(PullOrCloneRepo),
  RenameRepo(RenameRepo),
  DeleteRepo(DeleteRepo),

  // Build
  GetDockerfileContentsOnHost(GetDockerfileContentsOnHost),
  WriteDockerfileContentsToHost(WriteDockerfileContentsToHost),
  Build(Build),
  PruneBuilders(PruneBuilders),
  PruneBuildx(PruneBuildx),

  // Compose (Read)
  GetComposeContentsOnHost(GetComposeContentsOnHost),
  GetComposeLog(GetComposeLog),
  GetComposeLogSearch(GetComposeLogSearch),

  // Compose (Write)
  WriteComposeContentsToHost(WriteComposeContentsToHost),
  WriteCommitComposeContents(WriteCommitComposeContents),
  ComposePull(ComposePull),
  ComposeUp(ComposeUp),
  ComposeExecution(ComposeExecution),
  ComposeRun(ComposeRun),

  // Container (Read)
  InspectContainer(InspectContainer),
  GetContainerLog(GetContainerLog),
  GetContainerLogSearch(GetContainerLogSearch),
  GetContainerStats(GetContainerStats),
  GetContainerStatsList(GetContainerStatsList),
  GetFullContainerStats(GetFullContainerStats),

  // Container (Write)
  RunContainer(RunContainer),
  StartContainer(StartContainer),
  RestartContainer(RestartContainer),
  PauseContainer(PauseContainer),
  UnpauseContainer(UnpauseContainer),
  StopContainer(StopContainer),
  StartAllContainers(StartAllContainers),
  RestartAllContainers(RestartAllContainers),
  PauseAllContainers(PauseAllContainers),
  UnpauseAllContainers(UnpauseAllContainers),
  StopAllContainers(StopAllContainers),
  RemoveContainer(RemoveContainer),
  RenameContainer(RenameContainer),
  PruneContainers(PruneContainers),

  // Networks (Read)
  InspectNetwork(InspectNetwork),

  // Networks (Write)
  CreateNetwork(CreateNetwork),
  DeleteNetwork(DeleteNetwork),
  PruneNetworks(PruneNetworks),

  // Image (Read)
  InspectImage(InspectImage),
  ImageHistory(ImageHistory),

  // Image (Write)
  PullImage(PullImage),
  DeleteImage(DeleteImage),
  PruneImages(PruneImages),

  // Volume (Read)
  InspectVolume(InspectVolume),

  // Volume (Write)
  DeleteVolume(DeleteVolume),
  PruneVolumes(PruneVolumes),

  // All in one (Write)
  PruneSystem(PruneSystem),

  // Swarm (Read)
  PollSwarmStatus(PollSwarmStatus),
  InspectSwarmNode(InspectSwarmNode),
  InspectSwarmStack(InspectSwarmStack),
  InspectSwarmService(InspectSwarmService),
  GetSwarmServiceLog(GetSwarmServiceLog),
  GetSwarmServiceLogSearch(GetSwarmServiceLogSearch),
  InspectSwarmTask(InspectSwarmTask),
  InspectSwarmConfig(InspectSwarmConfig),
  InspectSwarmSecret(InspectSwarmSecret),

  // Swarm (Write)
  UpdateSwarmNode(UpdateSwarmNode),
  RemoveSwarmNodes(RemoveSwarmNodes),
  DeploySwarmStack(DeploySwarmStack),
  RemoveSwarmStacks(RemoveSwarmStacks),
  CreateSwarmService(CreateSwarmService),
  UpdateSwarmService(UpdateSwarmService),
  RemoveSwarmServices(RemoveSwarmServices),
  RemoveSwarmConfigs(RemoveSwarmConfigs),
  RemoveSwarmSecrets(RemoveSwarmSecrets),

  // Terminal
  ListTerminals(ListTerminals),
  CreateServerTerminal(CreateServerTerminal),
  CreateContainerExecTerminal(CreateContainerExecTerminal),
  CreateContainerAttachTerminal(CreateContainerAttachTerminal),
  DeleteTerminal(DeleteTerminal),
  DeleteAllTerminals(DeleteAllTerminals),
  ConnectTerminal(ConnectTerminal),
  DisconnectTerminal(DisconnectTerminal),
  ExecuteTerminal(ExecuteTerminal),

  // Keys
  RotatePrivateKey(RotatePrivateKey),
  RotateCorePublicKey(RotateCorePublicKey),
}

//

impl Resolve<Args> for GetHealth {
  async fn resolve(
    self,
    _: &Args,
  ) -> anyhow::Result<GetHealthResponse> {
    Ok(GetHealthResponse {})
  }
}

//

impl Resolve<Args> for GetVersion {
  async fn resolve(
    self,
    _: &Args,
  ) -> anyhow::Result<GetVersionResponse> {
    Ok(GetVersionResponse {
      version: env!("CARGO_PKG_VERSION").to_string(),
    })
  }
}

//

impl Resolve<Args> for GetSystemProcesses {
  async fn resolve(
    self,
    _: &Args,
  ) -> anyhow::Result<Vec<SystemProcess>> {
    Ok(stats_client().read().await.get_processes())
  }
}

//

impl Resolve<Args> for ListGitProviders {
  async fn resolve(
    self,
    _: &Args,
  ) -> anyhow::Result<Vec<GitProvider>> {
    Ok(periphery_config().git_providers.0.clone())
  }
}

impl Resolve<Args> for ListDockerRegistries {
  async fn resolve(
    self,
    _: &Args,
  ) -> anyhow::Result<Vec<DockerRegistry>> {
    Ok(periphery_config().docker_registries.0.clone())
  }
}

//

impl Resolve<Args> for ListSecrets {
  async fn resolve(self, _: &Args) -> anyhow::Result<Vec<String>> {
    Ok(
      periphery_config()
        .secrets
        .keys()
        .cloned()
        .collect::<Vec<_>>(),
    )
  }
}

impl Resolve<Args> for PruneSystem {
  #[instrument(
    "PruneSystem",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
    )
  )]
  async fn resolve(self, args: &Args) -> anyhow::Result<Log> {
    let command = String::from("docker system prune -a -f --volumes");
    Ok(
      run_komodo_standard_command("Prune System", None, command)
        .await,
    )
  }
}
