use anyhow::Context as _;
use formatting::format_serror;
use komodo_client::entities::{
  docker::config::SwarmConfig, update::Log,
};
use periphery_client::api::swarm::*;
use resolver_api::Resolve;

use crate::{
  docker::config::{inspect_swarm_config, remove_swarm_configs},
  state::docker_client,
};

impl Resolve<crate::api::Args> for InspectSwarmConfig {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Vec<SwarmConfig>> {
    inspect_swarm_config(&self.config).await
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmConfigs {
  #[instrument(
    "RemoveSwarmConfigs",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      configs = serde_json::to_string(&self.configs).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      remove_swarm_configs(self.configs.iter().map(String::as_str))
        .await,
    )
  }
}

impl Resolve<crate::api::Args> for RotateSwarmConfig {
  #[instrument(
    "RotateSwarmConfig",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      config = self.config,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Vec<Log>> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;

    let mut logs = Vec::new();
    if let Err(e) = client
      .rotate_swarm_config(&self.config, &self.data, &mut logs)
      .await
    {
      logs.push(Log::error(
        "Rotate Swarm Config",
        format_serror(&e.into()),
      ))
    }

    Ok(logs)
  }
}
