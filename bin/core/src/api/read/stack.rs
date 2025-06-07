use std::collections::HashSet;

use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::*,
  entities::{
    config::core::CoreConfig,
    docker::container::Container,
    permission::PermissionLevel,
    server::{Server, ServerState},
    stack::{Stack, StackActionState, StackListItem, StackState},
  },
};
use periphery_client::api::{
  compose::{GetComposeLog, GetComposeLogSearch},
  container::InspectContainer,
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::{periphery_client, query::get_all_tags},
  permission::get_check_permissions,
  resource,
  stack::get_stack_and_server,
  state::{
    action_states, github_client, server_status_cache,
    stack_status_cache,
  },
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetStack {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Stack> {
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
  ) -> serror::Result<ListStackServicesResponse> {
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

impl Resolve<ReadArgs> for GetStackLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetStackLogResponse> {
    let GetStackLog {
      stack,
      services,
      tail,
      timestamps,
    } = self;
    let (stack, server) = get_stack_and_server(
      &stack,
      user,
      PermissionLevel::Read.logs(),
      true,
    )
    .await?;
    let res = periphery_client(&server)?
      .request(GetComposeLog {
        project: stack.project_name(false),
        services,
        tail,
        timestamps,
      })
      .await
      .context("Failed to get stack log from periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for SearchStackLog {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<SearchStackLogResponse> {
    let SearchStackLog {
      stack,
      services,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let (stack, server) = get_stack_and_server(
      &stack,
      user,
      PermissionLevel::Read.logs(),
      true,
    )
    .await?;
    let res = periphery_client(&server)?
      .request(GetComposeLogSearch {
        project: stack.project_name(false),
        services,
        terms,
        combinator,
        invert,
        timestamps,
      })
      .await
      .context("Failed to search stack log from periphery")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for InspectStackContainer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Container> {
    let InspectStackContainer { stack, service } = self;
    let stack = get_check_permissions::<Stack>(
      &stack,
      user,
      PermissionLevel::Read.inspect(),
    )
    .await?;
    if stack.config.server_id.is_empty() {
      return Err(
        anyhow!("Cannot inspect stack, not attached to any server")
          .into(),
      );
    }
    let server =
      resource::get::<Server>(&stack.config.server_id).await?;
    let cache = server_status_cache()
      .get_or_insert_default(&server.id)
      .await;
    if cache.state != ServerState::Ok {
      return Err(
        anyhow!(
          "Cannot inspect container: server is {:?}",
          cache.state
        )
        .into(),
      );
    }
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
    let res = periphery_client(&server)?
      .request(InspectContainer { name })
      .await?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for ListCommonStackExtraArgs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListCommonStackExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let stacks = resource::list_full_for_user::<Stack>(
      self.query,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await
    .context("failed to get resources matching query")?;

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
  ) -> serror::Result<ListCommonStackBuildExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let stacks = resource::list_full_for_user::<Stack>(
      self.query,
      user,
      PermissionLevel::Read.into(),
      &all_tags,
    )
    .await
    .context("failed to get resources matching query")?;

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
  ) -> serror::Result<Vec<StackListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let only_update_available = self.query.specific.update_available;
    let stacks = resource::list_for_user::<Stack>(
      self.query,
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
    Ok(stacks)
  }
}

impl Resolve<ReadArgs> for ListFullStacks {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullStacksResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Stack>(
        self.query,
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
  ) -> serror::Result<StackActionState> {
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
  ) -> serror::Result<GetStacksSummaryResponse> {
    let stacks = resource::list_full_for_user::<Stack>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get stacks from db")?;

    let mut res = GetStacksSummaryResponse::default();

    let cache = stack_status_cache();

    for stack in stacks {
      res.total += 1;
      match cache.get(&stack.id).await.unwrap_or_default().curr.state
      {
        StackState::Running => res.running += 1,
        StackState::Stopped | StackState::Paused => res.stopped += 1,
        StackState::Down => res.down += 1,
        StackState::Unknown => res.unknown += 1,
        _ => res.unhealthy += 1,
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetStackWebhooksEnabled {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetStackWebhooksEnabledResponse> {
    let Some(github) = github_client() else {
      return Ok(GetStackWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        deploy_enabled: false,
      });
    };

    let stack = get_check_permissions::<Stack>(
      &self.stack,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    if stack.config.git_provider != "github.com"
      || stack.config.repo.is_empty()
    {
      return Ok(GetStackWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        deploy_enabled: false,
      });
    }

    let mut split = stack.config.repo.split('/');
    let owner = split.next().context("Sync repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Ok(GetStackWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        deploy_enabled: false,
      });
    };

    let repo_name =
      split.next().context("Repo repo has no repo after the /")?;

    let github_repos = github.repos();

    let webhooks = github_repos
      .list_all_webhooks(owner, repo_name)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      ..
    } = core_config();

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let refresh_url =
      format!("{host}/listener/github/stack/{}/refresh", stack.id);
    let deploy_url =
      format!("{host}/listener/github/stack/{}/deploy", stack.id);

    let mut refresh_enabled = false;
    let mut deploy_enabled = false;

    for webhook in webhooks {
      if webhook.active && webhook.config.url == refresh_url {
        refresh_enabled = true
      }
      if webhook.active && webhook.config.url == deploy_url {
        deploy_enabled = true
      }
    }

    Ok(GetStackWebhooksEnabledResponse {
      managed: true,
      refresh_enabled,
      deploy_enabled,
    })
  }
}
