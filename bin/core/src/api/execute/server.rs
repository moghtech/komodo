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
  #[instrument(
    "StartContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::container::StartContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "Start Container",
        format_serror(&e.context("Failed to start container").into()),
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
  #[instrument(
    "RestartContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::container::RestartContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "Restart Container",
        format_serror(
          &e.context("Failed to restart container").into(),
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
  #[instrument(
    "PauseContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::container::PauseContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "Pause Container",
        format_serror(&e.context("Failed to pause container").into()),
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
  #[instrument(
    "UnpauseContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::container::UnpauseContainer {
        name: self.container,
      })
      .await
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "Unpause Container",
        format_serror(
          &e.context("Failed to unpause container").into(),
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
  #[instrument(
    "StopContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
      signal = format!("{:?}", self.signal),
      time = self.time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

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
        "Stop Container",
        format_serror(&e.context("Failed to stop container").into()),
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
  #[instrument(
    "DestroyContainer",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      container = self.container,
      signal = format!("{:?}", self.signal),
      time = self.time,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

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
        "Remove Container",
        format_serror(
          &e.context("Failed to remove container").into(),
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

impl Resolve<ExecuteArgs> for StartAllContainers {
  #[instrument(
    "StartAllContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let logs = periphery_client(&server)
      .await?
      .request(api::container::StartAllContainers {})
      .await
      .context("Failed to start all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "Start All Containers",
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
  #[instrument(
    "RestartAllContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let logs = periphery_client(&server)
      .await?
      .request(api::container::RestartAllContainers {})
      .await
      .context("Failed to restart all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "Restart All Containers",
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
  #[instrument(
    "PauseAllContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let logs = periphery_client(&server)
      .await?
      .request(api::container::PauseAllContainers {})
      .await
      .context("Failed to pause all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "Pause All Containers",
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
  #[instrument(
    "UnpauseAllContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let logs = periphery_client(&server)
      .await?
      .request(api::container::UnpauseAllContainers {})
      .await
      .context("Failed to unpause all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "Unpause All Containers",
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
  #[instrument(
    "StopAllContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let logs = periphery_client(&server)
      .await?
      .request(api::container::StopAllContainers {})
      .await
      .context("Failed to stop all containers on host")?;

    update.logs.extend(logs);

    if all_logs_success(&update.logs) {
      update.push_simple_log(
        "Stop All Containers",
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
  #[instrument(
    "PruneContainers",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::container::PruneContainers {})
      .await
      .context(format!(
        "Failed to prune containers on server {}",
        server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "Prune Containers",
        format_serror(
          &e.context("Failed to prune containers").into(),
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
  #[instrument(
    "DeleteNetwork",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      network = self.name
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::docker::DeleteNetwork {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "Failed to delete network {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "Delete Network",
        format_serror(
          &e.context(format!(
            "Failed to delete network {}",
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
  #[instrument(
    "PruneNetworks",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::docker::PruneNetworks {})
      .await
      .context(format!(
        "Failed to prune networks on server {}",
        server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "Prune Networks",
        format_serror(&e.context("Failed to prune networks").into()),
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
  #[instrument(
    "DeleteImage",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      image = self.name,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::docker::DeleteImage {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "Failed to delete image {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "delete image",
        format_serror(
          &e.context(format!("Failed to delete image {}", self.name))
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
  #[instrument(
    "PruneImages",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log =
      match periphery.request(api::docker::PruneImages {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "Prune Images",
          format!(
            "Failed to prune images on server {} | {e:#?}",
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
  #[instrument(
    "DeleteVolume",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
      volume = self.name,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let periphery = periphery_client(&server).await?;

    let log = match periphery
      .request(api::docker::DeleteVolume {
        name: self.name.clone(),
      })
      .await
      .context(format!(
        "Failed to delete volume {} on server {}",
        self.name, server.name
      )) {
      Ok(log) => log,
      Err(e) => Log::error(
        "delete volume",
        format_serror(
          &e.context(format!(
            "Failed to delete volume {}",
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
  #[instrument(
    "PruneVolumes",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log =
      match periphery.request(api::docker::PruneVolumes {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "Prune Volumes",
          format!(
            "Failed to prune volumes on server {} | {e:#?}",
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
  #[instrument(
    "PruneDockerBuilders",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log =
      match periphery.request(api::build::PruneBuilders {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "Prune Builders",
          format!(
            "Failed to docker builder prune on server {} | {e:#?}",
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
  #[instrument(
    "PruneBuildx",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log =
      match periphery.request(api::build::PruneBuildx {}).await {
        Ok(log) => log,
        Err(e) => Log::error(
          "Prune Buildx",
          format!(
            "Failed to docker buildx prune on server {} | {e:#?}",
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
  #[instrument(
    "PruneSystem",
    skip_all,
    fields(
      id = id.to_string(),
      operator = user.id,
      update_id = update.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update, id }: &ExecuteArgs,
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

    let periphery = periphery_client(&server).await?;

    let log = match periphery.request(api::PruneSystem {}).await {
      Ok(log) => log,
      Err(e) => Log::error(
        "Prune System",
        format!(
          "Failed to docker system prune on server {} | {e:#?}",
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
