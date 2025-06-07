use futures::future::join_all;
use komodo_client::{
  api::read::*,
  entities::{
    ResourceTarget,
    action::{Action, ActionState},
    permission::PermissionLevel,
    procedure::{Procedure, ProcedureState},
    resource::ResourceQuery,
    schedule::Schedule,
  },
};
use resolver_api::Resolve;

use crate::{
  helpers::query::{
    get_action_state, get_all_tags, get_last_run_at,
    get_procedure_state,
  },
  resource::list_full_for_user,
  schedule::get_schedule_item_info,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for ListSchedules {
  async fn resolve(
    self,
    args: &ReadArgs,
  ) -> serror::Result<Vec<Schedule>> {
    let all_tags = get_all_tags(None).await?;
    let (actions, procedures) = tokio::try_join!(
      list_full_for_user::<Action>(
        ResourceQuery {
          names: Default::default(),
          tag_behavior: self.tag_behavior,
          tags: self.tags.clone(),
          specific: Default::default(),
        },
        &args.user,
        PermissionLevel::Read.into(),
        &all_tags,
      ),
      list_full_for_user::<Procedure>(
        ResourceQuery {
          names: Default::default(),
          tag_behavior: self.tag_behavior,
          tags: self.tags.clone(),
          specific: Default::default(),
        },
        &args.user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
    )?;
    let actions = actions.into_iter().map(async |action| {
      let (next_scheduled_run, schedule_error) =
        get_schedule_item_info(&ResourceTarget::Action(
          action.id.clone(),
        ));
      let (state, last_run_at) = tokio::join!(
        get_action_state(&action.id),
        get_last_run_at::<Action>(&action.id)
      );
      Schedule {
        target: ResourceTarget::Action(action.id),
        name: action.name,
        enabled: action.config.schedule_enabled,
        schedule_format: action.config.schedule_format,
        schedule: action.config.schedule,
        schedule_timezone: action.config.schedule_timezone,
        last_run_at: last_run_at.unwrap_or(None),
        last_run_success: matches!(state, ActionState::Ok),
        next_scheduled_run,
        schedule_error,
      }
    });
    let procedures = procedures.into_iter().map(async |procedure| {
      let (next_scheduled_run, schedule_error) =
        get_schedule_item_info(&ResourceTarget::Procedure(
          procedure.id.clone(),
        ));
      let (state, last_run_at) = tokio::join!(
        get_procedure_state(&procedure.id),
        get_last_run_at::<Procedure>(&procedure.id)
      );
      Schedule {
        target: ResourceTarget::Procedure(procedure.id),
        name: procedure.name,
        enabled: procedure.config.schedule_enabled,
        schedule_format: procedure.config.schedule_format,
        schedule: procedure.config.schedule,
        schedule_timezone: procedure.config.schedule_timezone,
        last_run_at: last_run_at.unwrap_or(None),
        last_run_success: matches!(state, ProcedureState::Ok),
        next_scheduled_run,
        schedule_error,
      }
    });
    let (actions, procedures) =
      tokio::join!(join_all(actions), join_all(procedures));

    Ok(
      actions
        .into_iter()
        .chain(procedures)
        .filter(|s| !s.schedule.is_empty())
        .collect(),
    )
  }
}
