use anyhow::Context;
use formatting::format_serror;
use komodo_client::{
  api::execute::*,
  entities::{
    all_logs_success,
    permission::PermissionLevel,
    server::Server,
    update::{Log, Update},
  },
};
use periphery_client::api;
use resolver_api::Resolve;

use crate::{
  helpers::{periphery_client, update::update_update},
  monitor::update_cache_for_server,
  permission::get_check_permissions,
  state::action_states,
};

use super::ExecuteArgs;

impl Resolve<ExecuteArgs> for StartContainer {
  #[instrument(name = "StartContainer", skip(self, user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure deployment not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.starting_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::StartContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "start container",
        format_serror(&e.context("failed to start container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RestartContainer {
  #[instrument(name = "RestartContainer", skip(self, user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the deployment (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.restarting_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::RestartContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "restart container",
        format_serror(
          &e.context("failed to restart container").into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PauseContainer {
  #[instrument(name = "PauseContainer", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pausing_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::PauseContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "pause container",
        format_serror(&e.context("failed to pause container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for UnpauseContainer {
  #[instrument(name = "UnpauseContainer", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.unpausing_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::UnpauseContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "unpause container",
        format_serror(
          &e.context("failed to unpause container").into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StopContainer {
  #[instrument(name = "StopContainer", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.stopping_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::StopContainer {
        name: self.container,
        signal: self.signal,
        time: self.time,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "stop container",
        format_serror(&e.context("failed to stop container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for DestroyContainer {
  #[instrument(name = "DestroyContainer", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let DestroyContainer {
      server,
      container,
      signal,
      time,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_containers = true)?;

    let mut update = update.clone();

    // Send update after setting action state, this way frontend gets correct state.
    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::RemoveContainer {
        name: container,
        signal,
        time,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "stop container",
        format_serror(&e.context("failed to stop container").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StartAllContainers {
  #[instrument(name = "StartAllContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.starting_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let logs = periphery_client(&server)?
      .request(api::container::StartAllContainers {})
      .await
      .context("failed to start all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "start all containers",
        String::from("All containers have been started on the host."),
      );
    }

    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for RestartAllContainers {
  #[instrument(name = "RestartAllContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.restarting_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let logs = periphery_client(&server)?
      .request(api::container::RestartAllContainers {})
      .await
      .context("failed to restart all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "restart all containers",
        String::from(
          "All containers have been restarted on the host.",
        ),
      );
    }

    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PauseAllContainers {
  #[instrument(name = "PauseAllContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pausing_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let logs = periphery_client(&server)?
      .request(api::container::PauseAllContainers {})
      .await
      .context("failed to pause all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "pause all containers",
        String::from("All containers have been paused on the host."),
      );
    }

    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for UnpauseAllContainers {
  #[instrument(name = "UnpauseAllContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.unpausing_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let logs = periphery_client(&server)?
      .request(api::container::UnpauseAllContainers {})
      .await
      .context("failed to unpause all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "unpause all containers",
        String::from(
          "All containers have been unpaused on the host.",
        ),
      );
    }

    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for StopAllContainers {
  #[instrument(name = "StopAllContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard = action_state
      .update(|state| state.stopping_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let logs = periphery_client(&server)?
      .request(api::container::StopAllContainers {})
      .await
      .context("failed to stop all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "stop all containers",
        String::from("All containers have been stopped on the host."),
      );
    }

    update_cache_for_server(&server, true).await;
    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneContainers {
  #[instrument(name = "PruneContainers", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_containers = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::container::PruneContainers {})
      .await
      .context(format!(
        "failed to prune containers on server {}",
        server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "prune containers",
        format_serror(
          &e.context("failed to prune containers").into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for DeleteNetwork {
  #[instrument(name = "DeleteNetwork", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::network::DeleteNetwork {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "failed to delete network {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "delete network",
        format_serror(
          &e.context(format!(
            "failed to delete network {}",
            self.name
          ))
          .into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneNetworks {
  #[instrument(name = "PruneNetworks", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_networks = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::network::PruneNetworks {})
      .await
      .context(format!(
        "failed to prune networks on server {}",
        server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "prune networks",
        format_serror(&e.context("failed to prune networks").into()),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for DeleteImage {
  #[instrument(name = "DeleteImage", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::image::DeleteImage {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "failed to delete image {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "delete image",
        format_serror(
          &e.context(format!("failed to delete image {}", self.name))
            .into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneImages {
  #[instrument(name = "PruneImages", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_images = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log =
      match periphery.request(api::image::PruneImages {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "prune images",
          format!(
            "failed to prune images on server {} | {e:#?}",
            server.name
          ),
        ),
      };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for DeleteVolume {
  #[instrument(name = "DeleteVolume", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery
      .request(api::volume::DeleteVolume {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "failed to delete volume {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "delete volume",
        format_serror(
          &e.context(format!(
            "failed to delete volume {}",
            self.name
          ))
          .into(),
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneVolumes {
  #[instrument(name = "PruneVolumes", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_volumes = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log =
      match periphery.request(api::volume::PruneVolumes {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "prune volumes",
          format!(
            "failed to prune volumes on server {} | {e:#?}",
            server.name
          ),
        ),
      };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneDockerBuilders {
  #[instrument(name = "PruneDockerBuilders", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_builders = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log =
      match periphery.request(api::build::PruneBuilders {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "prune builders",
          format!(
            "failed to docker builder prune on server {} | {e:#?}",
            server.name
          ),
        ),
      };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneBuildx {
  #[instrument(name = "PruneBuildx", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_buildx = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log =
      match periphery.request(api::build::PruneBuildx {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "prune buildx",
          format!(
            "failed to docker buildx prune on server {} | {e:#?}",
            server.name
          ),
        ),
      };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<ExecuteArgs> for PruneSystem {
  #[instrument(name = "PruneSystem", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    // get the action state for the server (or insert default).
    let action_state = action_states()
      .server
      .get_or_insert_default(&server.id)
      .await;

    // Will check to ensure server not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.pruning_system = true)?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server)?;

    let log = match periphery.request(api::PruneSystem {}).await {
      Ok(log) => log,
      Err(e) => Log::error(
        "prune system",
        format!(
          "failed to docker system prune on server {} | {e:#?}",
          server.name
        ),
      ),
    };

    update.logs.push(log);
    update_cache_for_server(&server, true).await;

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
