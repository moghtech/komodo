use std::{
  collections::HashMap,
  sync::{OnceLock, RwLock},
};

use anyhow::{Context, anyhow};
use async_timing_util::Timelength;
use chrono::Local;
use formatting::format_serror;
use komodo_client::{
  api::execute::{RunAction, RunProcedure},
  entities::{
    ResourceTarget, ResourceTargetVariant, ScheduleFormat,
    action::Action,
    alert::{Alert, AlertData, SeverityLevel},
    komodo_timestamp,
    procedure::Procedure,
    user::{action_user, procedure_user},
  },
};
use mungos::find::find_collect;
use resolver_api::Resolve;

use crate::{
  alert::send_alerts,
  api::execute::{ExecuteArgs, ExecuteRequest},
  helpers::update::init_execution_update,
  state::db_client,
};

pub fn spawn_schedule_executor() {
  // Executor thread
  tokio::spawn(async move {
    loop {
      let current_time = async_timing_util::wait_until_timelength(
        Timelength::OneSecond,
        0,
      )
      .await as i64;
      let mut lock = schedules().write().unwrap();
      let drained = lock.drain().collect::<Vec<_>>();
      for (target, next_run) in drained {
        match next_run {
          Ok(next_run_time) if current_time >= next_run_time => {
            tokio::spawn(async move {
              match &target {
                ResourceTarget::Action(id) => {
                  let action = match crate::resource::get::<Action>(
                    id,
                  )
                  .await
                  {
                    Ok(action) => action,
                    Err(e) => {
                      warn!(
                        "Scheduled action run on {id} failed | failed to get procedure | {e:?}"
                      );
                      return;
                    }
                  };
                  let request =
                    ExecuteRequest::RunAction(RunAction {
                      action: id.clone(),
                    });
                  let update = match init_execution_update(
                    &request,
                    action_user(),
                  )
                  .await
                  {
                    Ok(update) => update,
                    Err(e) => {
                      error!(
                        "Failed to make update for scheduled action run, action {id} is not being run | {e:#}"
                      );
                      return;
                    }
                  };
                  let ExecuteRequest::RunAction(request) = request
                  else {
                    unreachable!()
                  };
                  if let Err(e) = request
                    .resolve(&ExecuteArgs {
                      user: action_user().to_owned(),
                      update,
                    })
                    .await
                  {
                    warn!(
                      "Scheduled action run on {id} failed | {e:?}"
                    );
                  }
                  update_schedule(&action);
                  if action.config.schedule_alert {
                    let alert = Alert {
                      id: Default::default(),
                      target,
                      ts: komodo_timestamp(),
                      resolved_ts: Some(komodo_timestamp()),
                      resolved: true,
                      level: SeverityLevel::Ok,
                      data: AlertData::ScheduleRun {
                        resource_type: ResourceTargetVariant::Action,
                        id: action.id,
                        name: action.name,
                      },
                    };
                    send_alerts(&[alert]).await
                  }
                }
                ResourceTarget::Procedure(id) => {
                  let procedure = match crate::resource::get::<
                    Procedure,
                  >(id)
                  .await
                  {
                    Ok(procedure) => procedure,
                    Err(e) => {
                      warn!(
                        "Scheduled procedure run on {id} failed | failed to get procedure | {e:?}"
                      );
                      return;
                    }
                  };
                  let request =
                    ExecuteRequest::RunProcedure(RunProcedure {
                      procedure: id.clone(),
                    });
                  let update = match init_execution_update(
                    &request,
                    procedure_user(),
                  )
                  .await
                  {
                    Ok(update) => update,
                    Err(e) => {
                      error!(
                        "Failed to make update for scheduled procedure run, procedure {id} is not being run | {e:#}"
                      );
                      return;
                    }
                  };
                  let ExecuteRequest::RunProcedure(request) = request
                  else {
                    unreachable!()
                  };
                  if let Err(e) = request
                    .resolve(&ExecuteArgs {
                      user: procedure_user().to_owned(),
                      update,
                    })
                    .await
                  {
                    warn!(
                      "Scheduled procedure run on {id} failed | {e:?}"
                    );
                  }
                  update_schedule(&procedure);
                  if procedure.config.schedule_alert {
                    let alert = Alert {
                      id: Default::default(),
                      target,
                      ts: komodo_timestamp(),
                      resolved_ts: Some(komodo_timestamp()),
                      resolved: true,
                      level: SeverityLevel::Ok,
                      data: AlertData::ScheduleRun {
                        resource_type:
                          ResourceTargetVariant::Procedure,
                        id: procedure.id,
                        name: procedure.name,
                      },
                    };
                    send_alerts(&[alert]).await
                  }
                }
                _ => unreachable!(),
              }
            });
          }
          other => {
            lock.insert(target, other);
            continue;
          }
        };
      }
    }
  });
  // Updater thread
  tokio::spawn(async move {
    update_schedules().await;
    loop {
      async_timing_util::wait_until_timelength(
        Timelength::FiveMinutes,
        500,
      )
      .await;
      update_schedules().await
    }
  });
}

type UnixTimestampMs = i64;
type Schedules =
  HashMap<ResourceTarget, Result<UnixTimestampMs, String>>;

fn schedules() -> &'static RwLock<Schedules> {
  static SCHEDULES: OnceLock<RwLock<Schedules>> = OnceLock::new();
  SCHEDULES.get_or_init(Default::default)
}

pub fn get_schedule_item_info(
  target: &ResourceTarget,
) -> (Option<i64>, Option<String>) {
  match schedules().read().unwrap().get(target) {
    Some(Ok(time)) => (Some(*time), None),
    Some(Err(e)) => (None, Some(e.clone())),
    None => (None, None),
  }
}

pub fn cancel_schedule(target: &ResourceTarget) {
  schedules().write().unwrap().remove(target);
}

pub async fn update_schedules() {
  let (procedures, actions) = tokio::join!(
    find_collect(&db_client().procedures, None, None),
    find_collect(&db_client().actions, None, None),
  );
  let procedures = match procedures
    .context("failed to get all procedures from db")
  {
    Ok(procedures) => procedures,
    Err(e) => {
      error!("failed to get procedures for schedule update | {e:#}");
      Vec::new()
    }
  };
  let actions =
    match actions.context("failed to get all actions from db") {
      Ok(actions) => actions,
      Err(e) => {
        error!("failed to get actions for schedule update | {e:#}");
        Vec::new()
      }
    };
  // clear out any schedules which don't match to existing resources
  {
    let mut lock = schedules().write().unwrap();
    lock.retain(|target, _| match target {
      ResourceTarget::Action(id) => {
        actions.iter().any(|action| &action.id == id)
      }
      ResourceTarget::Procedure(id) => {
        procedures.iter().any(|procedure| &procedure.id == id)
      }
      _ => unreachable!(),
    });
  }
  for procedure in procedures {
    update_schedule(&procedure);
  }
  for action in actions {
    update_schedule(&action);
  }
}

/// Re/spawns the schedule for the given procedure
pub fn update_schedule(schedule: impl HasSchedule) {
  // Cancel any existing schedule for the procedure
  cancel_schedule(&schedule.target());

  if !schedule.enabled() || schedule.schedule().is_empty() {
    return;
  }

  schedules().write().unwrap().insert(
    schedule.target(),
    find_next_occurrence(schedule)
      .map_err(|e| format_serror(&e.into())),
  );
}

/// Finds the next run occurence in UTC ms.
fn find_next_occurrence(
  schedule: impl HasSchedule,
) -> anyhow::Result<i64> {
  let cron = match schedule.format() {
    ScheduleFormat::Cron => croner::Cron::new(schedule.schedule())
      .with_seconds_required()
      .with_dom_and_dow()
      .parse()
      .context("Failed to parse schedule CRON")?,
    ScheduleFormat::English => {
      let cron =
        english_to_cron::str_cron_syntax(&schedule.schedule())
          .map_err(|e| {
            anyhow!("Failed to parse english to cron | {e:?}")
          })?
          .split(' ')
          // croner does not accept year
          .take(6)
          .collect::<Vec<_>>()
          .join(" ");
      croner::Cron::new(&cron)
        .with_seconds_required()
        .with_dom_and_dow()
        .parse()
        .with_context(|| {
          format!("Failed to parse schedule CRON: {cron}")
        })?
    }
  };
  let next = if schedule.timezone().is_empty() {
    let tz_time = chrono::Local::now().with_timezone(&Local);
    cron
      .find_next_occurrence(&tz_time, false)
      .context("Failed to find next run time")?
      .timestamp_millis()
  } else {
    let tz: chrono_tz::Tz = schedule
      .timezone()
      .parse()
      .context("Failed to parse schedule timezone")?;
    let tz_time = chrono::Local::now().with_timezone(&tz);
    cron
      .find_next_occurrence(&tz_time, false)
      .context("Failed to find next run time")?
      .timestamp_millis()
  };
  Ok(next)
}

pub trait HasSchedule {
  fn target(&self) -> ResourceTarget;
  fn enabled(&self) -> bool;
  fn format(&self) -> ScheduleFormat;
  fn schedule(&self) -> &str;
  fn timezone(&self) -> &str;
}

impl HasSchedule for &Procedure {
  fn target(&self) -> ResourceTarget {
    ResourceTarget::Procedure(self.id.clone())
  }
  fn enabled(&self) -> bool {
    self.config.schedule_enabled
  }
  fn format(&self) -> ScheduleFormat {
    self.config.schedule_format
  }
  fn schedule(&self) -> &str {
    &self.config.schedule
  }
  fn timezone(&self) -> &str {
    &self.config.schedule_timezone
  }
}

impl HasSchedule for &Action {
  fn target(&self) -> ResourceTarget {
    ResourceTarget::Action(self.id.clone())
  }
  fn enabled(&self) -> bool {
    self.config.schedule_enabled
  }
  fn format(&self) -> ScheduleFormat {
    self.config.schedule_format
  }
  fn schedule(&self) -> &str {
    &self.config.schedule
  }
  fn timezone(&self) -> &str {
    &self.config.schedule_timezone
  }
}
