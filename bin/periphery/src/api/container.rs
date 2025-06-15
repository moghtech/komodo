use anyhow::{Context, anyhow};
use command::run_komodo_command;
use futures::future::join_all;
use komodo_client::entities::{
  docker::container::{Container, ContainerListItem, ContainerStats},
  update::Log,
};
use periphery_client::api::container::*;
use resolver_api::Resolve;

use crate::{
  docker::{container_stats, docker_client, stop_container_command},
  helpers::log_grep,
};

// ======
//  READ
// ======

//

impl Resolve<super::Args> for InspectContainer {
  #[instrument(name = "InspectContainer", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Container> {
    Ok(docker_client().inspect_container(&self.name).await?)
  }
}

//

impl Resolve<super::Args> for GetContainerLog {
  #[instrument(name = "GetContainerLog", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let GetContainerLog {
      name,
      tail,
      timestamps,
    } = self;
    let timestamps =
      timestamps.then_some(" --timestamps").unwrap_or_default();
    let command =
      format!("docker logs {name} --tail {tail}{timestamps}");
    Ok(run_komodo_command("Get container log", None, command).await)
  }
}

//

impl Resolve<super::Args> for GetContainerLogSearch {
  #[instrument(name = "GetContainerLogSearch", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let GetContainerLogSearch {
      name,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let grep = log_grep(&terms, combinator, invert);
    let timestamps =
      timestamps.then_some(" --timestamps").unwrap_or_default();
    let command = format!(
      "docker logs {name} --tail 5000{timestamps} 2>&1 | {grep}"
    );
    Ok(
      run_komodo_command("Get container log grep", None, command)
        .await,
    )
  }
}

//

impl Resolve<super::Args> for GetContainerStats {
  #[instrument(name = "GetContainerStats", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<ContainerStats> {
    let error = anyhow!("no stats matching {}", self.name);
    let mut stats = container_stats(Some(self.name)).await?;
    let stats = stats.pop().ok_or(error)?;
    Ok(stats)
  }
}

//

impl Resolve<super::Args> for GetContainerStatsList {
  #[instrument(name = "GetContainerStatsList", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<ContainerStats>> {
    Ok(container_stats(None).await?)
  }
}

// =========
//  ACTIONS
// =========

impl Resolve<super::Args> for StartContainer {
  #[instrument(name = "StartContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    Ok(
      run_komodo_command(
        "Docker Start",
        None,
        format!("docker start {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<super::Args> for RestartContainer {
  #[instrument(name = "RestartContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    Ok(
      run_komodo_command(
        "Docker Restart",
        None,
        format!("docker restart {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<super::Args> for PauseContainer {
  #[instrument(name = "PauseContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    Ok(
      run_komodo_command(
        "Docker Pause",
        None,
        format!("docker pause {}", self.name),
      )
      .await,
    )
  }
}

impl Resolve<super::Args> for UnpauseContainer {
  #[instrument(name = "UnpauseContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    Ok(
      run_komodo_command(
        "Docker Unpause",
        None,
        format!("docker unpause {}", self.name),
      )
      .await,
    )
  }
}

//

impl Resolve<super::Args> for StopContainer {
  #[instrument(name = "StopContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let StopContainer { name, signal, time } = self;
    let command = stop_container_command(&name, signal, time);
    let log = run_komodo_command("Docker Stop", None, command).await;
    if log.stderr.contains("unknown flag: --signal") {
      let command = stop_container_command(&name, None, time);
      let mut log =
        run_komodo_command("Docker Stop", None, command).await;
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

impl Resolve<super::Args> for RemoveContainer {
  #[instrument(name = "RemoveContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let RemoveContainer { name, signal, time } = self;
    let stop_command = stop_container_command(&name, signal, time);
    let command =
      format!("{stop_command} && docker container rm {name}");
    let log =
      run_komodo_command("Docker Stop and Remove", None, command)
        .await;
    if log.stderr.contains("unknown flag: --signal") {
      let stop_command = stop_container_command(&name, None, time);
      let command =
        format!("{stop_command} && docker container rm {name}");
      let mut log =
        run_komodo_command("Docker Stop and Remove", None, command)
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

impl Resolve<super::Args> for RenameContainer {
  #[instrument(name = "RenameContainer")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let RenameContainer {
      curr_name,
      new_name,
    } = self;
    let command = format!("docker rename {curr_name} {new_name}");
    Ok(run_komodo_command("Docker Rename", None, command).await)
  }
}

//

impl Resolve<super::Args> for PruneContainers {
  #[instrument(name = "PruneContainers", skip_all)]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker container prune -f");
    Ok(run_komodo_command("Prune Containers", None, command).await)
  }
}

//

impl Resolve<super::Args> for StartAllContainers {
  #[instrument(name = "StartAllContainers", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let containers = docker_client()
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if labels.contains_key("komodo.skip") {
          return None;
        }
        let command = format!("docker start {name}");
        Some(async move {
          run_komodo_command(&command.clone(), None, command).await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<super::Args> for RestartAllContainers {
  #[instrument(name = "RestartAllContainers", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let containers = docker_client()
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if labels.contains_key("komodo.skip") {
          return None;
        }
        let command = format!("docker restart {name}");
        Some(async move {
          run_komodo_command(&command.clone(), None, command).await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<super::Args> for PauseAllContainers {
  #[instrument(name = "PauseAllContainers", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let containers = docker_client()
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if labels.contains_key("komodo.skip") {
          return None;
        }
        let command = format!("docker pause {name}");
        Some(async move {
          run_komodo_command(&command.clone(), None, command).await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<super::Args> for UnpauseAllContainers {
  #[instrument(name = "UnpauseAllContainers", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let containers = docker_client()
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if labels.contains_key("komodo.skip") {
          return None;
        }
        let command = format!("docker unpause {name}");
        Some(async move {
          run_komodo_command(&command.clone(), None, command).await
        })
      },
    );
    Ok(join_all(futures).await)
  }
}

//

impl Resolve<super::Args> for StopAllContainers {
  #[instrument(name = "StopAllContainers", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let containers = docker_client()
      .list_containers()
      .await
      .context("failed to list all containers on host")?;
    let futures = containers.iter().filter_map(
      |ContainerListItem { name, labels, .. }| {
        if labels.contains_key("komodo.skip") {
          return None;
        }
        Some(async move {
          run_komodo_command(
            &format!("docker stop {name}"),
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
