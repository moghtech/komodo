use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    procedure::{Procedure, ProcedureState},
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::query::get_all_tags,
  permission::get_check_permissions,
  resource,
  state::{action_states, procedure_state_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetProcedure {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetProcedureResponse> {
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
  ) -> serror::Result<ListProceduresResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Procedure>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullProcedures {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullProceduresResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Procedure>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetProceduresSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetProceduresSummaryResponse> {
    let procedures = resource::list_full_for_user::<Procedure>(
      Default::default(),
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
  ) -> serror::Result<GetProcedureActionStateResponse> {
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
