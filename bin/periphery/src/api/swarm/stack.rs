use command::run_komodo_standard_command;
use komodo_client::entities::{
  docker::stack::SwarmStack, update::Log,
};
use periphery_client::api::swarm::{
  DeploySwarmStack, InspectSwarmStack, RemoveSwarmStacks,
};
use resolver_api::Resolve;

use crate::docker::stack::inspect_swarm_stack;

impl Resolve<crate::api::Args> for InspectSwarmStack {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmStack> {
    inspect_swarm_stack(self.stack).await
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmStacks {
  #[instrument(
    "RemoveSwarmStacks",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stacks = serde_json::to_string(&self.stacks).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let mut command = String::from("docker stack rm");
    // This defaults to true, only need when false
    if !self.detach {
      command += " --detach=false"
    }
    for stack in self.stacks {
      command += " ";
      command += &stack;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Stacks",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for DeploySwarmStack {
  #[instrument(
    "DeploySwarmStack",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = self.stack.name,
      repo = self.repo.as_ref().map(|repo| &repo.name),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> Result<Self::Response, Self::Error> {
    todo!()
  }
}
