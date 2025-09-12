use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    action::{
      Action, ActionActionState, ActionListItem, ActionState,
    },
    permission::PermissionLevel,
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::query::get_all_tags,
  permission::get_check_permissions,
  resource,
  state::{action_state_cache, action_states},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetAction {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Action> {
    Ok(
      get_check_permissions::<Action>(
        &self.action,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListActions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<ActionListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Action>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullActions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullActionsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Action>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetActionActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ActionActionState> {
    let action = get_check_permissions::<Action>(
      &self.action,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .action
      .get(&action.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetActionsSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetActionsSummaryResponse> {
    let actions = resource::list_full_for_user::<Action>(
      Default::default(),
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get actions from db")?;

    let mut res = GetActionsSummaryResponse::default();

    let cache = action_state_cache();
    let action_states = action_states();

    for action in actions {
      res.total += 1;

      match (
        cache.get(&action.id).await.unwrap_or_default(),
        action_states
          .action
          .get(&action.id)
          .await
          .unwrap_or_default()
          .get()?,
      ) {
        (_, action_states) if action_states.running > 0 => {
          res.running += action_states.running;
        }
        (ActionState::Ok, _) => res.ok += 1,
        (ActionState::Failed, _) => res.failed += 1,
        (ActionState::Unknown, _) => res.unknown += 1,
        // will never come off the cache in the running state, since that comes from action states
        (ActionState::Running, _) => unreachable!(),
      }
    }

    Ok(res)
  }
}
