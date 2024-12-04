use std::path::PathBuf;

use komodo_client::entities::{
  update::Log, CloneArgs, EnvironmentVar, LatestCommit,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(LatestCommit)]
#[error(serror::Error)]
pub struct GetLatestCommit {
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(RepoActionResponse)]
#[error(serror::Error)]
pub struct CloneRepo {
  pub args: CloneArgs,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

fn default_env_file_path() -> String {
  String::from(".env")
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(RepoActionResponse)]
#[error(serror::Error)]
pub struct PullRepo {
  pub args: CloneArgs,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

//

/// Either pull or clone depending on whether it exists.
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(RepoActionResponse)]
#[error(serror::Error)]
pub struct PullOrCloneRepo {
  pub args: CloneArgs,
  #[serde(default)]
  pub environment: Vec<EnvironmentVar>,
  #[serde(default = "default_env_file_path")]
  pub env_file_path: String,
  #[serde(default)]
  pub skip_secret_interp: bool,
  /// Override git token with one sent from core.
  pub git_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

//

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoActionResponse {
  pub logs: Vec<Log>,
  pub commit_hash: Option<String>,
  pub commit_message: Option<String>,
  /// Don't need to send this one to core, its only needed for calls local to single periphery
  #[serde(skip_serializing)]
  pub env_file_path: Option<PathBuf>,
}

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
}
