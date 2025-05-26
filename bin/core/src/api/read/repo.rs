use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    config::core::CoreConfig,
    permission::PermissionLevel,
    repo::{Repo, RepoActionState, RepoListItem, RepoState},
  },
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::query::get_all_tags,
  permission::get_check_permissions,
  resource,
  state::{action_states, github_client, repo_state_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetRepo {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Repo> {
    Ok(
      get_check_permissions::<Repo>(
        &self.repo,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListRepos {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<RepoListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Repo>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullRepos {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullReposResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Repo>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetRepoActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<RepoActionState> {
    let repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .repo
      .get(&repo.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetReposSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetReposSummaryResponse> {
    let repos = resource::list_full_for_user::<Repo>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get repos from db")?;

    let mut res = GetReposSummaryResponse::default();

    let cache = repo_state_cache();
    let action_states = action_states();

    for repo in repos {
      res.total += 1;

      match (
        cache.get(&repo.id).await.unwrap_or_default(),
        action_states
          .repo
          .get(&repo.id)
          .await
          .unwrap_or_default()
          .get()?,
      ) {
        (_, action_states) if action_states.cloning => {
          res.cloning += 1;
        }
        (_, action_states) if action_states.pulling => {
          res.pulling += 1;
        }
        (_, action_states) if action_states.building => {
          res.building += 1;
        }
        (RepoState::Ok, _) => res.ok += 1,
        (RepoState::Failed, _) => res.failed += 1,
        (RepoState::Unknown, _) => res.unknown += 1,
        // will never come off the cache in the building state, since that comes from action states
        (RepoState::Cloning, _)
        | (RepoState::Pulling, _)
        | (RepoState::Building, _) => {
          unreachable!()
        }
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetRepoWebhooksEnabled {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetRepoWebhooksEnabledResponse> {
    let Some(github) = github_client() else {
      return Ok(GetRepoWebhooksEnabledResponse {
        managed: false,
        clone_enabled: false,
        pull_enabled: false,
        build_enabled: false,
      });
    };

    let repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;

    if repo.config.git_provider != "github.com"
      || repo.config.repo.is_empty()
    {
      return Ok(GetRepoWebhooksEnabledResponse {
        managed: false,
        clone_enabled: false,
        pull_enabled: false,
        build_enabled: false,
      });
    }

    let mut split = repo.config.repo.split('/');
    let owner = split.next().context("Repo repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Ok(GetRepoWebhooksEnabledResponse {
        managed: false,
        clone_enabled: false,
        pull_enabled: false,
        build_enabled: false,
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
    let clone_url =
      format!("{host}/listener/github/repo/{}/clone", repo.id);
    let pull_url =
      format!("{host}/listener/github/repo/{}/pull", repo.id);
    let build_url =
      format!("{host}/listener/github/repo/{}/build", repo.id);

    let mut clone_enabled = false;
    let mut pull_enabled = false;
    let mut build_enabled = false;

    for webhook in webhooks {
      if !webhook.active {
        continue;
      }
      if webhook.config.url == clone_url {
        clone_enabled = true
      }
      if webhook.config.url == pull_url {
        pull_enabled = true
      }
      if webhook.config.url == build_url {
        build_enabled = true
      }
    }

    Ok(GetRepoWebhooksEnabledResponse {
      managed: true,
      clone_enabled,
      pull_enabled,
      build_enabled,
    })
  }
}
