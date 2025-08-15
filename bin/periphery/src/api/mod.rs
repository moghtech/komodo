use anyhow::Context;
use command::run_komodo_command;
use derive_variants::EnumVariants;
use futures::TryFutureExt;
use komodo_client::entities::{
  SystemCommand,
  config::{DockerRegistry, GitProvider},
  update::Log,
};
use periphery_client::api::{
  build::*, compose::*, container::*, git::*, image::*, network::*,
  stats::*, terminal::*, volume::*, *,
};
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};

use crate::{config::periphery_config, docker::docker_client};

mod build;
mod compose;
mod container;
mod deploy;
mod git;
mod image;
mod network;
mod router;
mod stats;
mod terminal;
mod volume;

pub use router::router;

pub struct Args;

#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[args(Args)]
#[response(Response)]
#[error(serror::Error)]
#[variant_derive(Debug)]
#[serde(tag = "type", content = "params")]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
pub enum PeripheryRequest {
  GetVersion(GetVersion),
  GetHealth(GetHealth),

  // Config (Read)
  ListGitProviders(ListGitProviders),
  ListDockerRegistries(ListDockerRegistries),
  ListSecrets(ListSecrets),

  // Stats / Info (Read)
  GetSystemInformation(GetSystemInformation),
  GetSystemStats(GetSystemStats),
  GetSystemProcesses(GetSystemProcesses),
  GetLatestCommit(GetLatestCommit),

  // Generic shell execution
  RunCommand(RunCommand),

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
  Deploy(Deploy),
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

  // All in one (Read)
  GetDockerLists(GetDockerLists),

  // All in one (Write)
  PruneSystem(PruneSystem),

  // Terminal
  ListTerminals(ListTerminals),
  CreateTerminal(CreateTerminal),
  DeleteTerminal(DeleteTerminal),
  DeleteAllTerminals(DeleteAllTerminals),
  CreateTerminalAuthToken(CreateTerminalAuthToken),
}

//

impl Resolve<Args> for GetHealth {
  #[instrument(name = "GetHealth", level = "debug", skip_all)]
  async fn resolve(
    self,
    _: &Args,
  ) -> serror::Result<GetHealthResponse> {
    Ok(GetHealthResponse {})
  }
}

//

impl Resolve<Args> for GetVersion {
  #[instrument(name = "GetVersion", level = "debug", skip(self))]
  async fn resolve(
    self,
    _: &Args,
  ) -> serror::Result<GetVersionResponse> {
    Ok(GetVersionResponse {
      version: env!("CARGO_PKG_VERSION").to_string(),
    })
  }
}

//

impl Resolve<Args> for ListGitProviders {
  #[instrument(name = "ListGitProviders", level = "debug", skip_all)]
  async fn resolve(
    self,
    _: &Args,
  ) -> serror::Result<Vec<GitProvider>> {
    Ok(periphery_config().git_providers.0.clone())
  }
}

impl Resolve<Args> for ListDockerRegistries {
  #[instrument(
    name = "ListDockerRegistries",
    level = "debug",
    skip_all
  )]
  async fn resolve(
    self,
    _: &Args,
  ) -> serror::Result<Vec<DockerRegistry>> {
    Ok(periphery_config().docker_registries.0.clone())
  }
}

//

impl Resolve<Args> for ListSecrets {
  #[instrument(name = "ListSecrets", level = "debug", skip_all)]
  async fn resolve(self, _: &Args) -> serror::Result<Vec<String>> {
    Ok(
      periphery_config()
        .secrets
        .keys()
        .cloned()
        .collect::<Vec<_>>(),
    )
  }
}

impl Resolve<Args> for GetDockerLists {
  #[instrument(name = "GetDockerLists", level = "debug", skip_all)]
  async fn resolve(
    self,
    _: &Args,
  ) -> serror::Result<GetDockerListsResponse> {
    let docker = docker_client();
    let containers =
      docker.list_containers().await.map_err(Into::into);
    // Should still try to retrieve other docker lists, but "in_use" will be false for images, networks, volumes
    let _containers = match &containers {
      Ok(containers) => containers.as_slice(),
      Err(_) => &[],
    };
    let (networks, images, volumes, projects) = tokio::join!(
      docker.list_networks(_containers).map_err(Into::into),
      docker.list_images(_containers).map_err(Into::into),
      docker.list_volumes(_containers).map_err(Into::into),
      ListComposeProjects {}
        .resolve(&Args)
        .map_err(|e| e.error.into())
    );
    Ok(GetDockerListsResponse {
      containers,
      networks,
      images,
      volumes,
      projects,
    })
  }
}

impl Resolve<Args> for RunCommand {
  #[instrument(name = "RunCommand")]
  async fn resolve(self, _: &Args) -> serror::Result<Log> {
    let RunCommand {
      command: SystemCommand { path, command },
    } = self;
    let res = tokio::spawn(async move {
      let command = if path.is_empty() {
        command
      } else {
        format!("cd {path} && {command}")
      };
      run_komodo_command("run command", None, command).await
    })
    .await
    .context("failure in spawned task")?;
    Ok(res)
  }
}

impl Resolve<Args> for PruneSystem {
  #[instrument(name = "PruneSystem", skip_all)]
  async fn resolve(self, _: &Args) -> serror::Result<Log> {
    let command = String::from("docker system prune -a -f --volumes");
    Ok(run_komodo_command("Prune System", None, command).await)
  }
}
