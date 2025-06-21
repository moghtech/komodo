use std::path::PathBuf;

use komodo_client::entities::{
  EnvironmentVar, LatestCommit, RepoExecutionArgs,
  RepoExecutionResponse, SystemCommand, update::Log,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

/// Returns `null` if not a repo
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Option<LatestCommit>)]
#[error(serror::Error)]
pub struct GetLatestCommit {
  pub name: String,
  pub path: Option<String>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(PeripheryRepoExecutionResponse)]
#[error(serror::Error)]
pub struct CloneRepo {
  pub args: RepoExecutionArgs,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  /// Relative to repo root
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  pub on_clone: Option<SystemCommand>,
  pub on_pull: Option<SystemCommand>,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

fn default_env_file_path() -> String {
  String::from(".env")
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(PeripheryRepoExecutionResponse)]
#[error(serror::Error)]
pub struct PullRepo {
  pub args: RepoExecutionArgs,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  pub on_pull: Option<SystemCommand>,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

//

/// Either pull or clone depending on whether it exists.
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(PeripheryRepoExecutionResponse)]
#[error(serror::Error)]
pub struct PullOrCloneRepo {
  pub args: RepoExecutionArgs,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  pub on_clone: Option<SystemCommand>,
  pub on_pull: Option<SystemCommand>,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeripheryRepoExecutionResponse {
  pub res: RepoExecutionResponse,
  pub env_file_path: Option<PathBuf>,
}

//

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct RenameRepo {
  pub curr_name: String,
  pub new_name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct DeleteRepo {
  pub name: String,
  /// Clears
  pub is_build: bool,
}
