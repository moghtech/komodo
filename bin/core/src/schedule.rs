use std::{
  collections::HashMap,
  sync::{OnceLock, RwLock},
};

use anyhow::{Context, anyhow};
use async_timing_util::Timelength;
use komodo_client::{
  api::execute::RunProcedure,
  entities::{
    procedure::{Procedure, ScheduleFormat},
    user::procedure_user,
  },
};
use mungos::find::find_collect;
use resolver_api::Resolve;

use crate::{
  api::execute::{ExecuteArgs, ExecuteRequest},
  helpers::update::init_execution_update,
  state::db_client,
};

pub fn spawn_schedule_management_threads() {
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
      for (procedure, next_run) in drained {
        match next_run {
          Ok(next_run_time) if current_time >= next_run_time => {
            tokio::spawn(async move {
              let request =
                ExecuteRequest::RunProcedure(RunProcedure {
                  procedure: procedure.clone(),
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
                    "Failed to make update for scheduled procedure run, procedure {procedure} is not being run | {e:#}"
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
                  "scheduled procedure run on {procedure} failed | {e:?}"
                );
              }
            });
          }
          other => {
            lock.insert(procedure, other);
            continue;
          }
        };
      }
    }
  });
  // Updater thread
  tokio::spawn(async move {
    update_procedure_schedules().await;
    loop {
      async_timing_util::wait_until_timelength(
        Timelength::OneMinute,
        500,
      )
      .await;
      update_procedure_schedules().await
    }
  });
}

type ProcedureId = String;
type UnixTimestampMs = i64;
type Schedules =
  HashMap<ProcedureId, Result<UnixTimestampMs, String>>;

fn schedules() -> &'static RwLock<Schedules> {
  static SCHEDULES: OnceLock<RwLock<Schedules>> = OnceLock::new();
  SCHEDULES.get_or_init(Default::default)
}

pub fn get_schedule_item_info(
  procedure_id: &str,
) -> (Option<i64>, Option<String>) {
  match schedules().read().unwrap().get(procedure_id) {
    Some(Ok(time)) => (Some(*time), None),
    Some(Err(e)) => (None, Some(e.clone())),
    None => (None, None),
  }
}

pub fn cancel_schedule(procedure_id: &str) {
  schedules().write().unwrap().remove(procedure_id);
}

pub async fn update_procedure_schedules() {
  let procedures =
    match find_collect(&db_client().procedures, None, None)
      .await
      .context("failed to get all procedures from db")
    {
      Ok(procedures) => procedures,
      Err(e) => {
        error!(
          "failed to get procedures for schedule update | {e:#}"
        );
        return;
      }
    };
  // clear out any schedules which don't match to existing procedures
  {
    let mut lock = schedules().write().unwrap();
    lock.retain(|procedure_id, _| {
      procedures
        .iter()
        .any(|procedure| &procedure.id == procedure_id)
    });
  }
  for procedure in procedures {
    update_procedure_scedule(&procedure);
  }
}

/// Re/spawns the schedule for the given procedure
pub fn update_procedure_scedule(procedure: &Procedure) {
  // Cancel any existing schedule for the procedure
  cancel_schedule(&procedure.id);

  if !procedure.config.schedule_enabled
    || procedure.config.schedule.is_empty()
  {
    return;
  }

  schedules().write().unwrap().insert(
    procedure.id.clone(),
    find_next_occurrence(procedure).map_err(|e| format!("{e:#?}")),
  );
}

/// Finds the next run occurence in UTC ms.
fn find_next_occurrence(
  procedure: &Procedure,
) -> anyhow::Result<i64> {
  let cron = match procedure.config.schedule_format {
    ScheduleFormat::Cron => {
      croner::Cron::new(&procedure.config.schedule)
        .parse()
        .context("Failed to parse schedule CRON")?
    }
    ScheduleFormat::English => {
      let cron =
        english_to_cron::str_cron_syntax(&procedure.config.schedule)
          .map_err(|e| {
            anyhow!("Failed to parse english to cron | {e:?}")
          })?
          .split(' ')
          // croner does not accept year
          .take(6)
          .collect::<Vec<_>>()
          .join(" ");
      croner::Cron::new(&cron).parse().with_context(|| {
        format!("Failed to parse schedule CRON: {cron}")
      })?
    }
  };
  let tz: chrono_tz::Tz = procedure
    .config
    .schedule_timezone
    .parse()
    .context("Failed to parse schedule timezone")?;
  let tz_time = chrono::Local::now().with_timezone(&tz);
  let next = cron
    .find_next_occurrence(&tz_time, false)
    .context("Failed to find next run time")?
    .timestamp_millis();
  Ok(next)
}
