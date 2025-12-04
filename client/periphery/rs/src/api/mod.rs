use komodo_client::entities::{
  FileContents,
  config::{DockerRegistry, GitProvider},
  stack::{StackRemoteFileContents, StackServiceNames},
  update::Log,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

pub mod build;
pub mod compose;
pub mod container;
pub mod docker;
pub mod git;
pub mod keys;
pub mod poll;
pub mod stats;
pub mod swarm;
pub mod terminal;

//

#[derive(Deserialize, Debug, Clone)]
pub struct CoreConnectionQuery {
  /// Core host (eg demo.komo.do)
  pub core: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PeripheryConnectionQuery {
  /// Server Id or name
  pub server: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(GetHealthResponse)]
#[error(anyhow::Error)]
pub struct GetHealth {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetHealthResponse {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(GetVersionResponse)]
#[error(anyhow::Error)]
pub struct GetVersion {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetVersionResponse {
  pub version: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(ListGitProvidersResponse)]
#[error(anyhow::Error)]
pub struct ListGitProviders {}

pub type ListGitProvidersResponse = Vec<GitProvider>;

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(ListDockerRegistriesResponse)]
#[error(anyhow::Error)]
pub struct ListDockerRegistries {}

pub type ListDockerRegistriesResponse = Vec<DockerRegistry>;

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<String>)]
#[error(anyhow::Error)]
pub struct ListSecrets {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct PruneSystem {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployStackResponse {
  /// If any of the required files are missing, they will be here.
  pub missing_files: Vec<String>,
  /// The logs produced by the deploy
  pub logs: Vec<Log>,
  /// Whether stack was successfully deployed
  pub deployed: bool,
  /// The stack services.
  ///
  /// Note. The "image" is after interpolation.
  pub services: Vec<StackServiceNames>,
  /// The deploy compose file contents if they could be acquired, or empty vec.
  pub file_contents: Vec<StackRemoteFileContents>,
  /// The error in getting remote file contents at the path, or null
  pub remote_errors: Vec<FileContents>,
  /// The output of `docker compose config` / `docker stack config` at deploy time
  pub merged_config: Option<String>,
  /// If its a repo based stack, will include the latest commit hash
  pub commit_hash: Option<String>,
  /// If its a repo based stack, will include the latest commit message
  pub commit_message: Option<String>,
}
