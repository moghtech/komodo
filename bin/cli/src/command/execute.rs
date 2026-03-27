use std::time::Duration;

use colored::Colorize;
use futures_util::{StreamExt, stream::FuturesUnordered};
use komodo_client::{
  api::execute::{
    BatchExecutionResponse, BatchExecutionResponseItem, Execution,
  },
  entities::{resource_link, update::Update},
};

use crate::config::cli_config;

enum ExecutionResult {
  Single(Box<Update>),
  Batch(BatchExecutionResponse),
}

pub async fn handle(
  execution: &Execution,
  yes: bool,
) -> anyhow::Result<()> {
  if matches!(execution, Execution::None(_)) {
    println!("Got 'none' execution. Doing nothing...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Finished doing nothing. Exiting...");
    std::process::exit(0);
  }

  println!("\n{}: Execution", "Mode".dimmed());

  macro_rules! handle_execution {
    (
      execute: [$($ExecVariant:ident),* $(,)?],
      batch: [$($BatchVariant:ident),* $(,)?],
    ) => {{
      // Print data
      match execution {
        $(Execution::$ExecVariant(data) => println!("{}: {data:?}", "Data".dimmed()),)*
        $(Execution::$BatchVariant(data) => println!("{}: {data:?}", "Data".dimmed()),)*
        Execution::CommitSync(data) => println!("{}: {data:?}", "Data".dimmed()),
        Execution::Sleep(data) => println!("{}: {data:?}", "Data".dimmed()),
        Execution::None(data) => println!("{}: {data:?}", "Data".dimmed()),
      }

      $crate::command::wait_for_enter("run execution", yes)?;

      info!("Running Execution...");

      let client = $crate::command::komodo_client().await?;

      // Execute and get result
      match execution.clone() {
        $(
          Execution::$ExecVariant(request) => client
            .execute(request)
            .await
            .map(|u| ExecutionResult::Single(u.into())),
        )*
        $(
          Execution::$BatchVariant(request) => {
            client.execute(request).await.map(ExecutionResult::Batch)
          }
        )*
        Execution::CommitSync(request) => client
          .write(request)
          .await
          .map(|u| ExecutionResult::Single(u.into())),
        Execution::Sleep(request) => {
          let duration =
            Duration::from_millis(request.duration_ms as u64);
          tokio::time::sleep(duration).await;
          println!("Finished sleeping!");
          std::process::exit(0)
        }
        Execution::None(_) => unreachable!(),
      }
    }};
  }

  let res = handle_execution!(
    execute: [
      RunAction,
      RunProcedure,
      RunBuild,
      CancelBuild,
      Deploy,
      PullDeployment,
      StartDeployment,
      RestartDeployment,
      PauseDeployment,
      UnpauseDeployment,
      StopDeployment,
      DestroyDeployment,
      CloneRepo,
      PullRepo,
      BuildRepo,
      CancelRepoBuild,
      StartContainer,
      RestartContainer,
      PauseContainer,
      UnpauseContainer,
      StopContainer,
      DestroyContainer,
      StartAllContainers,
      RestartAllContainers,
      PauseAllContainers,
      UnpauseAllContainers,
      StopAllContainers,
      PruneContainers,
      DeleteNetwork,
      PruneNetworks,
      DeleteImage,
      PruneImages,
      DeleteVolume,
      PruneVolumes,
      PruneDockerBuilders,
      PruneBuildx,
      PruneSystem,
      RunSync,
      DeployStack,
      DeployStackIfChanged,
      PullStack,
      StartStack,
      RestartStack,
      PauseStack,
      UnpauseStack,
      StopStack,
      DestroyStack,
      RunStackService,
      TestAlerter,
      SendAlert,
      RemoveSwarmNodes,
      UpdateSwarmNode,
      RemoveSwarmStacks,
      RemoveSwarmServices,
      CreateSwarmConfig,
      RotateSwarmConfig,
      RemoveSwarmConfigs,
      CreateSwarmSecret,
      RotateSwarmSecret,
      RemoveSwarmSecrets,
      ClearRepoCache,
      BackupCoreDatabase,
      GlobalAutoUpdate,
      RotateAllServerKeys,
      RotateCoreKeys,
    ],
    batch: [
      BatchRunAction,
      BatchRunProcedure,
      BatchRunBuild,
      BatchDeploy,
      BatchDestroyDeployment,
      BatchCloneRepo,
      BatchPullRepo,
      BatchBuildRepo,
      BatchDeployStack,
      BatchDeployStackIfChanged,
      BatchPullStack,
      BatchDestroyStack,
    ],
  );

  match res {
    Ok(ExecutionResult::Single(update)) => {
      poll_update_until_complete(&update).await
    }
    Ok(ExecutionResult::Batch(updates)) => {
      let mut handles = updates
        .iter()
        .map(|update| async move {
          match update {
            BatchExecutionResponseItem::Ok(update) => {
              poll_update_until_complete(update).await
            }
            BatchExecutionResponseItem::Err(e) => {
              error!("{e:#?}");
              Ok(())
            }
          }
        })
        .collect::<FuturesUnordered<_>>();
      while let Some(res) = handles.next().await {
        match res {
          Ok(()) => {}
          Err(e) => {
            error!("{e:#?}");
          }
        }
      }
      Ok(())
    }
    Err(e) => {
      error!("{e:#?}");
      Ok(())
    }
  }
}

async fn poll_update_until_complete(
  update: &Update,
) -> anyhow::Result<()> {
  let link = if update.id.is_empty() {
    let (resource_type, id) = update.target.extract_variant_id();
    resource_link(&cli_config().host, resource_type, id)
  } else {
    format!("{}/updates/{}", cli_config().host, update.id)
  };
  println!("Link: '{}'", link.bold());

  let client = super::komodo_client().await?;

  let timer = tokio::time::Instant::now();
  let update = client.poll_update_until_complete(&update.id).await?;
  if update.success {
    println!(
      "FINISHED in {}: {}",
      format!("{:.1?}", timer.elapsed()).bold(),
      "EXECUTION SUCCESSFUL".green(),
    );
  } else {
    eprintln!(
      "FINISHED in {}: {}",
      format!("{:.1?}", timer.elapsed()).bold(),
      "EXECUTION FAILED".red(),
    );
  }
  Ok(())
}
