use std::time::{Duration, Instant};

use anyhow::{Context, anyhow};
use database::mungos::by_id::find_one_by_id;
use formatting::{Color, bold, colored, format_serror, muted};
use futures_util::future::join_all;
use komodo_client::{
  api::execute::*,
  entities::{
    action::Action,
    build::Build,
    deployment::Deployment,
    permission::PermissionLevel,
    procedure::Procedure,
    repo::Repo,
    stack::Stack,
    update::{Log, Update},
    user::procedure_user,
  },
};
use resolver_api::Resolve;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
  api::{
    execute::{ExecuteArgs, ExecuteRequest},
    write::WriteArgs,
  },
  resource::{KomodoResource, list_full_for_user_using_pattern},
  state::db_client,
};

use super::update::{init_execution_update, update_update};

pub async fn execute_procedure(
  procedure: &Procedure,
  update: &Mutex<Update>,
) -> anyhow::Result<()> {
  for stage in &procedure.config.stages {
    if !stage.enabled {
      continue;
    }
    add_line_to_update(
      update,
      &format!(
        "{}: Executing stage: '{}'",
        muted("INFO"),
        bold(&stage.name)
      ),
    )
    .await;
    let timer = Instant::now();
    execute_procedure_stage(
      stage
        .executions
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.execution.clone())
        .collect(),
      &procedure.id,
      &procedure.name,
      update,
    )
    .await
    .with_context(|| {
      format!(
        "Failed stage '{}' execution after {:?}",
        bold(&stage.name),
        timer.elapsed(),
      )
    })?;
    add_line_to_update(
      update,
      &format!(
        "{}: {} stage '{}' execution in {:?}",
        muted("INFO"),
        colored("Finished", Color::Green),
        bold(&stage.name),
        timer.elapsed()
      ),
    )
    .await;
  }

  Ok(())
}

#[instrument("ExecuteProcedureStage", skip_all)]
async fn execute_procedure_stage(
  _executions: Vec<Execution>,
  parent_id: &str,
  parent_name: &str,
  update: &Mutex<Update>,
) -> anyhow::Result<()> {
  let mut executions = Vec::with_capacity(_executions.capacity());
  for execution in _executions {
    match execution {
      Execution::BatchRunAction(exec) => {
        extend_batch_exection::<BatchRunAction>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchRunProcedure(exec) => {
        extend_batch_exection::<BatchRunProcedure>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchRunBuild(exec) => {
        extend_batch_exection::<BatchRunBuild>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchCloneRepo(exec) => {
        extend_batch_exection::<BatchCloneRepo>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchPullRepo(exec) => {
        extend_batch_exection::<BatchPullRepo>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchBuildRepo(exec) => {
        extend_batch_exection::<BatchBuildRepo>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchDeploy(exec) => {
        extend_batch_exection::<BatchDeploy>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchDestroyDeployment(exec) => {
        extend_batch_exection::<BatchDestroyDeployment>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchDeployStack(exec) => {
        extend_batch_exection::<BatchDeployStack>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchDeployStackIfChanged(exec) => {
        extend_batch_exection::<BatchDeployStackIfChanged>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchPullStack(exec) => {
        extend_batch_exection::<BatchPullStack>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      Execution::BatchDestroyStack(exec) => {
        extend_batch_exection::<BatchDestroyStack>(
          &exec.pattern,
          &mut executions,
        )
        .await?;
      }
      execution => executions.push(execution),
    }
  }
  let futures = executions.into_iter().map(|execution| async move {
    let now = Instant::now();
    add_line_to_update(
      update,
      &format!("{}: Executing: {execution:?}", muted("INFO")),
    )
    .await;
    let fail_log = format!(
      "{}: Failed on {execution:?}",
      colored("ERROR", Color::Red)
    );
    let res =
      execute_execution(execution.clone(), parent_id, parent_name)
        .await
        .context(fail_log);
    add_line_to_update(
      update,
      &format!(
        "{}: {} execution in {:?}: {execution:?}",
        muted("INFO"),
        colored("Finished", Color::Green),
        now.elapsed()
      ),
    )
    .await;
    res
  });
  join_all(futures)
    .await
    .into_iter()
    .collect::<anyhow::Result<Vec<_>>>()?;
  Ok(())
}

#[instrument(
  "ExecuteProcedureExecution",
  skip(parent_id, parent_name)
)]
async fn execute_execution(
  execution: Execution,
  // used to prevent recursive procedure
  parent_id: &str,
  parent_name: &str,
) -> anyhow::Result<()> {
  let user = procedure_user().to_owned();
  let id = Uuid::new_v4();
  let update = match execution {
    Execution::None(_) => return Ok(()),
    Execution::RunProcedure(req) => {
      if req.procedure == parent_id || req.procedure == parent_name {
        return Err(anyhow!("Self referential procedure detected"));
      }
      let req = ExecuteRequest::RunProcedure(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RunProcedure(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RunProcedure"),
        &update_id,
      )
      .await?
    }
    Execution::BatchRunProcedure(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchRunProcedure not implemented correctly"
      ));
    }
    Execution::RunAction(req) => {
      let req = ExecuteRequest::RunAction(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RunAction(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RunAction"),
        &update_id,
      )
      .await?
    }
    Execution::BatchRunAction(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchRunAction not implemented correctly"
      ));
    }
    Execution::RunBuild(req) => {
      let req = ExecuteRequest::RunBuild(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RunBuild(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RunBuild"),
        &update_id,
      )
      .await?
    }
    Execution::BatchRunBuild(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchRunBuild not implemented correctly"
      ));
    }
    Execution::CancelBuild(req) => {
      let req = ExecuteRequest::CancelBuild(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::CancelBuild(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at CancelBuild"),
        &update_id,
      )
      .await?
    }
    Execution::Deploy(req) => {
      let req = ExecuteRequest::Deploy(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::Deploy(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at Deploy"),
        &update_id,
      )
      .await?
    }
    Execution::BatchDeploy(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchDeploy not implemented correctly"
      ));
    }
    Execution::PullDeployment(req) => {
      let req = ExecuteRequest::PullDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PullDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PullDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::StartDeployment(req) => {
      let req = ExecuteRequest::StartDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StartDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StartDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::RestartDeployment(req) => {
      let req = ExecuteRequest::RestartDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RestartDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RestartDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::PauseDeployment(req) => {
      let req = ExecuteRequest::PauseDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PauseDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PauseDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::UnpauseDeployment(req) => {
      let req = ExecuteRequest::UnpauseDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::UnpauseDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at UnpauseDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::StopDeployment(req) => {
      let req = ExecuteRequest::StopDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StopDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StopDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::DestroyDeployment(req) => {
      let req = ExecuteRequest::DestroyDeployment(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DestroyDeployment(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DestroyDeployment"),
        &update_id,
      )
      .await?
    }
    Execution::BatchDestroyDeployment(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchDestroyDeployment not implemented correctly"
      ));
    }
    Execution::CloneRepo(req) => {
      let req = ExecuteRequest::CloneRepo(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::CloneRepo(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at CloneRepo"),
        &update_id,
      )
      .await?
    }
    Execution::BatchCloneRepo(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchCloneRepo not implemented correctly"
      ));
    }
    Execution::PullRepo(req) => {
      let req = ExecuteRequest::PullRepo(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PullRepo(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PullRepo"),
        &update_id,
      )
      .await?
    }
    Execution::BatchPullRepo(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchPullRepo not implemented correctly"
      ));
    }
    Execution::BuildRepo(req) => {
      let req = ExecuteRequest::BuildRepo(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::BuildRepo(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at BuildRepo"),
        &update_id,
      )
      .await?
    }
    Execution::BatchBuildRepo(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchBuildRepo not implemented correctly"
      ));
    }
    Execution::CancelRepoBuild(req) => {
      let req = ExecuteRequest::CancelRepoBuild(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::CancelRepoBuild(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at CancelRepoBuild"),
        &update_id,
      )
      .await?
    }
    Execution::StartContainer(req) => {
      let req = ExecuteRequest::StartContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StartContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StartContainer"),
        &update_id,
      )
      .await?
    }
    Execution::RestartContainer(req) => {
      let req = ExecuteRequest::RestartContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RestartContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RestartContainer"),
        &update_id,
      )
      .await?
    }
    Execution::PauseContainer(req) => {
      let req = ExecuteRequest::PauseContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PauseContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PauseContainer"),
        &update_id,
      )
      .await?
    }
    Execution::UnpauseContainer(req) => {
      let req = ExecuteRequest::UnpauseContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::UnpauseContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at UnpauseContainer"),
        &update_id,
      )
      .await?
    }
    Execution::StopContainer(req) => {
      let req = ExecuteRequest::StopContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StopContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StopContainer"),
        &update_id,
      )
      .await?
    }
    Execution::DestroyContainer(req) => {
      let req = ExecuteRequest::DestroyContainer(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DestroyContainer(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveContainer"),
        &update_id,
      )
      .await?
    }
    Execution::StartAllContainers(req) => {
      let req = ExecuteRequest::StartAllContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StartAllContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StartAllContainers"),
        &update_id,
      )
      .await?
    }
    Execution::RestartAllContainers(req) => {
      let req = ExecuteRequest::RestartAllContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RestartAllContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RestartAllContainers"),
        &update_id,
      )
      .await?
    }
    Execution::PauseAllContainers(req) => {
      let req = ExecuteRequest::PauseAllContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PauseAllContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PauseAllContainers"),
        &update_id,
      )
      .await?
    }
    Execution::UnpauseAllContainers(req) => {
      let req = ExecuteRequest::UnpauseAllContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::UnpauseAllContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at UnpauseAllContainers"),
        &update_id,
      )
      .await?
    }
    Execution::StopAllContainers(req) => {
      let req = ExecuteRequest::StopAllContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StopAllContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StopAllContainers"),
        &update_id,
      )
      .await?
    }
    Execution::PruneContainers(req) => {
      let req = ExecuteRequest::PruneContainers(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneContainers(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneContainers"),
        &update_id,
      )
      .await?
    }
    Execution::DeleteNetwork(req) => {
      let req = ExecuteRequest::DeleteNetwork(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeleteNetwork(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DeleteNetwork"),
        &update_id,
      )
      .await?
    }
    Execution::PruneNetworks(req) => {
      let req = ExecuteRequest::PruneNetworks(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneNetworks(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneNetworks"),
        &update_id,
      )
      .await?
    }
    Execution::DeleteImage(req) => {
      let req = ExecuteRequest::DeleteImage(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeleteImage(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DeleteImage"),
        &update_id,
      )
      .await?
    }
    Execution::PruneImages(req) => {
      let req = ExecuteRequest::PruneImages(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneImages(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneImages"),
        &update_id,
      )
      .await?
    }
    Execution::DeleteVolume(req) => {
      let req = ExecuteRequest::DeleteVolume(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeleteVolume(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DeleteVolume"),
        &update_id,
      )
      .await?
    }
    Execution::PruneVolumes(req) => {
      let req = ExecuteRequest::PruneVolumes(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneVolumes(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneVolumes"),
        &update_id,
      )
      .await?
    }
    Execution::PruneDockerBuilders(req) => {
      let req = ExecuteRequest::PruneDockerBuilders(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneDockerBuilders(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneDockerBuilders"),
        &update_id,
      )
      .await?
    }
    Execution::PruneBuildx(req) => {
      let req = ExecuteRequest::PruneBuildx(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneBuildx(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneBuildx"),
        &update_id,
      )
      .await?
    }
    Execution::PruneSystem(req) => {
      let req = ExecuteRequest::PruneSystem(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PruneSystem(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PruneSystem"),
        &update_id,
      )
      .await?
    }
    Execution::RunSync(req) => {
      let req = ExecuteRequest::RunSync(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RunSync(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RunSync"),
        &update_id,
      )
      .await?
    }
    // Exception: This is a write operation.
    Execution::CommitSync(req) => req
      .resolve(&WriteArgs { user })
      .await
      .map_err(|e| e.error)
      .context("Failed at CommitSync")?,
    Execution::DeployStack(req) => {
      let req = ExecuteRequest::DeployStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeployStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DeployStack"),
        &update_id,
      )
      .await?
    }
    Execution::BatchDeployStack(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchDeployStack not implemented correctly"
      ));
    }
    Execution::DeployStackIfChanged(req) => {
      let req = ExecuteRequest::DeployStackIfChanged(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeployStackIfChanged(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DeployStackIfChanged"),
        &update_id,
      )
      .await?
    }
    Execution::BatchDeployStackIfChanged(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchDeployStackIfChanged not implemented correctly"
      ));
    }
    Execution::PullStack(req) => {
      let req = ExecuteRequest::PullStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PullStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PullStack"),
        &update_id,
      )
      .await?
    }
    Execution::BatchPullStack(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchPullStack not implemented correctly"
      ));
    }
    Execution::StartStack(req) => {
      let req = ExecuteRequest::StartStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StartStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StartStack"),
        &update_id,
      )
      .await?
    }
    Execution::RestartStack(req) => {
      let req = ExecuteRequest::RestartStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RestartStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RestartStack"),
        &update_id,
      )
      .await?
    }
    Execution::PauseStack(req) => {
      let req = ExecuteRequest::PauseStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::PauseStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at PauseStack"),
        &update_id,
      )
      .await?
    }
    Execution::UnpauseStack(req) => {
      let req = ExecuteRequest::UnpauseStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::UnpauseStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at UnpauseStack"),
        &update_id,
      )
      .await?
    }
    Execution::StopStack(req) => {
      let req = ExecuteRequest::StopStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::StopStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at StopStack"),
        &update_id,
      )
      .await?
    }
    Execution::DestroyStack(req) => {
      let req = ExecuteRequest::DestroyStack(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DestroyStack(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at DestroyStack"),
        &update_id,
      )
      .await?
    }
    Execution::RunStackService(req) => {
      let req = ExecuteRequest::RunStackService(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RunStackService(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RunStackService"),
        &update_id,
      )
      .await?
    }
    Execution::BatchDestroyStack(_) => {
      // All batch executions must be expanded in `execute_stage`
      return Err(anyhow!(
        "Batch method BatchDestroyStack not implemented correctly"
      ));
    }
    Execution::TestAlerter(req) => {
      let req = ExecuteRequest::TestAlerter(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::TestAlerter(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at TestAlerter"),
        &update_id,
      )
      .await?
    }
    Execution::SendAlert(req) => {
      let req = ExecuteRequest::SendAlert(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::SendAlert(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at SendAlert"),
        &update_id,
      )
      .await?
    }
    Execution::RemoveSwarmNodes(req) => {
      let req = ExecuteRequest::RemoveSwarmNodes(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RemoveSwarmNodes(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveSwarmNodes"),
        &update_id,
      )
      .await?
    }
    Execution::RemoveSwarmStacks(req) => {
      let req = ExecuteRequest::RemoveSwarmStacks(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RemoveSwarmStacks(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveSwarmStacks"),
        &update_id,
      )
      .await?
    }
    Execution::RemoveSwarmServices(req) => {
      let req = ExecuteRequest::RemoveSwarmServices(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RemoveSwarmServices(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveSwarmServices"),
        &update_id,
      )
      .await?
    }
    Execution::RemoveSwarmConfigs(req) => {
      let req = ExecuteRequest::RemoveSwarmConfigs(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RemoveSwarmConfigs(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveSwarmConfigs"),
        &update_id,
      )
      .await?
    }
    Execution::RemoveSwarmSecrets(req) => {
      let req = ExecuteRequest::RemoveSwarmSecrets(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RemoveSwarmSecrets(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RemoveSwarmSecrets"),
        &update_id,
      )
      .await?
    }
    Execution::ClearRepoCache(req) => {
      let req = ExecuteRequest::ClearRepoCache(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::ClearRepoCache(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at ClearRepoCache"),
        &update_id,
      )
      .await?
    }
    Execution::BackupCoreDatabase(req) => {
      let req = ExecuteRequest::BackupCoreDatabase(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::BackupCoreDatabase(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at BackupCoreDatabase"),
        &update_id,
      )
      .await?
    }
    Execution::GlobalAutoUpdate(req) => {
      let req = ExecuteRequest::GlobalAutoUpdate(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::GlobalAutoUpdate(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at GlobalAutoUpdate"),
        &update_id,
      )
      .await?
    }
    Execution::RotateAllServerKeys(req) => {
      let req = ExecuteRequest::RotateAllServerKeys(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RotateAllServerKeys(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RotateAllServerKeys"),
        &update_id,
      )
      .await?
    }
    Execution::RotateCoreKeys(req) => {
      let req = ExecuteRequest::RotateCoreKeys(req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::RotateCoreKeys(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs { user, update, id })
          .await
          .map_err(|e| e.error)
          .context("Failed at RotateCoreKeys"),
        &update_id,
      )
      .await?
    }
    Execution::Sleep(req) => {
      let duration = Duration::from_millis(req.duration_ms as u64);
      tokio::time::sleep(duration).await;
      Update {
        success: true,
        ..Default::default()
      }
    }
  };
  if update.success {
    Ok(())
  } else {
    Err(anyhow!(
      "{}: execution not successful. see update '{}'",
      colored("ERROR", Color::Red),
      bold(&update.id),
    ))
  }
}

/// If the call to .resolve returns Err, the update may not be closed.
/// This will ensure it is closed with error log attached.
async fn handle_resolve_result(
  res: anyhow::Result<Update>,
  update_id: &str,
) -> anyhow::Result<Update> {
  match res {
    Ok(res) => Ok(res),
    Err(e) => {
      let log =
        Log::error("execution error", format_serror(&e.into()));
      let mut update =
        find_one_by_id(&db_client().updates, update_id)
          .await
          .context("Failed to query to db")?
          .context("no update exists with given id")?;
      update.logs.push(log);
      update.finalize();
      update_update(update.clone()).await?;
      Ok(update)
    }
  }
}

/// ASSUMES FIRST LOG IS ALREADY CREATED
async fn add_line_to_update(update: &Mutex<Update>, line: &str) {
  let mut lock = update.lock().await;
  let log = &mut lock.logs[0];
  log.stdout.push('\n');
  log.stdout.push_str(line);
  let update = lock.clone();
  drop(lock);
  if let Err(e) = update_update(update).await {
    error!("Failed to update an update during procedure | {e:#}");
  };
}

async fn extend_batch_exection<E: ExtendBatch>(
  pattern: &str,
  executions: &mut Vec<Execution>,
) -> anyhow::Result<()> {
  let more = list_full_for_user_using_pattern::<E::Resource>(
    pattern,
    Default::default(),
    procedure_user(),
    PermissionLevel::Read.into(),
    &[],
  )
  .await?
  .into_iter()
  .map(|resource| E::single_execution(resource.name));
  executions.extend(more);
  Ok(())
}

trait ExtendBatch {
  type Resource: KomodoResource;
  fn single_execution(name: String) -> Execution;
}

impl ExtendBatch for BatchRunProcedure {
  type Resource = Procedure;
  fn single_execution(procedure: String) -> Execution {
    Execution::RunProcedure(RunProcedure { procedure })
  }
}

impl ExtendBatch for BatchRunAction {
  type Resource = Action;
  fn single_execution(action: String) -> Execution {
    Execution::RunAction(RunAction {
      action,
      args: Default::default(),
    })
  }
}

impl ExtendBatch for BatchRunBuild {
  type Resource = Build;
  fn single_execution(build: String) -> Execution {
    Execution::RunBuild(RunBuild { build })
  }
}

impl ExtendBatch for BatchCloneRepo {
  type Resource = Repo;
  fn single_execution(repo: String) -> Execution {
    Execution::CloneRepo(CloneRepo { repo })
  }
}

impl ExtendBatch for BatchPullRepo {
  type Resource = Repo;
  fn single_execution(repo: String) -> Execution {
    Execution::PullRepo(PullRepo { repo })
  }
}

impl ExtendBatch for BatchBuildRepo {
  type Resource = Repo;
  fn single_execution(repo: String) -> Execution {
    Execution::BuildRepo(BuildRepo { repo })
  }
}

impl ExtendBatch for BatchDeploy {
  type Resource = Deployment;
  fn single_execution(deployment: String) -> Execution {
    Execution::Deploy(Deploy {
      deployment,
      stop_signal: None,
      stop_time: None,
    })
  }
}

impl ExtendBatch for BatchDestroyDeployment {
  type Resource = Deployment;
  fn single_execution(deployment: String) -> Execution {
    Execution::DestroyDeployment(DestroyDeployment {
      deployment,
      signal: None,
      time: None,
    })
  }
}

impl ExtendBatch for BatchDeployStack {
  type Resource = Stack;
  fn single_execution(stack: String) -> Execution {
    Execution::DeployStack(DeployStack {
      stack,
      services: Vec::new(),
      stop_time: None,
    })
  }
}

impl ExtendBatch for BatchDeployStackIfChanged {
  type Resource = Stack;
  fn single_execution(stack: String) -> Execution {
    Execution::DeployStackIfChanged(DeployStackIfChanged {
      stack,
      stop_time: None,
    })
  }
}

impl ExtendBatch for BatchPullStack {
  type Resource = Stack;
  fn single_execution(stack: String) -> Execution {
    Execution::PullStack(PullStack {
      stack,
      services: Vec::new(),
    })
  }
}

impl ExtendBatch for BatchDestroyStack {
  type Resource = Stack;
  fn single_execution(stack: String) -> Execution {
    Execution::DestroyStack(DestroyStack {
      stack,
      services: Vec::new(),
      remove_orphans: false,
      stop_time: None,
    })
  }
}
