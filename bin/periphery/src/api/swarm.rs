use anyhow::Context as _;
use command::{
  run_komodo_shell_command, run_komodo_standard_command,
};
use komodo_client::entities::{
  docker::{
    SwarmLists, config::SwarmConfig, node::SwarmNode,
    secret::SwarmSecret, service::SwarmService,
    stack::SwarmStackLists, task::SwarmTask,
  },
  update::Log,
};
use periphery_client::api::swarm::*;
use resolver_api::Resolve;

use crate::{
  docker::{
    config::{inspect_swarm_config, list_swarm_configs},
    stack::{inspect_swarm_stack, list_swarm_stacks},
  },
  helpers::format_log_grep,
  state::docker_client,
};

impl Resolve<super::Args> for PollSwarmStatus {
  async fn resolve(
    self,
    _: &super::Args,
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

impl Resolve<super::Args> for InspectSwarmNode {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<SwarmNode> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_node(&self.node).await
  }
}

impl Resolve<super::Args> for UpdateSwarmNode {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
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

impl Resolve<super::Args> for RemoveSwarmNodes {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
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

// =======
//  Stack
// =======

impl Resolve<super::Args> for InspectSwarmStack {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<SwarmStackLists> {
    inspect_swarm_stack(self.stack).await
  }
}

impl Resolve<super::Args> for RemoveSwarmStacks {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
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

// =========
//  Service
// =========

impl Resolve<super::Args> for InspectSwarmService {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<SwarmService> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_service(&self.service).await
  }
}

impl Resolve<super::Args> for GetSwarmServiceLog {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
    let GetSwarmServiceLog {
      service,
      tail,
      timestamps,
      no_task_ids,
      no_resolve,
      details,
    } = self;
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let no_task_ids = if no_task_ids {
      " --no-task-ids"
    } else {
      Default::default()
    };
    let no_resolve = if no_resolve {
      " --no-resolve"
    } else {
      Default::default()
    };
    let details = if details {
      " --details"
    } else {
      Default::default()
    };
    let command = format!(
      "docker service logs --tail {tail}{timestamps}{no_task_ids}{no_resolve}{details} {service}",
    );
    Ok(
      run_komodo_standard_command(
        "Get Swarm Service Log",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<super::Args> for GetSwarmServiceLogSearch {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
    let GetSwarmServiceLogSearch {
      service,
      terms,
      combinator,
      invert,
      timestamps,
      no_task_ids,
      no_resolve,
      details,
    } = self;
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let no_task_ids = if no_task_ids {
      " --no-task-ids"
    } else {
      Default::default()
    };
    let no_resolve = if no_resolve {
      " --no-resolve"
    } else {
      Default::default()
    };
    let details = if details {
      " --details"
    } else {
      Default::default()
    };
    let grep = format_log_grep(&terms, combinator, invert);
    let command = format!(
      "docker service logs --tail 5000{timestamps}{no_task_ids}{no_resolve}{details} {service} 2>&1 | {grep}",
    );
    Ok(
      run_komodo_shell_command(
        "Search Swarm Service Log",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<super::Args> for RemoveSwarmServices {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
    let mut command = String::from("docker service rm");
    for service in self.services {
      command += " ";
      command += &service;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Services",
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

impl Resolve<super::Args> for InspectSwarmTask {
  async fn resolve(
    self,
    _: &super::Args,
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

impl Resolve<super::Args> for InspectSwarmConfig {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<Vec<SwarmConfig>> {
    inspect_swarm_config(&self.config).await
  }
}

impl Resolve<super::Args> for RemoveSwarmConfigs {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
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

impl Resolve<super::Args> for InspectSwarmSecret {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<SwarmSecret> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_secret(&self.secret).await
  }
}

impl Resolve<super::Args> for RemoveSwarmSecrets {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
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
