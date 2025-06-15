use std::time::Duration;

use async_timing_util::{Timelength, get_timelength_in_ms};
use komodo_client::{
  api::write::{
    RefreshBuildCache, RefreshRepoCache, RefreshResourceSyncPending,
    RefreshStackCache,
  },
  entities::user::{build_user, repo_user, stack_user, sync_user},
};
use mungos::find::find_collect;
use resolver_api::Resolve;

use crate::{
  api::{execute::pull_deployment_inner, write::WriteArgs},
  config::core_config,
  helpers::all_resources::AllResourcesById,
  state::{all_resources_cache, db_client},
};

pub fn spawn_all_resources_refresh_loop() {
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    loop {
      interval.tick().await;
      refresh_all_resources_cache().await;
    }
  });
}

pub async fn refresh_all_resources_cache() {
  let all = match AllResourcesById::load().await {
    Ok(all) => all,
    Err(e) => {
      error!("Failed to load all resources by id cache | {e:#}");
      return;
    }
  };
  all_resources_cache().store(all.into());
}

pub fn spawn_resource_refresh_loop() {
  let interval: Timelength = core_config()
    .resource_poll_interval
    .try_into()
    .expect("Invalid resource poll interval");
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(
      get_timelength_in_ms(interval) as u64,
    ));
    loop {
      interval.tick().await;
      refresh_all().await;
    }
  });
}

async fn refresh_all() {
  refresh_stacks().await;
  refresh_deployments().await;
  refresh_builds().await;
  refresh_repos().await;
  refresh_syncs().await;
}

async fn refresh_stacks() {
  let Ok(stacks) = find_collect(&db_client().stacks, None, None)
    .await
    .inspect_err(|e| {
      warn!(
        "Failed to get Stacks from database in refresh task | {e:#}"
      )
    })
  else {
    return;
  };
  for stack in stacks {
    RefreshStackCache { stack: stack.id }
      .resolve(
        &WriteArgs { user: stack_user().clone() },
      )
      .await
      .inspect_err(|e| {
        warn!("Failed to refresh Stack cache in refresh task | Stack: {} | {:#}", stack.name, e.error)
      })
      .ok();
  }
}

async fn refresh_deployments() {
  let servers = find_collect(&db_client().servers, None, None)
    .await
    .inspect_err(|e| {
      warn!(
        "Failed to get Servers from database in refresh task | {e:#}"
      )
    })
    .unwrap_or_default();
  let Ok(deployments) = find_collect(&db_client().deployments, None, None)
    .await
    .inspect_err(|e| {
      warn!(
        "Failed to get Deployments from database in refresh task | {e:#}"
      )
    })
  else {
    return;
  };
  for deployment in deployments {
    if deployment.config.poll_for_updates
      || deployment.config.auto_update
    {
      if let Some(server) =
        servers.iter().find(|s| s.id == deployment.config.server_id)
      {
        let name = deployment.name.clone();
        if let Err(e) =
          pull_deployment_inner(deployment, server).await
        {
          warn!(
            "Failed to pull latest image for Deployment {name} | {e:#}"
          );
        }
      }
    }
  }
}

async fn refresh_builds() {
  let Ok(builds) = find_collect(&db_client().builds, None, None)
    .await
    .inspect_err(|e| {
      warn!(
        "Failed to get Builds from database in refresh task | {e:#}"
      )
    })
  else {
    return;
  };
  for build in builds {
    RefreshBuildCache { build: build.id }
      .resolve(
        &WriteArgs { user: build_user().clone() },
      )
      .await
      .inspect_err(|e| {
        warn!("Failed to refresh Build cache in refresh task | Build: {} | {:#}", build.name, e.error)
      })
      .ok();
  }
}

async fn refresh_repos() {
  let Ok(repos) = find_collect(&db_client().repos, None, None)
    .await
    .inspect_err(|e| {
      warn!(
        "Failed to get Repos from database in refresh task | {e:#}"
      )
    })
  else {
    return;
  };
  for repo in repos {
    RefreshRepoCache { repo: repo.id }
      .resolve(
        &WriteArgs { user: repo_user().clone() },
      )
      .await
      .inspect_err(|e| {
        warn!("Failed to refresh Repo cache in refresh task | Repo: {} | {:#}", repo.name, e.error)
      })
      .ok();
  }
}

async fn refresh_syncs() {
  let Ok(syncs) = find_collect(
    &db_client().resource_syncs,
    None,
    None,
  )
  .await
  .inspect_err(|e| {
    warn!(
      "failed to get resource syncs from db in refresh task | {e:#}"
    )
  }) else {
    return;
  };
  for sync in syncs {
    RefreshResourceSyncPending { sync: sync.id }
      .resolve(
        &WriteArgs { user: sync_user().clone() },
      )
      .await
      .inspect_err(|e| {
        warn!("Failed to refresh ResourceSync in refresh task | Sync: {} | {:#}", sync.name, e.error)
      })
      .ok();
  }
}
