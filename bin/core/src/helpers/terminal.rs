use anyhow::{Context as _, anyhow};
use komodo_client::{
  api::{terminal::InitTerminal, write::CreateTerminal},
  entities::{
    deployment::Deployment,
    permission::PermissionLevel,
    server::Server,
    stack::Stack,
    terminal::{ContainerTerminalMode, Terminal, TerminalTarget},
    user::User,
  },
};
use periphery_client::api;

use crate::{
  helpers::periphery_client, periphery::PeripheryClient,
  permission::get_check_permissions, resource,
  state::stack_status_cache,
};

pub async fn setup_target_for_user(
  target: TerminalTarget,
  terminal: Option<String>,
  init: Option<InitTerminal>,
  user: &User,
) -> anyhow::Result<(TerminalTarget, String, PeripheryClient)> {
  match target {
    TerminalTarget::Server { server } => {
      setup_server_target_for_user(
        server.context("Missing 'target.params.server'")?,
        terminal,
        init,
        user,
      )
      .await
    }
    TerminalTarget::Container { server, container } => {
      setup_container_target_for_user(
        server, container, terminal, init, user,
      )
      .await
    }
    TerminalTarget::Stack { stack, service } => {
      setup_stack_service_target_for_user(
        stack,
        service.context("Missing 'target.params.service'")?,
        terminal,
        init,
        user,
      )
      .await
    }
    TerminalTarget::Deployment { deployment } => {
      setup_deployment_target_for_user(
        deployment, terminal, init, user,
      )
      .await
    }
  }
}

async fn setup_server_target_for_user(
  server: String,
  terminal: Option<String>,
  init: Option<InitTerminal>,
  user: &User,
) -> anyhow::Result<(TerminalTarget, String, PeripheryClient)> {
  let server = get_check_permissions::<Server>(
    &server,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;

  let terminal = terminal.unwrap_or_else(|| {
    init
      .as_ref()
      .and_then(|init| init.command.clone())
      .unwrap_or_else(|| String::from("term"))
  });

  let periphery = periphery_client(&server).await?;

  if let Some(init) = init {
    periphery
      .request(api::terminal::CreateServerTerminal {
        name: Some(terminal.clone()),
        command: init.command,
        recreate: init.recreate,
      })
      .await
      .context("Failed to create Server Terminal on Periphery")?;
  }

  Ok((
    TerminalTarget::Server {
      server: Some(server.id),
    },
    terminal,
    periphery,
  ))
}

async fn setup_container_target_for_user(
  server: String,
  container: String,
  terminal: Option<String>,
  init: Option<InitTerminal>,
  user: &User,
) -> anyhow::Result<(TerminalTarget, String, PeripheryClient)> {
  let server = get_check_permissions::<Server>(
    &server,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;

  let terminal = default_container_terminal_name(
    terminal,
    &container,
    init.as_ref(),
  );

  let periphery = periphery_client(&server).await?;

  let target = TerminalTarget::Container {
    server: server.id,
    container: container.clone(),
  };

  if let Some(init) = init {
    create_container_terminal_inner(
      CreateTerminal {
        name: Some(terminal.clone()),
        target: target.clone(),
        command: init.command,
        mode: init.mode,
        recreate: init.recreate,
      },
      &periphery,
      container,
    )
    .await?;
  }

  Ok((target, terminal, periphery))
}

async fn setup_stack_service_target_for_user(
  stack: String,
  service: String,
  terminal: Option<String>,
  init: Option<InitTerminal>,
  user: &User,
) -> anyhow::Result<(TerminalTarget, String, PeripheryClient)> {
  let (target, periphery, container) =
    get_stack_service_periphery_container(&stack, &service, user)
      .await?;

  let terminal = default_container_terminal_name(
    terminal,
    &container,
    init.as_ref(),
  );

  if let Some(init) = init {
    create_container_terminal_inner(
      CreateTerminal {
        name: Some(terminal.clone()),
        target: target.clone(),
        command: init.command,
        mode: init.mode,
        recreate: init.recreate,
      },
      &periphery,
      container,
    )
    .await?;
  }

  Ok((target, terminal, periphery))
}

async fn setup_deployment_target_for_user(
  deployment: String,
  terminal: Option<String>,
  init: Option<InitTerminal>,
  user: &User,
) -> anyhow::Result<(TerminalTarget, String, PeripheryClient)> {
  let (target, periphery, container) =
    get_deployment_periphery_container(&deployment, user).await?;

  let terminal = default_container_terminal_name(
    terminal,
    &container,
    init.as_ref(),
  );

  if let Some(init) = init {
    create_container_terminal_inner(
      CreateTerminal {
        name: Some(terminal.clone()),
        target: target.clone(),
        command: init.command,
        mode: init.mode,
        recreate: init.recreate,
      },
      &periphery,
      container,
    )
    .await?;
  }

  Ok((target, terminal, periphery))
}

fn default_container_terminal_name(
  terminal: Option<String>,
  container: &str,
  init: Option<&InitTerminal>,
) -> String {
  terminal.unwrap_or_else(|| {
    init
      .as_ref()
      .map(|init| {
        init.command.clone().unwrap_or_else(|| {
          init.mode.unwrap_or_default().as_ref().to_string()
        })
      })
      .unwrap_or_else(|| container.to_string())
  })
}

pub async fn create_container_terminal_inner(
  CreateTerminal {
    name,
    target,
    command,
    mode,
    recreate,
  }: CreateTerminal,
  periphery: &PeripheryClient,
  container: String,
) -> anyhow::Result<Terminal> {
  match mode.unwrap_or_default() {
    ContainerTerminalMode::Exec => periphery
      .request(periphery_client::api::terminal::CreateContainerExecTerminal {
        name,
        target,
        container,
        command,
        recreate,
      })
      .await
      .context(
        "Failed to create Container Exec Terminal on Periphery",
      ),
    ContainerTerminalMode::Attach => periphery
      .request(periphery_client::api::terminal::CreateContainerAttachTerminal {
        name,
        target,
        container,
        recreate,
      })
      .await
      .context(
        "Failed to create Container Attach Terminal on Periphery",
      ),
  }
}

pub async fn get_stack_service_periphery_container(
  stack: &str,
  service: &str,
  user: &User,
) -> anyhow::Result<(TerminalTarget, PeripheryClient, String)> {
  let stack = get_check_permissions::<Stack>(
    stack,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;

  let server =
    resource::get::<Server>(&stack.config.server_id).await?;

  let Some(status) = stack_status_cache().get(&stack.id).await else {
    return Err(anyhow!("Could not get Stack status"));
  };

  let container = status
    .curr
    .services
    .iter()
    .find(|s| s.service.as_str() == service)
    .with_context(|| {
      format!("Did not find Stack service matching {service}")
    })?
    .container
    .as_ref()
    .with_context(|| {
      format!("Did not find container for Stack service {service}")
    })?
    .name
    .clone();

  let periphery = periphery_client(&server).await?;

  Ok((
    TerminalTarget::Stack {
      stack: stack.id,
      service: Some(service.to_string()),
    },
    periphery,
    container,
  ))
}

pub async fn get_deployment_periphery_container(
  deployment: &str,
  user: &User,
) -> anyhow::Result<(TerminalTarget, PeripheryClient, String)> {
  let deployment = get_check_permissions::<Deployment>(
    deployment,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;

  let server =
    resource::get::<Server>(&deployment.config.server_id).await?;

  let periphery = periphery_client(&server).await?;

  let container = deployment.name.clone();

  Ok((
    TerminalTarget::Deployment {
      deployment: deployment.id,
    },
    periphery,
    container,
  ))
}
