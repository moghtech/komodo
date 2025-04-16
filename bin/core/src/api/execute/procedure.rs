use std::pin::Pin;

use formatting::{Color, bold, colored, format_serror, muted};
use komodo_client::{
  api::execute::{
    BatchExecutionResponse, BatchRunProcedure, RunProcedure,
  },
  entities::{
    alert::{Alert, AlertData, SeverityLevel}, komodo_timestamp, permission::PermissionLevel, procedure::Procedure, update::Update, user::User
  },
};
use mungos::{by_id::update_one_by_id, mongodb::bson::to_document};
use resolver_api::Resolve;
use tokio::sync::Mutex;

use crate::{
  alert::send_alerts, helpers::{procedure::execute_procedure, update::update_update}, resource::{self, refresh_procedure_state_cache}, state::{action_states, db_client}
};

use super::{ExecuteArgs, ExecuteRequest};

impl super::BatchExecute for BatchRunProcedure {
  type Resource = Procedure;
  fn single_request(procedure: String) -> ExecuteRequest {
    ExecuteRequest::RunProcedure(RunProcedure { procedure })
  }
}

impl Resolve<ExecuteArgs> for BatchRunProcedure {
  #[instrument(name = "BatchRunProcedure", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, .. }: &ExecuteArgs,
  ) -> serror::Result<BatchExecutionResponse> {
    Ok(
      super::batch_execute::<BatchRunProcedure>(&self.pattern, user)
        .await?,
    )
  }
}

impl Resolve<ExecuteArgs> for RunProcedure {
  #[instrument(name = "RunProcedure", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> serror::Result<Update> {
    Ok(
      resolve_inner(self.procedure, user.clone(), update.clone())
        .await?,
    )
  }
}

fn resolve_inner(
  procedure: String,
  user: User,
  mut update: Update,
) -> Pin<
  Box<
    dyn std::future::Future<Output = anyhow::Result<Update>> + Send,
  >,
> {
  Box::pin(async move {
    let procedure = resource::get_check_permissions::<Procedure>(
      &procedure,
      &user,
      PermissionLevel::Execute,
    )
    .await?;

    // Need to push the initial log, as execute_procedure
    // assumes first log is already created
    // and will panic otherwise.
    update.push_simple_log(
      "Execute procedure",
      format!(
        "{}: executing procedure '{}'",
        muted("INFO"),
        bold(&procedure.name)
      ),
    );

    // get the action state for the procedure (or insert default).
    let action_state = action_states()
      .procedure
      .get_or_insert_default(&procedure.id)
      .await;

    // This will set action state back to default when dropped.
    // Will also check to ensure procedure not already busy before updating.
    let _action_guard =
      action_state.update(|state| state.running = true)?;

    update_update(update.clone()).await?;

    let update = Mutex::new(update);

    let res = execute_procedure(&procedure, &update).await;

    let mut update = update.into_inner();

    match res {
      Ok(_) => {
        update.push_simple_log(
          "Execution ok",
          format!(
            "{}: The procedure has {} with no errors",
            muted("INFO"),
            colored("completed", Color::Green)
          ),
        );
      }
      Err(e) => update
        .push_error_log("execution error", format_serror(&e.into())),
    }

    update.finalize();

    // Need to manually update the update before cache refresh,
    // and before broadcast with add_update.
    // The Err case of to_document should be unreachable,
    // but will fail to update cache in that case.
    if let Ok(update_doc) = to_document(&update) {
      let _ = update_one_by_id(
        &db_client().updates,
        &update.id,
        mungos::update::Update::Set(update_doc),
        None,
      )
      .await;
      refresh_procedure_state_cache().await;
    }

    update_update(update.clone()).await?;

    if !update.success && procedure.config.failure_alert {
      warn!("procedure unsuccessful, alerting...");
      let target = update.target.clone();
      tokio::spawn(async move {
        let alert = Alert {
          id: Default::default(),
          target,
          ts: komodo_timestamp(),
          resolved_ts: Some(komodo_timestamp()),
          resolved: true,
          level: SeverityLevel::Warning,
          data: AlertData::ProcedureFailed {
            id: procedure.id,
            name: procedure.name,
          },
        };
        send_alerts(&[alert]).await
      });
    }

    Ok(update)
  })
}
