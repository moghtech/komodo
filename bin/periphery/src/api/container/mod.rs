use anyhow::Context;
use command::{
  run_komodo_shell_command, run_komodo_standard_command,
};
use futures_util::future::join_all;
use komodo_client::entities::{
  docker::{
    container::{Container, ContainerListItem, ContainerStats},
    stats::FullContainerStats,
  },
  update::Log,
};
use periphery_client::api::container::*;
use resolver_api::Resolve;

use crate::{
  docker::{stats::get_container_stats, stop_container_command},
  helpers::format_log_grep,
  state::docker_client,
};

mod run;

// ======
//  READ
// ======

//

impl Resolve<crate::api::Args> for InspectContainer {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Container> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_container(&self.name).await
  }
}

//

impl Resolve<crate::api::Args> for GetContainerLog {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetContainerLog {
      name,
      tail,
      timestamps,
    } = self;
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command =
      format!("docker logs {name} --tail {tail}{timestamps}");
    Ok(
      run_komodo_standard_command("Get container log", None, command)
        .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for GetContainerLogSearch {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetContainerLogSearch {
      name,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let grep = format_log_grep(&terms, combinator, invert);
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command = format!(
      "docker logs {name} --tail 5000{timestamps} 2>&1 | {grep}"
    );
    Ok(
      run_komodo_shell_command(
        "Get container log grep",
        None,
        command,
      )
      .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for GetContainerStats {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<ContainerStats> {
    let mut stats = get_container_stats(Some(self.name)).await?;
    let stats =
      stats.pop().context("No stats found for container")?;
    Ok(stats)
  }
}

//

impl Resolve<crate::api::Args> for GetFullContainerStats {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<FullContainerStats> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.full_container_stats(&self.name).await
  }
}

//

impl Resolve<crate::api::Args> for GetContainerStatsList {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Vec<ContainerStats>> {
    get_container_stats(None).await
  }
}

// =========
//  ACTIONS
// =========

impl Resolve<crate::api::Args> for StartContainer {
  #[instrument(
    "StartContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      run_komodo_standard_command(
        "Docker Start",
        None,
        format!("docker start {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for RestartContainer {
  #[instrument(
    "RestartContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      run_komodo_standard_command(
        "Docker Restart",
        None,
        format!("docker restart {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for PauseContainer {
  #[instrument(
    "PauseContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      run_komodo_standard_command(
        "Docker Pause",
        None,
        format!("docker pause {}", self.name),
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for UnpauseContainer {
  #[instrument(
    "UnpauseContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    Ok(
      run_komodo_standard_command(
        "Docker Unpause",
        None,
        format!("docker unpause {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for StopContainer {
  #[instrument(
    "StopContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let StopContainer { name, signal, time } = self;
    let command = stop_container_command(&name, signal, time);
    let log =
      run_komodo_standard_command("Docker Stop", None, command).await;
    if log.stderr.contains("unknown flag: --signal") {
      let command = stop_container_command(&name, None, time);
      let mut log =
        run_komodo_standard_command("Docker Stop", None, command)
          .await;
      log.stderr = format!(
        "old docker version: unable to use --signal flag{}",
        if !log.stderr.is_empty() {
          format!("\n\n{}", log.stderr)
        } else {
          String::new()
        }
      );
      Ok(log)
    } else {
      Ok(log)
    }
  }
}

//

impl Resolve<crate::api::Args> for RemoveContainer {
  #[instrument(
    "RemoveContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      container = self.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let RemoveContainer { name, signal, time } = self;
    let stop_command = stop_container_command(&name, signal, time);
    let command =
      format!("{stop_command} && docker container rm {name}");
    let log = run_komodo_shell_command(
      "Docker Stop and Remove",
      None,
      command,
    )
    .await;
    if log.stderr.contains("unknown flag: --signal") {
      let stop_command = stop_container_command(&name, None, time);
      let command =
        format!("{stop_command} && docker container rm {name}");
      let mut log = run_komodo_shell_command(
        "Docker Stop and Remove",
        None,
        command,
      )
      .await;
      log.stderr = format!(
        "Old docker version: unable to use --signal flag{}",
        if !log.stderr.is_empty() {
          format!("\n\n{}", log.stderr)
        } else {
          String::new()
        }
      );
      Ok(log)
    } else {
      Ok(log)
    }
  }
}

//

impl Resolve<crate::api::Args> for RenameContainer {
  #[instrument(
    "RenameContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      current = self.curr_name,
      new = self.new_name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let RenameContainer {
      curr_name,
      new_name,
    } = self;
    let command = format!("docker rename {curr_name} {new_name}");
    Ok(
      run_komodo_standard_command("Docker Rename", None, command)
        .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for PruneContainers {
  #[instrument(
    "PruneContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let command = String::from("docker container prune -f");
    Ok(
      run_komodo_standard_command("Prune Containers", None, command)
        .await,
    )
  }
}

//

impl Resolve<crate::api::Args> for StartAllContainers {
  #[instrument(
    "StartAllContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
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
    let containers = client
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if let Some(skip) = labels.get("komodo.skip")
          && skip != "false"
        {
          return None;
        }
        let command = format!("docker start {name}");
        Some(async move {
          run_komodo_standard_command(&command.clone(), None, command)
            .await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<crate::api::Args> for RestartAllContainers {
  #[instrument(
    "RestartAllContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
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
    let containers = client
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if let Some(skip) = labels.get("komodo.skip")
          && skip != "false"
        {
          return None;
        }
        let command = format!("docker restart {name}");
        Some(async move {
          run_komodo_standard_command(&command.clone(), None, command)
            .await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<crate::api::Args> for PauseAllContainers {
  #[instrument(
    "PauseAllContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
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
    let containers = client
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if let Some(skip) = labels.get("komodo.skip")
          && skip != "false"
        {
          return None;
        }
        let command = format!("docker pause {name}");
        Some(async move {
          run_komodo_standard_command(&command.clone(), None, command)
            .await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<crate::api::Args> for UnpauseAllContainers {
  #[instrument(
    "UnpauseAllContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
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
    let containers = client
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if let Some(skip) = labels.get("komodo.skip")
          && skip != "false"
        {
          return None;
        }
        let command = format!("docker unpause {name}");
        Some(async move {
          run_komodo_standard_command(&command.clone(), None, command)
            .await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<crate::api::Args> for StopAllContainers {
  #[instrument(
    "StopAllContainers",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core
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
    let containers = client
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if let Some(skip) = labels.get("komodo.skip")
          && skip != "false"
        {
          return None;
        }
        Some(async move {
          run_komodo_standard_command(
            &format!("Docker stop {name}"),
            None,
            stop_container_command(name, None, None),
          )
          .await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}
