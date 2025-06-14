use anyhow::anyhow;
use command::run_komodo_command;
use periphery_client::api::compose::ComposeUpResponse;

use crate::config::periphery_config;

pub mod up;
pub mod write;

pub fn docker_compose() -> &'static str {
  if periphery_config().legacy_compose_cli {
    "docker-compose"
  } else {
    "docker compose"
  }
}

async fn compose_down(
  project: &str,
  services: &[String],
  res: &mut ComposeUpResponse,
) -> anyhow::Result<()> {
  let docker_compose = docker_compose();
  let service_args = if services.is_empty() {
    String::new()
  } else {
    format!(" {}", services.join(" "))
  };
  let log = run_komodo_command(
    "Compose Down",
    None,
    format!("{docker_compose} -p {project} down{service_args}"),
  )
  .await;
  let success = log.success;
  res.logs.push(log);
  if !success {
    return Err(anyhow!(
      "Failed to bring down existing container(s) with docker compose down. Stopping run."
    ));
  }

  Ok(())
}
