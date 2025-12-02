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
  match execution {
    Execution::None(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RunAction(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchRunAction(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RunProcedure(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchRunProcedure(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RunBuild(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchRunBuild(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::CancelBuild(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::Deploy(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchDeploy(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PullDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StartDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RestartDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PauseDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::UnpauseDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StopDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DestroyDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchDestroyDeployment(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::CloneRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchCloneRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PullRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchPullRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BuildRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchBuildRepo(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::CancelRepoBuild(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StartContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RestartContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PauseContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::UnpauseContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StopContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DestroyContainer(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StartAllContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RestartAllContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PauseAllContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::UnpauseAllContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StopAllContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneContainers(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DeleteNetwork(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneNetworks(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DeleteImage(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneImages(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DeleteVolume(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneVolumes(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneDockerBuilders(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneBuildx(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PruneSystem(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RunSync(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::CommitSync(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DeployStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchDeployStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DeployStackIfChanged(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchDeployStackIfChanged(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PullStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchPullStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StartStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RestartStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::PauseStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::UnpauseStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::StopStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::DestroyStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BatchDestroyStack(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RunStackService(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::TestAlerter(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::SendAlert(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RemoveSwarmNodes(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RemoveSwarmStacks(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RemoveSwarmServices(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RemoveSwarmConfigs(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RemoveSwarmSecrets(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::ClearRepoCache(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::BackupCoreDatabase(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::GlobalAutoUpdate(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RotateAllServerKeys(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::RotateCoreKeys(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
    Execution::Sleep(data) => {
      println!("{}: {data:?}", "Data".dimmed())
    }
  }

  super::wait_for_enter("run execution", yes)?;

  info!("Running Execution...");

  let client = super::komodo_client().await?;

  let res = match execution.clone() {
    Execution::RunAction(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchRunAction(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::RunProcedure(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchRunProcedure(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::RunBuild(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchRunBuild(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::CancelBuild(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::Deploy(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchDeploy(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::PullDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StartDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RestartDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PauseDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::UnpauseDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StopDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DestroyDeployment(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchDestroyDeployment(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::CloneRepo(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchCloneRepo(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::PullRepo(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchPullRepo(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::BuildRepo(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchBuildRepo(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::CancelRepoBuild(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StartContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RestartContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PauseContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::UnpauseContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StopContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DestroyContainer(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StartAllContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RestartAllContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PauseAllContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::UnpauseAllContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StopAllContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneContainers(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DeleteNetwork(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneNetworks(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DeleteImage(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneImages(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DeleteVolume(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneVolumes(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneDockerBuilders(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneBuildx(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PruneSystem(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RunSync(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::CommitSync(request) => client
      .write(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DeployStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchDeployStack(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::DeployStackIfChanged(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchDeployStackIfChanged(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::PullStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchPullStack(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::StartStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RestartStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::PauseStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::UnpauseStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::StopStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::DestroyStack(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BatchDestroyStack(request) => {
      client.execute(request).await.map(ExecutionResult::Batch)
    }
    Execution::RunStackService(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::TestAlerter(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::SendAlert(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RemoveSwarmNodes(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RemoveSwarmStacks(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RemoveSwarmServices(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RemoveSwarmConfigs(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RemoveSwarmSecrets(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::ClearRepoCache(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::BackupCoreDatabase(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::GlobalAutoUpdate(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RotateAllServerKeys(request) => client
      .execute(request)
      .await
      .map(|u| ExecutionResult::Single(u.into())),
    Execution::RotateCoreKeys(request) => client
      .execute(request)
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
  };

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
