use komodo_client::entities::{
  FileContents, RepoExecutionResponse, SearchCombinator,
  repo::Repo,
  stack::{Stack, StackFileDependency, StackRemoteFileContents},
  update::Log,
};
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::{DeployStackResponse, ExternalEnvVar};

/// Direct service-scoped environment variable resolved from an external Secret Provider.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalServiceEnvBinding {
  pub variable: String,
  pub value: String,
  #[serde(default)]
  pub services: Vec<String>,
}

impl std::fmt::Debug for ExternalServiceEnvBinding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExternalServiceEnvBinding")
      .field("variable", &self.variable)
      .field("value", &"[REDACTED]")
      .field("services", &self.services)
      .finish()
  }
}

/// Runtime Compose file secret resolved from an external Secret Provider.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalFileSecretBinding {
  pub secret_name: String,
  #[serde(default)]
  pub bytes: Vec<u8>,
  #[serde(default)]
  pub services: Vec<String>,
  #[serde(default)]
  pub target_path: String,
  #[serde(default)]
  pub uid: String,
  #[serde(default)]
  pub gid: String,
  #[serde(default)]
  pub mode: Option<u32>,
}

impl std::fmt::Debug for ExternalFileSecretBinding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ExternalFileSecretBinding")
      .field("secret_name", &self.secret_name)
      .field("bytes_len", &self.bytes.len())
      .field("services", &self.services)
      .field("target_path", &self.target_path)
      .finish()
  }
}

//

/// Get the compose contents on the host, for stacks using
/// `files_on_host`.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(GetComposeContentsOnHostResponse)]
#[error(anyhow::Error)]
pub struct GetComposeContentsOnHost {
  /// The name of the stack
  pub name: String,
  pub run_directory: String,
  /// Both compose files and env / additional files, all relative to run directory.
  pub file_paths: Vec<StackFileDependency>,
  /// Relative file paths containing resolved external secrets (e.g. `.env`, `.komodo-secrets`)
  /// that must never be returned to Core's `StackInfo`.
  #[serde(default)]
  pub exclude_paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetComposeContentsOnHostResponse {
  pub contents: Vec<StackRemoteFileContents>,
  pub errors: Vec<FileContents>,
}

//

/// The stack folder must already exist for this to work
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct GetComposeLog {
  /// The name of the project
  pub project: String,
  /// Filter the logs to only ones from specific services.
  /// If empty, will include logs from all services.
  #[serde(default)]
  pub services: Vec<String>,
  /// Pass `--tail` for only recent log contents. Max of 5000
  #[serde(default = "default_tail")]
  pub tail: u64,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
}

fn default_tail() -> u64 {
  50
}

//

/// The stack folder must already exist for this to work
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct GetComposeLogSearch {
  /// The name of the project
  pub project: String,
  /// Filter the logs to only ones from specific services.
  /// If empty, will include logs from all services.
  #[serde(default)]
  pub services: Vec<String>,
  /// The search terms.
  pub terms: Vec<String>,
  /// And: Only lines matching all terms
  /// Or: Lines matching any one of the terms
  #[serde(default)]
  pub combinator: SearchCombinator,
  /// Invert the search (search for everything not matching terms)
  #[serde(default)]
  pub invert: bool,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
}

//

/// Write the compose / additional file contents to the file on the host, for stacks using
/// `files_on_host`.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct WriteComposeContentsToHost {
  /// The name of the stack
  pub name: String,
  /// The run directory of the stack
  pub run_directory: String,
  /// Relative to the stack folder + run directory,
  /// or absolute path.
  pub file_path: String,
  /// The contents to write.
  pub contents: String,
}

//

/// Write and commit compose contents.
/// Only works with git repo based stacks.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(RepoExecutionResponse)]
#[error(anyhow::Error)]
pub struct WriteCommitComposeContents {
  /// The stack to write to.
  pub stack: Stack,
  /// Optional linked repo.
  pub repo: Option<Repo>,
  /// The username of user which committed the file.
  pub username: Option<String>,
  /// Relative to the stack folder + run directory.
  pub file_path: String,
  /// The contents to write.
  pub contents: String,
  /// If provided, use it to login in. Otherwise check periphery local git providers.
  pub git_token: Option<String>,
}

//

/// Rewrites the compose directory, pulls any images, takes down existing containers,
/// and runs docker compose up. Response: [ComposePullResponse]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(ComposePullResponse)]
#[error(anyhow::Error)]
pub struct ComposePull {
  /// The stack to deploy
  pub stack: Stack,
  /// Filter to only pull specific services.
  /// If empty, will pull all services.
  #[serde(default)]
  pub services: Vec<String>,
  /// The linked repo, if it exists.
  pub repo: Option<Repo>,
  /// If provided, use it to login in. Otherwise check periphery local git providers.
  pub git_token: Option<String>,
  /// If provided, use it to login in. Otherwise check periphery local registry providers.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
  /// Resolved external `.env` variables (never re-interpolated on Periphery).
  #[serde(default)]
  pub external_env_vars: Vec<ExternalEnvVar>,
  /// Resolved external service-scoped environment bindings (`ServiceEnv`).
  #[serde(default)]
  pub service_env_bindings: Vec<ExternalServiceEnvBinding>,
  /// Resolved external Compose file secret bindings (`ComposeFileSecret`).
  #[serde(default)]
  pub file_secret_bindings: Vec<ExternalFileSecretBinding>,
  /// Secret-bearing env var keys for structured `merged_config` redaction.
  #[serde(default)]
  pub secret_env_keys: Vec<String>,
}

/// Response for [ComposePull]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposePullResponse {
  /// If any of the required files are missing, they will be here.
  pub missing_files: Vec<String>,
  /// The error in getting remote file contents at the path, or null
  pub remote_errors: Vec<FileContents>,
  /// The logs produced by the pull
  pub logs: Vec<Log>,
}

//

/// docker compose up.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(DeployStackResponse)]
#[error(anyhow::Error)]
pub struct ComposeUp {
  /// The stack to deploy
  pub stack: Stack,
  /// Filter to only deploy specific services.
  /// If empty, will deploy all services.
  #[serde(default)]
  pub services: Vec<String>,
  /// The linked repo, if it exists.
  pub repo: Option<Repo>,
  /// If provided, use it to login in. Otherwise check periphery local registries.
  pub git_token: Option<String>,
  /// If provided, use it to login in. Otherwise check periphery local git providers.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
  /// Resolved external `.env` variables (never re-interpolated on Periphery).
  #[serde(default)]
  pub external_env_vars: Vec<ExternalEnvVar>,
  /// Resolved external service-scoped environment bindings (`ServiceEnv`).
  #[serde(default)]
  pub service_env_bindings: Vec<ExternalServiceEnvBinding>,
  /// Resolved external Compose file secret bindings (`ComposeFileSecret`).
  #[serde(default)]
  pub file_secret_bindings: Vec<ExternalFileSecretBinding>,
  /// Secret-bearing env var keys for structured `merged_config` redaction.
  #[serde(default)]
  pub secret_env_keys: Vec<String>,
}

//

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeRunResponse {
  /// Logs produced during stack write/prepare for the run
  pub logs: Vec<Log>,
}

//

/// General compose command runner
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct ComposeExecution {
  /// The compose project name to run the execution on.
  /// Usually its he name of the stack / folder under the `stack_dir`.
  pub project: String,
  /// The command in `docker compose -p {project} {command}`
  pub command: String,
  /// Purge all runtime secret generations and manifest for `project` after `down` succeeds.
  #[serde(default)]
  pub cleanup_runtime_secrets: bool,
}

//

/// docker compose run one-time service execution.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct ComposeRun {
  /// The stack to run a service for
  pub stack: Stack,
  /// The linked repo, if it exists.
  pub repo: Option<Repo>,
  /// If provided, use it to login in. Otherwise check periphery local registries.
  pub git_token: Option<String>,
  /// If provided, use it to login in. Otherwise check periphery local git providers.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
  /// Resolved external `.env` variables (never re-interpolated on Periphery).
  #[serde(default)]
  pub external_env_vars: Vec<ExternalEnvVar>,
  /// Resolved external service-scoped environment bindings (`ServiceEnv`).
  #[serde(default)]
  pub service_env_bindings: Vec<ExternalServiceEnvBinding>,
  /// Resolved external Compose file secret bindings (`ComposeFileSecret`).
  #[serde(default)]
  pub file_secret_bindings: Vec<ExternalFileSecretBinding>,
  /// Secret-bearing env var keys for structured `merged_config` redaction.
  #[serde(default)]
  pub secret_env_keys: Vec<String>,

  /// Service to run
  pub service: String,
  /// Command
  #[serde(default)]
  pub command: Option<Vec<String>>,
  /// Do not allocate TTY
  #[serde(default)]
  pub no_tty: Option<bool>,
  /// Do not start linked services
  #[serde(default)]
  pub no_deps: Option<bool>,
  /// Detach container on run
  #[serde(default)]
  pub detach: Option<bool>,
  /// Map service ports to the host
  #[serde(default)]
  pub service_ports: Option<bool>,
  /// Extra environment variables for the run
  #[serde(default)]
  pub env: Option<HashMap<String, String>>,
  /// Working directory inside the container
  #[serde(default)]
  pub workdir: Option<String>,
  /// User to run as inside the container
  #[serde(default)]
  pub user: Option<String>,
  /// Override the default entrypoint
  #[serde(default)]
  pub entrypoint: Option<String>,
  /// Pull the image before running
  #[serde(default)]
  pub pull: Option<bool>,
}
