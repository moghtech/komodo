use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::anyhow;
use bollard::Docker;
use command::run_komodo_command;
use komodo_client::entities::{TerminationSignal, update::Log};
use run_command::async_run_command;

pub mod stats;

mod containers;
mod images;
mod networks;
mod volumes;

pub fn docker_client() -> &'static DockerClient {
  static DOCKER_CLIENT: OnceLock<DockerClient> = OnceLock::new();
  DOCKER_CLIENT.get_or_init(Default::default)
}

pub struct DockerClient {
  docker: Docker,
}

impl Default for DockerClient {
  fn default() -> DockerClient {
    DockerClient {
      docker: Docker::connect_with_defaults()
        .expect("failed to connect to docker daemon"),
    }
  }
}

/// Returns a docker config directory path if login was performed.
/// Uses a per-(domain, account) config directory so that multiple
/// accounts on the same registry domain don't overwrite each other's
/// credentials (fixes #1063).
#[instrument(skip(registry_token))]
pub async fn docker_login(
  domain: &str,
  account: &str,
  // For local token override from core.
  registry_token: Option<&str>,
) -> anyhow::Result<Option<PathBuf>> {
  if domain.is_empty() || account.is_empty() {
    return Ok(None);
  }
  let registry_token = match registry_token {
    Some(token) => token,
    None => crate::helpers::registry_token(domain, account)?,
  };
  let config_dir = docker_config_dir(domain, account);
  // Ensure the config directory exists
  std::fs::create_dir_all(&config_dir).ok();
  let config_arg = config_dir.display();
  let log = async_run_command(&format!(
    "echo {registry_token} | docker --config '{config_arg}' login {domain} --username '{account}' --password-stdin",
  ))
  .await;
  if log.success() {
    Ok(Some(config_dir))
  } else {
    let mut e = anyhow!("End of trace");
    for line in
      log.stderr.split('\n').filter(|line| !line.is_empty()).rev()
    {
      e = e.context(line.to_string());
    }
    for line in
      log.stdout.split('\n').filter(|line| !line.is_empty()).rev()
    {
      e = e.context(line.to_string());
    }
    Err(e.context(format!("Registry {domain} login error")))
  }
}

/// Returns a stable, per-(domain, account) config directory path
/// under `/tmp/komodo-docker-config/`.
pub fn docker_config_dir(domain: &str, account: &str) -> PathBuf {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  let mut hasher = DefaultHasher::new();
  domain.hash(&mut hasher);
  account.hash(&mut hasher);
  let hash = hasher.finish();
  PathBuf::from(format!(
    "/tmp/komodo-docker-config/{domain}-{hash:x}"
  ))
}

/// Format a `--config` flag for docker commands if a config dir is provided.
pub fn docker_config_flag(config_dir: &Option<PathBuf>) -> String {
  match config_dir {
    Some(dir) => format!(" --config '{}'", dir.display()),
    None => String::new(),
  }
}

#[instrument]
pub async fn pull_image(
  image: &str,
  config_dir: &Option<PathBuf>,
) -> Log {
  let config_flag = docker_config_flag(config_dir);
  let command = format!("docker{config_flag} pull {image}");
  run_komodo_command("Docker Pull", None, command).await
}

pub fn stop_container_command(
  container_name: &str,
  signal: Option<TerminationSignal>,
  time: Option<i32>,
) -> String {
  let signal = signal
    .map(|signal| format!(" --signal {signal}"))
    .unwrap_or_default();
  let time = time
    .map(|time| format!(" --time {time}"))
    .unwrap_or_default();
  format!("docker stop{signal}{time} {container_name}")
}
