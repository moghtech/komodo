use anyhow::Context as _;
use formatting::format_serror;
use komodo_client::entities::{
  docker::secret::SwarmSecret, update::Log,
};
use mogh_resolver::Resolve;
use periphery_client::api::swarm::*;

use crate::{
  docker::secret::{create_swarm_secret, remove_swarm_secrets},
  state::docker_client,
};

impl Resolve<crate::api::Args> for InspectSwarmSecret {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmSecret> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_secret(&self.secret).await
  }
}

impl Resolve<crate::api::Args> for CreateSwarmSecret {
  #[instrument(
    "CreateSwarmSecret",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      config = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    create_swarm_secret(&self).await
  }
}

impl Resolve<crate::api::Args> for RotateSwarmSecret {
  #[instrument(
    "RotateSwarmSecret",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      secret = self.secret,
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
      .rotate_swarm_secret(&self.secret, self.data, &mut logs)
      .await
    {
      logs.push(Log::error(
        "Rotate Swarm Secret",
        format_serror(&e.into()),
      ))
    }

    Ok(logs)
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmSecrets {
  #[instrument(
    "RemoveSwarmSecrets",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      secrets = serde_json::to_string(&self.secrets).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      remove_swarm_secrets(self.secrets.iter().map(String::as_str))
        .await,
    )
  }
}
