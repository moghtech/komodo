use std::cmp::Ordering;

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
    terminal::{Terminal, TerminalSortBy, TerminalTarget},
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

use super::{ReadArgs, list_limit};

//

impl Resolve<ReadArgs> for ListTerminals {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListTerminalsResponse> {
    let mut terminals = match self.target {
      None => {
        list_all_terminals_for_user(user, self.use_names).await?
      }
      Some(target) => match &target {
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
          list_terminals_on_server(&server, Some(target)).await?
        }
        TerminalTarget::Container { server, .. } => {
          let server = get_check_permissions::<Server>(
            server,
            user,
            PermissionLevel::Read.terminal(),
          )
          .await?;
          list_terminals_on_server(&server, Some(target)).await?
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
          list_terminals_on_server(&server, Some(target)).await?
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
          list_terminals_on_server(&server, Some(target)).await?
        }
      },
    };

    // The terminals come from Periphery agents rather than the db,
    // so the terms filter / sort / pagination are applied in memory.
    if !self.terms.is_empty() {
      let terms = self
        .terms
        .iter()
        .map(|term| term.to_lowercase())
        .collect::<Vec<_>>();
      terminals.retain(|terminal| {
        let name = terminal.name.to_lowercase();
        terms.iter().all(|term| name.contains(term))
      });
    }

    // All comparators fall back to name based sorting for equal
    // sort keys, inside `compare`, so descending sorts are fully
    // descending, matching the List<Resource> apis.
    let compare: fn(&Terminal, &Terminal) -> Ordering =
      match self.sort_by {
        TerminalSortBy::Name => |a, b| a.name.cmp(&b.name),
        TerminalSortBy::Target => |a, b| {
          a.target.cmp(&b.target).then_with(|| a.name.cmp(&b.name))
        },
        TerminalSortBy::Command => |a, b| {
          a.command.cmp(&b.command).then_with(|| a.name.cmp(&b.name))
        },
        TerminalSortBy::Size => |a, b| {
          a.stored_size_kb
            .total_cmp(&b.stored_size_kb)
            .then_with(|| a.name.cmp(&b.name))
        },
        TerminalSortBy::Created => |a, b| {
          a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.name.cmp(&b.name))
        },
      };
    if self.sort_desc {
      terminals.sort_by(|a, b| compare(b, a));
    } else {
      terminals.sort_by(compare);
    }

    let limit = list_limit(self.limit);
    let skip = limit.saturating_mul(self.page) as usize;
    let take = if limit == 0 {
      usize::MAX
    } else {
      limit as usize
    };
    Ok(terminals.into_iter().skip(skip).take(take).collect())
  }
}

async fn list_all_terminals_for_user(
  user: &User,
  use_names: bool,
) -> mogh_error::Result<Vec<Terminal>> {
  let (mut servers, stacks, deployments) = tokio::try_join!(
    resource::list_full_for_user::<Server>(
      Default::default(),
      None,
      None,
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
      None,
      None,
      user,
      PermissionLevel::Read.terminal(),
      &[]
    ),
    resource::list_full_for_user::<Deployment>(
      Default::default(),
      None,
      None,
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

  let terminals = servers
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
                  terminal.target_name = Some(server_name.clone());
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
                  terminal.target_name = Some(server_name.clone());
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
                  terminal.target_name = Some(s.name.clone());
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
                    terminal.target_name = Some(d.name.clone());
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
