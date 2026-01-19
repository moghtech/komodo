use anyhow::Context as _;
use futures_util::{StreamExt as _, stream::FuturesUnordered};
use komodo_client::{
  api::write::*,
  entities::{
    NoData, deployment::Deployment, permission::PermissionLevel,
    server::Server, stack::Stack, terminal::TerminalTarget,
    user::User,
  },
};
use mogh_resolver::Resolve;
use periphery_client::api;
use reqwest::StatusCode;
use mogh_error::AddStatusCode;

use crate::{
  helpers::{
    periphery_client,
    query::get_all_tags,
    terminal::{
      create_container_terminal_inner,
      get_deployment_periphery_container,
      get_stack_service_periphery_container,
    },
  },
  permission::get_check_permissions,
  resource,
};

use super::WriteArgs;

//

impl Resolve<WriteArgs> for CreateTerminal {
  #[instrument(
    "CreateTerminal",
    skip_all,
    fields(
      operator = user.id,
      terminal = self.name,
      target = format!("{:?}", self.target),
      command = self.command,
      mode = format!("{:?}", self.mode),
      recreate = format!("{:?}", self.recreate),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<NoData> {
    match self.target.clone() {
      TerminalTarget::Server { server } => {
        let server = server
          .context("Must provide 'target.params.server'")
          .status_code(StatusCode::BAD_REQUEST)?;
        create_server_terminal(self, server, user).await?;
      }
      TerminalTarget::Container { server, container } => {
        create_container_terminal(self, server, container, user)
          .await?;
      }
      TerminalTarget::Stack { stack, service } => {
        let service = service
          .context("Must provide 'target.params.service'")
          .status_code(StatusCode::BAD_REQUEST)?;
        create_stack_service_terminal(self, stack, service, user)
          .await?;
      }
      TerminalTarget::Deployment { deployment } => {
        create_deployment_terminal(self, deployment, user).await?;
      }
    };
    Ok(NoData {})
  }
}

async fn create_server_terminal(
  CreateTerminal {
    name,
    command,
    recreate,
    target: _,
    mode: _,
  }: CreateTerminal,
  server: String,
  user: &User,
) -> anyhow::Result<()> {
  let server = get_check_permissions::<Server>(
    &server,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;

  let periphery = periphery_client(&server).await?;

  periphery
    .request(api::terminal::CreateServerTerminal {
      name,
      command,
      recreate,
    })
    .await
    .context("Failed to create Server Terminal on Periphery")?;

  Ok(())
}

async fn create_container_terminal(
  req: CreateTerminal,
  server: String,
  container: String,
  user: &User,
) -> anyhow::Result<()> {
  let server = get_check_permissions::<Server>(
    &server,
    user,
    PermissionLevel::Read.terminal(),
  )
  .await?;
  let periphery = periphery_client(&server).await?;
  create_container_terminal_inner(req, &periphery, container).await
}

async fn create_stack_service_terminal(
  req: CreateTerminal,
  stack: String,
  service: String,
  user: &User,
) -> anyhow::Result<()> {
  let (_, periphery, container) =
    get_stack_service_periphery_container(&stack, &service, user)
      .await?;
  create_container_terminal_inner(req, &periphery, container).await
}

async fn create_deployment_terminal(
  req: CreateTerminal,
  deployment: String,
  user: &User,
) -> anyhow::Result<()> {
  let (_, periphery, container) =
    get_deployment_periphery_container(&deployment, user).await?;
  create_container_terminal_inner(req, &periphery, container).await
}

//

impl Resolve<WriteArgs> for DeleteTerminal {
  #[instrument(
    "DeleteTerminal",
    skip_all,
    fields(
      operator = user.id,
      target = format!("{:?}", self.target),
      terminal = self.terminal,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<NoData> {
    let server = match &self.target {
      TerminalTarget::Server { server } => {
        let server = server
          .as_ref()
          .context("Must provide 'target.params.server'")
          .status_code(StatusCode::BAD_REQUEST)?;
        get_check_permissions::<Server>(
          server,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?
      }
      TerminalTarget::Container { server, .. } => {
        get_check_permissions::<Server>(
          server,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?
      }
      TerminalTarget::Stack { stack, .. } => {
        let server = get_check_permissions::<Stack>(
          stack,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?
        .config
        .server_id;
        resource::get::<Server>(&server).await?
      }
      TerminalTarget::Deployment { deployment } => {
        let server = get_check_permissions::<Deployment>(
          deployment,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?
        .config
        .server_id;
        resource::get::<Server>(&server).await?
      }
    };

    let periphery = periphery_client(&server).await?;

    periphery
      .request(api::terminal::DeleteTerminal {
        target: self.target,
        terminal: self.terminal,
      })
      .await
      .context("Failed to delete terminal on Periphery")?;

    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for DeleteAllTerminals {
  #[instrument(
    "DeleteAllTerminals",
    skip_all,
    fields(
      operator = user.id,
      server = self.server,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<NoData> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.terminal(),
    )
    .await?;

    let periphery = periphery_client(&server).await?;

    periphery
      .request(api::terminal::DeleteAllTerminals {})
      .await
      .context("Failed to delete all terminals on Periphery")?;

    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for BatchDeleteAllTerminals {
  #[instrument(
    "BatchDeleteAllTerminals",
    skip_all,
    fields(
      operator = user.id,
      query = format!("{:?}", self.query),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };

    resource::list_full_for_user::<Server>(
      self.query,
      user,
      PermissionLevel::Read.terminal(),
      &all_tags,
    )
    .await?
    .into_iter()
    .map(|server| async move {
      let res = async {
        let periphery = periphery_client(&server).await?;

        periphery
          .request(api::terminal::DeleteAllTerminals {})
          .await
          .context("Failed to delete all terminals on Periphery")?;

        anyhow::Ok(())
      }
      .await;
      if let Err(e) = res {
        warn!(
          "Failed to delete all terminals on {} ({}) | {e:#}",
          server.name, server.id
        )
      }
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<_>>()
    .await;

    Ok(NoData {})
  }
}
