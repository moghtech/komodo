use std::collections::HashSet;

use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::*,
  entities::{
    SwarmOrServer,
    docker::{
      container::Container, service::SwarmService, stack::SwarmStack,
    },
    permission::PermissionLevel,
    stack::{
      Stack, StackActionState, StackListItem, StackQuery,
      StackService, StackState,
    },
  },
};
use mogh_error::AddStatusCodeError as _;
use mogh_resolver::Resolve;
use periphery_client::api::{
  compose::{GetComposeLog, GetComposeLogSearch},
  container::InspectContainer,
};
use reqwest::StatusCode;

use crate::{
  helpers::{
    periphery_client, query::get_all_tags, swarm::swarm_request,
  },
  permission::get_check_permissions,
  resource,
  stack::setup_stack_execution,
  state::{action_states, stack_status_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetStack {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Stack> {
    Ok(
      get_check_permissions::<Stack>(
        &self.stack,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListStackServices {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListStackServicesResponse> {
    let stack = get_check_permissions::<Stack>(
      &self.stack,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    let services = stack_status_cache()
      .get(&stack.id)
      .await
      .unwrap_or_default()
      .curr
      .services
      .clone();

    Ok(services)
  }
}

impl Resolve<ReadArgs> for ListAllStackServices {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListStackServicesResponse> {
    let stacks = resource::list_for_user::<Stack>(
      StackQuery::builder()
        .names(self.stacks.clone())
        .tags(self.tags)
        .build(),
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?;

    let mut services = Vec::<StackService>::new();
    let mut skipped = 0;
    let limit = self.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    let limit_usize = limit as usize;

    for stack in stacks {
      let cache =
        stack_status_cache().get_or_insert_default(&stack.id).await;
      let more = cache.curr.services
        .iter()
        .filter(|service| {
          // Apply state filter if defined.
          (self.state.is_empty() || self.state.contains(&service.state)) &&
          // Apply terms filter if defined
          (self.terms.is_empty()
            // Match when all terms contained within a name.
            || self.terms.iter().all(|term| service.service.contains(term)))
        });
      for service in more {
        if skipped < limit * self.page {
          // Eg. page 1 skips until after 100 services, page 2 after 200.
          skipped += 1;
        } else {
          // push and maybe early return
          services.push(service.clone());
          if limit > 0 && services.len() >= limit_usize {
            return Ok(services);
          }
        }
      }
    }

    Ok(services)
  }
}

impl Resolve<ReadArgs> for GetStackLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetStackLogResponse> {
    let GetStackLog {
      stack,
      mut services,
      tail,
      timestamps,
    } = self;
    let (stack, swarm_or_server) = setup_stack_execution(
      &stack,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;

    swarm_or_server.verify_has_target()?;

    let log = match swarm_or_server {
      SwarmOrServer::None => unreachable!(),
      SwarmOrServer::Swarm(swarm) => {
        let service = services.pop().context(
          "Must pass single service for Swarm mode Stack logs",
        )?;
        swarm_request(
          &swarm.config.server_ids,
          periphery_client::api::swarm::GetSwarmServiceLog {
            // The actual service name on swarm will be stackname_servicename
            service: format!(
              "{}_{service}",
              stack.project_name(false)
            ),
            tail,
            timestamps,
            no_task_ids: false,
            no_resolve: false,
            details: false,
          },
        )
        .await
        .context("Failed to get stack service log from swarm")?
      }
      SwarmOrServer::Server(server) => periphery_client(&server)
        .await?
        .request(GetComposeLog {
          project: stack.project_name(false),
          services,
          tail,
          timestamps,
        })
        .await
        .context("Failed to get stack log from periphery")?,
    };

    Ok(log)
  }
}

impl Resolve<ReadArgs> for SearchStackLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<SearchStackLogResponse> {
    let SearchStackLog {
      stack,
      mut services,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let (stack, swarm_or_server) = setup_stack_execution(
      &stack,
      user,
      PermissionLevel::Read.logs(),
    )
    .await?;

    swarm_or_server.verify_has_target()?;

    let log = match swarm_or_server {
      SwarmOrServer::None => unreachable!(),
      SwarmOrServer::Swarm(swarm) => {
        let service = services.pop().context(
          "Must pass single service for Swarm mode Stack logs",
        )?;
        swarm_request(
          &swarm.config.server_ids,
          periphery_client::api::swarm::GetSwarmServiceLogSearch {
            service,
            terms,
            combinator,
            invert,
            timestamps,
            no_task_ids: false,
            no_resolve: false,
            details: false,
          },
        )
        .await
        .context("Failed to get stack service log from swarm")?
      }
      SwarmOrServer::Server(server) => periphery_client(&server)
        .await?
        .request(GetComposeLogSearch {
          project: stack.project_name(false),
          services,
          terms,
          combinator,
          invert,
          timestamps,
        })
        .await
        .context("Failed to search stack log from periphery")?,
    };

    Ok(log)
  }
}

impl Resolve<ReadArgs> for InspectStackContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Container> {
    let InspectStackContainer { stack, service } = self;
    let (stack, swarm_or_server) = setup_stack_execution(
      &stack,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;

    let SwarmOrServer::Server(server) = swarm_or_server else {
      return Err(
        anyhow!(
          "InspectStackContainer should not be called for Stack in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

    let services = &stack_status_cache()
      .get(&stack.id)
      .await
      .unwrap_or_default()
      .curr
      .services;

    let Some(name) = services
      .iter()
      .find(|s| s.service == service)
      .and_then(|s| s.container.as_ref().map(|c| c.name.clone()))
    else {
      return Err(anyhow!(
        "No service found matching '{service}'. Was the stack last deployed manually?"
      ).into());
    };

    let res = periphery_client(&server)
      .await?
      .request(InspectContainer { name })
      .await
      .context("Failed to inspect container on server")?;

    Ok(res)
  }
}

impl Resolve<ReadArgs> for InspectStackSwarmService {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<SwarmService> {
    let InspectStackSwarmService { stack, service } = self;
    let (stack, swarm_or_server) = setup_stack_execution(
      &stack,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;

    let SwarmOrServer::Swarm(swarm) = swarm_or_server else {
      return Err(
        anyhow!(
          "InspectStackSwarmService should only be called for Stack in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

    let services = &stack_status_cache()
      .get(&stack.id)
      .await
      .unwrap_or_default()
      .curr
      .services;

    let Some(service) = services
      .iter()
      .find(|s| s.service == service)
      .and_then(|s| {
        s.swarm_service.as_ref().and_then(|c| c.name.clone())
      })
    else {
      return Err(anyhow!(
        "No service found matching '{service}'. Was the stack last deployed manually?"
      ).into());
    };

    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmService { service },
    )
    .await
    .context("Failed to inspect service on swarm")
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for InspectStackSwarmInfo {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<SwarmStack> {
    let (stack, swarm_or_server) = setup_stack_execution(
      &self.stack,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;

    let SwarmOrServer::Swarm(swarm) = swarm_or_server else {
      return Err(
        anyhow!(
          "InspectStackSwarmInfo should only be called for Stack in Swarm Mode"
        )
        .status_code(StatusCode::BAD_REQUEST),
      );
    };

    swarm_request(
      &swarm.config.server_ids,
      periphery_client::api::swarm::InspectSwarmStack {
        stack: stack.project_name(false),
      },
    )
    .await
    .context("Failed to inspect stack info on swarm")
    .map_err(Into::into)
  }
}

impl Resolve<ReadArgs> for ListCommonStackExtraArgs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListCommonStackExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let stacks = resource::list_full_for_user::<Stack>(
      self.query,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await
    .context("Failed to get resources matching query")?;

    // first collect with guaranteed uniqueness
    let mut res = HashSet::<String>::new();

    for stack in stacks {
      for extra_arg in stack.config.extra_args {
        res.insert(extra_arg);
      }
    }

    let mut res = res.into_iter().collect::<Vec<_>>();
    res.sort();
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListCommonStackBuildExtraArgs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListCommonStackBuildExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let stacks = resource::list_full_for_user::<Stack>(
      self.query,
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await
    .context("Failed to get resources matching query")?;

    // first collect with guaranteed uniqueness
    let mut res = HashSet::<String>::new();

    for stack in stacks {
      for extra_arg in stack.config.build_extra_args {
        res.insert(extra_arg);
      }
    }

    let mut res = res.into_iter().collect::<Vec<_>>();
    res.sort();
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListStacks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<StackListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let only_update_available = self.query.specific.update_available;
    let states = self.query.specific.states.clone();
    let limit = self.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    let stacks = resource::list_for_user::<Stack>(
      self.query,
      limit as i64,
      self.page * limit,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await?;
    let stacks = if only_update_available {
      stacks
        .into_iter()
        .filter(|stack| {
          stack
            .info
            .services
            .iter()
            .any(|service| service.update_available)
        })
        .collect()
    } else {
      stacks
    };
    // The state is not stored on the database,
    // it can only be filtered after they are attached to list items.
    let stacks = if states.is_empty() {
      stacks
    } else {
      stacks
        .into_iter()
        .filter(|stack| states.contains(&stack.info.state))
        .collect()
    };
    Ok(stacks)
  }
}

impl Resolve<ReadArgs> for ListFullStacks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullStacksResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let limit = self.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    Ok(
      resource::list_full_for_user::<Stack>(
        self.query,
        limit as i64,
        self.page * limit,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetStackActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<StackActionState> {
    let stack = get_check_permissions::<Stack>(
      &self.stack,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .stack
      .get(&stack.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetStacksSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetStacksSummaryResponse> {
    let stacks = resource::list_full_for_user::<Stack>(
      Default::default(),
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("Failed to get stacks from database")?;

    let mut res = GetStacksSummaryResponse::default();

    let cache = stack_status_cache();

    for stack in stacks {
      res.total += 1;
      match cache.get(&stack.id).await.unwrap_or_default().curr.state
      {
        StackState::Running => res.running += 1,
        StackState::Stopped | StackState::Paused => res.stopped += 1,
        StackState::Down => res.down += 1,
        StackState::Unknown => {
          if !stack.template {
            res.unknown += 1
          }
        }
        _ => res.unhealthy += 1,
      }
    }

    Ok(res)
  }
}
