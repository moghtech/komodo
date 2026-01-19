use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    repo::{Repo, RepoActionState, RepoListItem, RepoState},
  },
};
use mogh_resolver::Resolve;

use crate::{
  helpers::query::get_all_tags,
  permission::get_check_permissions,
  resource,
  state::{action_states, repo_state_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetRepo {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Repo> {
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
  ) -> mogh_error::Result<Vec<RepoListItem>> {
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
  ) -> mogh_error::Result<ListFullReposResponse> {
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
  ) -> mogh_error::Result<RepoActionState> {
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
  ) -> mogh_error::Result<GetReposSummaryResponse> {
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
        (RepoState::Unknown, _) => {
          if !repo.template {
            res.unknown += 1
          }
        }
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
