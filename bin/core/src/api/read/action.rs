use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    action::{
      Action, ActionActionState, ActionListItem, ActionSortBy,
      ActionState,
    },
    permission::PermissionLevel,
  },
};
use mogh_resolver::Resolve;

use crate::{
  helpers::query::{get_action_state, get_all_tags},
  permission::get_check_permissions,
  resource,
  state::{action_state_cache, action_states},
};

use super::{ReadArgs, list_limit};

impl Resolve<ReadArgs> for GetAction {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Action> {
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
  ) -> mogh_error::Result<Vec<ActionListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = list_limit(self.limit);
    let sort_by: resource::ListItemSort<ActionListItem> =
      match self.sort_by {
        ActionSortBy::Name => resource::ListItemSort::Name,
        ActionSortBy::State => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info
              .state
              .cmp(&b.info.state)
              .then_with(|| a.name.cmp(&b.name))
          }))
        }
        ActionSortBy::NextRun => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info
              .next_scheduled_run
              .cmp(&b.info.next_scheduled_run)
              .then_with(|| a.name.cmp(&b.name))
          }))
        }
      };
    let actions = resource::list_items_for_user::<Action>(
      self.query,
      resource::ListItemsQueryOptions {
        limit,
        page: self.page,
        sort_desc: self.sort_desc,
        sort_by,
      },
      user,
      PermissionLevel::Read.into(),
      &all_tags,
      |action| {
        states.is_empty() || states.contains(&action.info.state)
      },
    )
    .await?;
    Ok(actions)
  }
}

impl Resolve<ReadArgs> for ListFullActions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullActionsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = list_limit(self.limit);
    Ok(
      resource::list_full_for_user_filtered::<Action, _>(
        self.query,
        limit,
        self.page,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
        |action| {
          let states = states.clone();
          async move {
            if states.is_empty()
              || states.contains(&get_action_state(&action.id).await)
            {
              Some(action)
            } else {
              None
            }
          }
        },
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetActionActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ActionActionState> {
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
  ) -> mogh_error::Result<GetActionsSummaryResponse> {
    let actions = resource::list_full_for_user::<Action>(
      Default::default(),
      None,
      None,
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
