use anyhow::Context as _;
use futures_util::{
  FutureExt, StreamExt as _, stream::FuturesUnordered,
};
use komodo_client::{
  api::read::{ListTerminals, ListTerminalsResponse},
  entities::{
    deployment::Deployment,
    permission::PermissionLevel,
    server::Server,
    stack::Stack,
    terminal::{Terminal, TerminalTarget},
    user::User,
  },
};
use mogh_error::AddStatusCode;
use mogh_resolver::Resolve;
use reqwest::StatusCode;

use crate::{
  helpers::periphery_client, permission::get_check_permissions,
  resource,
};

use super::ReadArgs;

//

impl Resolve<ReadArgs> for ListTerminals {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListTerminalsResponse> {
    let Some(target) = self.target else {
      return list_all_terminals_for_user(user, self.use_names).await;
    };
    match &target {
      TerminalTarget::Server { server } => {
        let server = server
          .as_ref()
          .context("Must provide 'target.params.server'")
          .status_code(StatusCode::BAD_REQUEST)?;
        let server = get_check_permissions::<Server>(
          server,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?;
        list_terminals_on_server(&server, Some(target)).await
      }
      TerminalTarget::Container { server, .. } => {
        let server = get_check_permissions::<Server>(
          server,
          user,
          PermissionLevel::Read.terminal(),
        )
        .await?;
        list_terminals_on_server(&server, Some(target)).await
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
        let server = resource::get::<Server>(&server).await?;
        list_terminals_on_server(&server, Some(target)).await
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
        let server = resource::get::<Server>(&server).await?;
        list_terminals_on_server(&server, Some(target)).await
      }
    }
  }
}

async fn list_all_terminals_for_user(
  user: &User,
  use_names: bool,
) -> mogh_error::Result<Vec<Terminal>> {
  let (mut servers, stacks, deployments) = tokio::try_join!(
    resource::list_full_for_user::<Server>(
      Default::default(),
      user,
      PermissionLevel::Read.terminal(),
      &[]
    )
    .map(|res| res.map(|servers| servers
      .into_iter()
      // true denotes user actually has permission on this Server.
      .map(|server| (server, true))
      .collect::<Vec<_>>())),
    resource::list_full_for_user::<Stack>(
      Default::default(),
      user,
      PermissionLevel::Read.terminal(),
      &[]
    ),
    resource::list_full_for_user::<Deployment>(
      Default::default(),
      user,
      PermissionLevel::Read.terminal(),
      &[]
    ),
  )?;

  // Ensure any missing servers are present to query
  for stack in &stacks {
    if !stack.config.server_id.is_empty()
      && !servers
        .iter()
        .any(|(server, _)| server.id == stack.config.server_id)
    {
      let server =
        resource::get::<Server>(&stack.config.server_id).await?;
      servers.push((server, false));
    }
  }
  for deployment in &deployments {
    if !deployment.config.server_id.is_empty()
      && !servers
        .iter()
        .any(|(server, _)| server.id == deployment.config.server_id)
    {
      let server =
        resource::get::<Server>(&deployment.config.server_id).await?;
      servers.push((server, false));
    }
  }

  let mut terminals = servers
    .into_iter()
    .map(|(server, server_permission)| async move {
      (
        list_terminals_on_server(&server, None).await,
        (server.id, server.name, server_permission),
      )
    })
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .flat_map(
      |(terminals, (server_id, server_name, server_permission))| {
        let terminals = terminals
          .ok()?
          .into_iter()
          .filter_map(|mut terminal| {
            // Only keep terminals with appropriate perms.
            match terminal.target.clone() {
              TerminalTarget::Server { .. } => server_permission
                .then(|| {
                  terminal.target = TerminalTarget::Server {
                    server: Some(if use_names {
                      server_name.clone()
                    } else {
                      server_id.clone()
                    }),
                  };
                  terminal
                }),
              TerminalTarget::Container { container, .. } => {
                server_permission.then(|| {
                  terminal.target = TerminalTarget::Container {
                    server: if use_names {
                      server_name.clone()
                    } else {
                      server_id.clone()
                    },
                    container,
                  };
                  terminal
                })
              }
              TerminalTarget::Stack { stack, service } => {
                stacks.iter().find(|s| s.id == stack).map(|s| {
                  terminal.target = TerminalTarget::Stack {
                    stack: if use_names {
                      s.name.clone()
                    } else {
                      s.id.clone()
                    },
                    service,
                  };
                  terminal
                })
              }
              TerminalTarget::Deployment { deployment } => {
                deployments.iter().find(|d| d.id == deployment).map(
                  |d| {
                    terminal.target = TerminalTarget::Deployment {
                      deployment: if use_names {
                        d.name.clone()
                      } else {
                        d.id.clone()
                      },
                    };
                    terminal
                  },
                )
              }
            }
          })
          .collect::<Vec<_>>();

        Some(terminals)
      },
    )
    .flatten()
    .collect::<Vec<_>>();

  terminals.sort_by(|a, b| {
    a.target.cmp(&b.target).then(a.name.cmp(&b.name))
  });

  Ok(terminals)
}

async fn list_terminals_on_server(
  server: &Server,
  target: Option<TerminalTarget>,
) -> mogh_error::Result<Vec<Terminal>> {
  periphery_client(server)
    .await?
    .request(periphery_client::api::terminal::ListTerminals {
      target,
    })
    .await
    .with_context(|| {
      format!(
        "Failed to get Terminal list from Server {} ({})",
        server.name, server.id
      )
    })
    .map_err(Into::into)
}
