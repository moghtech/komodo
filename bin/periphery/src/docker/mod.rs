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

/// Returns whether build result should be pushed after build
#[instrument(skip(registry_token))]
pub async fn docker_login(
  domain: &str,
  account: &str,
  // For local token override from core.
  registry_token: Option<&str>,
) -> anyhow::Result<bool> {
  if domain.is_empty() || account.is_empty() {
    return Ok(false);
  }
  let registry_token = match registry_token {
    Some(token) => token,
    None => crate::helpers::registry_token(domain, account)?,
  };
  let log = async_run_command(&format!(
    "echo {registry_token} | docker login {domain} --username '{account}' --password-stdin",
  ))
  .await;
  if log.success() {
    Ok(true)
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

#[instrument]
pub async fn pull_image(image: &str) -> Log {
  let command = format!("docker pull {image}");
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
