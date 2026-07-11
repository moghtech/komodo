use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    procedure::{
      Procedure, ProcedureListItem, ProcedureSortBy, ProcedureState,
    },
  },
};
use mogh_resolver::Resolve;

use crate::{
  helpers::query::{get_all_tags, get_procedure_state},
  permission::get_check_permissions,
  resource,
  state::{action_states, procedure_state_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetProcedure {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetProcedureResponse> {
    Ok(
      get_check_permissions::<Procedure>(
        &self.procedure,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListProcedures {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListProceduresResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = self.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    let sort_by: resource::ListItemSort<ProcedureListItem> =
      match self.sort_by {
        ProcedureSortBy::Name => resource::ListItemSort::Name,
        ProcedureSortBy::State => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info.state.cmp(&b.info.state)
          }))
        }
        ProcedureSortBy::NextRun => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info.next_scheduled_run.cmp(&b.info.next_scheduled_run)
          }))
        }
      };
    let procedures = resource::list_items_for_user::<Procedure>(
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
      |procedure| {
        states.is_empty() || states.contains(&procedure.info.state)
      },
    )
    .await?;
    Ok(procedures)
  }
}

impl Resolve<ReadArgs> for ListFullProcedures {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullProceduresResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = self.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    Ok(
      resource::list_full_for_user_filtered::<Procedure, _>(
        self.query,
        limit,
        self.page,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
        |procedure| {
          let states = states.clone();
          async move {
            if states.is_empty()
              || states
                .contains(&get_procedure_state(&procedure.id).await)
            {
              Some(procedure)
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

impl Resolve<ReadArgs> for GetProceduresSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetProceduresSummaryResponse> {
    let procedures = resource::list_full_for_user::<Procedure>(
      Default::default(),
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await
    .context("failed to get procedures from db")?;

    let mut res = GetProceduresSummaryResponse::default();

    let cache = procedure_state_cache();
    let action_states = action_states();

    for procedure in procedures {
      res.total += 1;

      match (
        cache.get(&procedure.id).await.unwrap_or_default(),
        action_states
          .procedure
          .get(&procedure.id)
          .await
          .unwrap_or_default()
          .get()?,
      ) {
        (_, action_states) if action_states.running => {
          res.running += 1;
        }
        (ProcedureState::Ok, _) => res.ok += 1,
        (ProcedureState::Failed, _) => res.failed += 1,
        (ProcedureState::Unknown, _) => res.unknown += 1,
        // will never come off the cache in the running state, since that comes from action states
        (ProcedureState::Running, _) => unreachable!(),
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetProcedureActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetProcedureActionStateResponse> {
    let procedure = get_check_permissions::<Procedure>(
      &self.procedure,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .procedure
      .get(&procedure.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}
