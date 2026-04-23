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
    procedure::{Procedure, ProcedureStage},
    repo::Repo,
    stack::Stack,
    update::{Log, Update},
    user::procedure_user,
  },
};
use mogh_resolver::Resolve;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
  api::{
    execute::{ExecuteArgs, ExecuteRequest},
    write::WriteArgs,
  },
  resource::{KomodoResource, list_full_for_user_using_pattern},
  state::{all_resources_cache, db_client},
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
  let task_id = Uuid::new_v4();
  // Standard pattern: init update, resolve with ExecuteArgs, handle result.
  macro_rules! resolve_execute {
    ($Variant:ident, $req:expr) => {{
      let req = ExecuteRequest::$Variant($req);
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::$Variant(req) = req else {
        unreachable!()
      };
      let update_id = update.id.clone();
      handle_resolve_result(
        req
          .resolve(&ExecuteArgs {
            user,
            update,
            task_id,
          })
          .await
          .map_err(|e| e.error)
          .context(concat!("Failed at ", stringify!($Variant))),
        &update_id,
      )
      .await?
    }};
  }

  // Batch methods must be expanded in `execute_stage`.
  macro_rules! batch_not_implemented {
    ($Variant:ident) => {
      return Err(anyhow!(concat!(
        "Batch method ",
        stringify!($Variant),
        " not implemented correctly"
      )))
    };
  }

  let update = match execution {
    Execution::None(_) => return Ok(()),
    // Special: self-referential guard
    Execution::RunProcedure(req) => {
      if req.procedure == parent_id || req.procedure == parent_name {
        return Err(anyhow!("Self referential procedure detected"));
      }
      resolve_execute!(RunProcedure, req)
    }
    // Special: write operation
    Execution::CommitSync(req) => req
      .resolve(&WriteArgs { user })
      .await
      .map_err(|e| e.error)
      .context("Failed at CommitSync")?,
    // Special: sleep
    Execution::Sleep(req) => {
      let duration = Duration::from_millis(req.duration_ms as u64);
      tokio::time::sleep(duration).await;
      Update {
        success: true,
        ..Default::default()
      }
    }
    // Batch variants
    Execution::BatchRunProcedure(_) => {
      batch_not_implemented!(BatchRunProcedure)
    }
    Execution::BatchRunAction(_) => {
      batch_not_implemented!(BatchRunAction)
    }
    Execution::BatchRunBuild(_) => {
      batch_not_implemented!(BatchRunBuild)
    }
    Execution::BatchDeploy(_) => batch_not_implemented!(BatchDeploy),
    Execution::BatchDestroyDeployment(_) => {
      batch_not_implemented!(BatchDestroyDeployment)
    }
    Execution::BatchCloneRepo(_) => {
      batch_not_implemented!(BatchCloneRepo)
    }
    Execution::BatchPullRepo(_) => {
      batch_not_implemented!(BatchPullRepo)
    }
    Execution::BatchBuildRepo(_) => {
      batch_not_implemented!(BatchBuildRepo)
    }
    Execution::BatchDeployStack(_) => {
      batch_not_implemented!(BatchDeployStack)
    }
    Execution::BatchDeployStackIfChanged(_) => {
      batch_not_implemented!(BatchDeployStackIfChanged)
    }
    Execution::BatchPullStack(_) => {
      batch_not_implemented!(BatchPullStack)
    }
    Execution::BatchDestroyStack(_) => {
      batch_not_implemented!(BatchDestroyStack)
    }
    // Standard executions
    Execution::RunAction(req) => resolve_execute!(RunAction, req),
    Execution::RunBuild(req) => resolve_execute!(RunBuild, req),
    Execution::CancelBuild(req) => resolve_execute!(CancelBuild, req),
    Execution::Deploy(req) => resolve_execute!(Deploy, req),
    Execution::PullDeployment(req) => {
      resolve_execute!(PullDeployment, req)
    }
    Execution::StartDeployment(req) => {
      resolve_execute!(StartDeployment, req)
    }
    Execution::RestartDeployment(req) => {
      resolve_execute!(RestartDeployment, req)
    }
    Execution::PauseDeployment(req) => {
      resolve_execute!(PauseDeployment, req)
    }
    Execution::UnpauseDeployment(req) => {
      resolve_execute!(UnpauseDeployment, req)
    }
    Execution::StopDeployment(req) => {
      resolve_execute!(StopDeployment, req)
    }
    Execution::DestroyDeployment(req) => {
      resolve_execute!(DestroyDeployment, req)
    }
    Execution::CloneRepo(req) => resolve_execute!(CloneRepo, req),
    Execution::PullRepo(req) => resolve_execute!(PullRepo, req),
    Execution::BuildRepo(req) => resolve_execute!(BuildRepo, req),
    Execution::CancelRepoBuild(req) => {
      resolve_execute!(CancelRepoBuild, req)
    }
    Execution::StartContainer(req) => {
      resolve_execute!(StartContainer, req)
    }
    Execution::RestartContainer(req) => {
      resolve_execute!(RestartContainer, req)
    }
    Execution::PauseContainer(req) => {
      resolve_execute!(PauseContainer, req)
    }
    Execution::UnpauseContainer(req) => {
      resolve_execute!(UnpauseContainer, req)
    }
    Execution::StopContainer(req) => {
      resolve_execute!(StopContainer, req)
    }
    Execution::DestroyContainer(req) => {
      resolve_execute!(DestroyContainer, req)
    }
    Execution::StartAllContainers(req) => {
      resolve_execute!(StartAllContainers, req)
    }
    Execution::RestartAllContainers(req) => {
      resolve_execute!(RestartAllContainers, req)
    }
    Execution::PauseAllContainers(req) => {
      resolve_execute!(PauseAllContainers, req)
    }
    Execution::UnpauseAllContainers(req) => {
      resolve_execute!(UnpauseAllContainers, req)
    }
    Execution::StopAllContainers(req) => {
      resolve_execute!(StopAllContainers, req)
    }
    Execution::PruneContainers(req) => {
      resolve_execute!(PruneContainers, req)
    }
    Execution::DeleteNetwork(req) => {
      resolve_execute!(DeleteNetwork, req)
    }
    Execution::PruneNetworks(req) => {
      resolve_execute!(PruneNetworks, req)
    }
    Execution::DeleteImage(req) => resolve_execute!(DeleteImage, req),
    Execution::PruneImages(req) => resolve_execute!(PruneImages, req),
    Execution::DeleteVolume(req) => {
      resolve_execute!(DeleteVolume, req)
    }
    Execution::PruneVolumes(req) => {
      resolve_execute!(PruneVolumes, req)
    }
    Execution::PruneDockerBuilders(req) => {
      resolve_execute!(PruneDockerBuilders, req)
    }
    Execution::PruneBuildx(req) => resolve_execute!(PruneBuildx, req),
    Execution::PruneSystem(req) => resolve_execute!(PruneSystem, req),
    Execution::RunSync(req) => resolve_execute!(RunSync, req),
    Execution::DeployStack(req) => resolve_execute!(DeployStack, req),
    Execution::DeployStackIfChanged(req) => {
      resolve_execute!(DeployStackIfChanged, req)
    }
    Execution::PullStack(req) => resolve_execute!(PullStack, req),
    Execution::StartStack(req) => resolve_execute!(StartStack, req),
    Execution::RestartStack(req) => {
      resolve_execute!(RestartStack, req)
    }
    Execution::PauseStack(req) => resolve_execute!(PauseStack, req),
    Execution::UnpauseStack(req) => {
      resolve_execute!(UnpauseStack, req)
    }
    Execution::StopStack(req) => resolve_execute!(StopStack, req),
    Execution::DestroyStack(req) => {
      resolve_execute!(DestroyStack, req)
    }
    Execution::RunStackService(req) => {
      resolve_execute!(RunStackService, req)
    }
    Execution::TestAlerter(req) => resolve_execute!(TestAlerter, req),
    Execution::SendAlert(req) => resolve_execute!(SendAlert, req),
    Execution::RemoveSwarmNodes(req) => {
      resolve_execute!(RemoveSwarmNodes, req)
    }
    Execution::UpdateSwarmNode(req) => {
      resolve_execute!(UpdateSwarmNode, req)
    }
    Execution::RemoveSwarmStacks(req) => {
      resolve_execute!(RemoveSwarmStacks, req)
    }
    Execution::RemoveSwarmServices(req) => {
      resolve_execute!(RemoveSwarmServices, req)
    }
    Execution::CreateSwarmConfig(req) => {
      resolve_execute!(CreateSwarmConfig, req)
    }
    Execution::RotateSwarmConfig(req) => {
      resolve_execute!(RotateSwarmConfig, req)
    }
    Execution::RemoveSwarmConfigs(req) => {
      resolve_execute!(RemoveSwarmConfigs, req)
    }
    Execution::CreateSwarmSecret(req) => {
      resolve_execute!(CreateSwarmSecret, req)
    }
    Execution::RotateSwarmSecret(req) => {
      resolve_execute!(RotateSwarmSecret, req)
    }
    Execution::RemoveSwarmSecrets(req) => {
      resolve_execute!(RemoveSwarmSecrets, req)
    }
    Execution::ClearRepoCache(req) => {
      resolve_execute!(ClearRepoCache, req)
    }
    Execution::BackupCoreDatabase(req) => {
      resolve_execute!(BackupCoreDatabase, req)
    }
    Execution::GlobalAutoUpdate(req) => {
      resolve_execute!(GlobalAutoUpdate, req)
    }
    Execution::RotateAllServerKeys(req) => {
      resolve_execute!(RotateAllServerKeys, req)
    }
    Execution::RotateCoreKeys(req) => {
      resolve_execute!(RotateCoreKeys, req)
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

pub fn replace_procedure_stage_ids_with_names(
  stages: &mut Vec<ProcedureStage>,
) {
  let all = all_resources_cache().load();
  for stage in stages {
    for execution in &mut stage.executions {
      // Replaces an id field on config with the resource name from `all`.
      macro_rules! replace_id_with_name {
          ($($Variant:ident => $field:ident, $collection:ident);* $(;)?) => {
            match &mut execution.execution {
              $(
                Execution::$Variant(config) => {
                  config.$field = all
                    .$collection
                    .get(&config.$field)
                    .map(|r| r.name.clone())
                    .unwrap_or_default();
                }
              )*
              // SendAlert maps a Vec of alerter ids
              Execution::SendAlert(config) => {
                config.alerters = config
                  .alerters
                  .iter()
                  .map(|alerter| {
                    all
                      .alerters
                      .get(alerter)
                      .map(|a| a.name.clone())
                      .unwrap_or_default()
                  })
                  .collect();
              }
              // No-op variants
              Execution::None(_)
              | Execution::BatchRunProcedure(_)
              | Execution::BatchRunAction(_)
              | Execution::BatchRunBuild(_)
              | Execution::BatchDeploy(_)
              | Execution::BatchDestroyDeployment(_)
              | Execution::BatchCloneRepo(_)
              | Execution::BatchPullRepo(_)
              | Execution::BatchBuildRepo(_)
              | Execution::BatchDeployStack(_)
              | Execution::BatchDeployStackIfChanged(_)
              | Execution::BatchPullStack(_)
              | Execution::BatchDestroyStack(_)
              | Execution::ClearRepoCache(_)
              | Execution::BackupCoreDatabase(_)
              | Execution::GlobalAutoUpdate(_)
              | Execution::RotateAllServerKeys(_)
              | Execution::RotateCoreKeys(_)
              | Execution::Sleep(_) => {}
            }
          };
        }

      replace_id_with_name!(
        RunProcedure => procedure, procedures;
        RunAction => action, actions;
        RunBuild => build, builds;
        CancelBuild => build, builds;
        Deploy => deployment, deployments;
        PullDeployment => deployment, deployments;
        StartDeployment => deployment, deployments;
        RestartDeployment => deployment, deployments;
        PauseDeployment => deployment, deployments;
        UnpauseDeployment => deployment, deployments;
        StopDeployment => deployment, deployments;
        DestroyDeployment => deployment, deployments;
        CloneRepo => repo, repos;
        PullRepo => repo, repos;
        BuildRepo => repo, repos;
        CancelRepoBuild => repo, repos;
        StartContainer => server, servers;
        RestartContainer => server, servers;
        PauseContainer => server, servers;
        UnpauseContainer => server, servers;
        StopContainer => server, servers;
        DestroyContainer => server, servers;
        StartAllContainers => server, servers;
        RestartAllContainers => server, servers;
        PauseAllContainers => server, servers;
        UnpauseAllContainers => server, servers;
        StopAllContainers => server, servers;
        PruneContainers => server, servers;
        DeleteNetwork => server, servers;
        PruneNetworks => server, servers;
        DeleteImage => server, servers;
        PruneImages => server, servers;
        DeleteVolume => server, servers;
        PruneVolumes => server, servers;
        PruneDockerBuilders => server, servers;
        PruneBuildx => server, servers;
        PruneSystem => server, servers;
        RunSync => sync, syncs;
        CommitSync => sync, syncs;
        DeployStack => stack, stacks;
        DeployStackIfChanged => stack, stacks;
        PullStack => stack, stacks;
        StartStack => stack, stacks;
        RestartStack => stack, stacks;
        PauseStack => stack, stacks;
        UnpauseStack => stack, stacks;
        StopStack => stack, stacks;
        DestroyStack => stack, stacks;
        RunStackService => stack, stacks;
        TestAlerter => alerter, alerters;
        RemoveSwarmNodes => swarm, swarms;
        UpdateSwarmNode => swarm, swarms;
        RemoveSwarmStacks => swarm, swarms;
        RemoveSwarmServices => swarm, swarms;
        CreateSwarmConfig => swarm, swarms;
        RotateSwarmConfig => swarm, swarms;
        RemoveSwarmConfigs => swarm, swarms;
        CreateSwarmSecret => swarm, swarms;
        RotateSwarmSecret => swarm, swarms;
        RemoveSwarmSecrets => swarm, swarms;
      );
    }
  }
}
