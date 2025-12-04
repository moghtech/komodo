use anyhow::Context as _;
use command::run_komodo_standard_command;
use komodo_client::entities::{
  docker::{
    SwarmLists, config::SwarmConfig, node::SwarmNode,
    secret::SwarmSecret, task::SwarmTask,
  },
  update::Log,
};
use periphery_client::api::swarm::*;
use resolver_api::Resolve;

use crate::{
  docker::{
    config::{inspect_swarm_config, list_swarm_configs},
    stack::list_swarm_stacks,
  },
  state::docker_client,
};

mod service;
mod stack;

impl Resolve<crate::api::Args> for PollSwarmStatus {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<PollSwarmStatusResponse> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    let (inspect, nodes, services, tasks, secrets, configs, stacks) = tokio::join!(
      client.inspect_swarm(),
      client.list_swarm_nodes(),
      client.list_swarm_services(),
      client.list_swarm_tasks(),
      client.list_swarm_secrets(),
      list_swarm_configs(),
      list_swarm_stacks(),
    );
    Ok(PollSwarmStatusResponse {
      inspect: inspect.ok(),
      lists: SwarmLists {
        nodes: nodes.unwrap_or_default(),
        services: services.unwrap_or_default(),
        tasks: tasks.unwrap_or_default(),
        secrets: secrets.unwrap_or_default(),
        configs: configs.unwrap_or_default(),
        stacks: stacks.unwrap_or_default(),
      },
    })
  }
}

// ======
//  Node
// ======

impl Resolve<crate::api::Args> for InspectSwarmNode {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmNode> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_node(&self.node).await
  }
}

impl Resolve<crate::api::Args> for UpdateSwarmNode {
  #[instrument(
    "UpdateSwarmNode",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      node = self.node,
      update = serde_json::to_string(&self).unwrap_or_else(|e| e.to_string())
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let mut command = String::from("docker node update");

    if let Some(role) = self.role {
      command += " --role=";
      command += role.as_ref();
    }

    if let Some(availability) = self.availability {
      command += " --availability=";
      command += availability.as_ref();
    }

    if let Some(label_add) = self.label_add {
      for (key, value) in label_add {
        command += " --label-add ";
        command += &key;
        if let Some(value) = value {
          command += "=";
          command += &value;
        }
      }
    }

    if let Some(label_rm) = self.label_rm {
      for key in label_rm {
        command += " --label-rm ";
        command += &key;
      }
    }

    command += " ";
    command += &self.node;

    Ok(
      run_komodo_standard_command("Update Swarm Node", None, command)
        .await,
    )
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmNodes {
  #[instrument(
    "RemoveSwarmNodes",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      nodes = serde_json::to_string(&self.nodes).unwrap_or_else(|e| e.to_string()),
      force = self.force,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let mut command = String::from("docker node rm");
    if self.force {
      command += " --force"
    }
    for node in self.nodes {
      command += " ";
      command += &node;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Nodes",
        None,
        command,
      )
      .await,
    )
  }
}

// ======
//  Task
// ======

impl Resolve<crate::api::Args> for InspectSwarmTask {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmTask> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_task(&self.task).await
  }
}

// ========
//  Config
// ========

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
    let mut command = String::from("docker config rm");
    for config in self.configs {
      command += " ";
      command += &config;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Configs",
        None,
        command,
      )
      .await,
    )
  }
}

// ========
//  Secret
// ========

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
    let mut command = String::from("docker secret rm");
    for secret in self.secrets {
      command += " ";
      command += &secret;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Secrets",
        None,
        command,
      )
      .await,
    )
  }
}
