use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use command::run_komodo_command;
use database::mungos::{find::find_collect, mongodb::bson::doc};
use formatting::{bold, format_serror};
use komodo_client::{
  api::execute::{
    BackupCoreDatabase, ClearRepoCache, GlobalAutoUpdate,
  },
  entities::{
    deployment::DeploymentState, server::ServerState,
    stack::StackState,
  },
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;
use tokio::sync::Mutex;

use crate::{
  api::execute::{
    ExecuteArgs, pull_deployment_inner, pull_stack_inner,
  },
  config::core_config,
  helpers::update::update_update,
  state::{
    db_client, deployment_status_cache, server_status_cache,
    stack_status_cache,
  },
};

/// Makes sure the method can only be called once at a time
fn clear_repo_cache_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(Default::default)
}

impl Resolve<ExecuteArgs> for ClearRepoCache {
  #[instrument(
    name = "ClearRepoCache",
    skip(user, update),
    fields(user_id = user.id, update_id = update.id)
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    if !user.admin {
      return Err(
        anyhow!("This method is admin only.")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let _lock = clear_repo_cache_lock()
      .try_lock()
      .context("Clear already in progress...")?;

    let mut update = update.clone();

    let mut contents =
      tokio::fs::read_dir(&core_config().repo_directory)
        .await
        .context("Failed to read repo cache directory")?;

    loop {
      let path = match contents
        .next_entry()
        .await
        .context("Failed to read contents at path")
      {
        Ok(Some(contents)) => contents.path(),
        Ok(None) => break,
        Err(e) => {
          update.push_error_log(
            "Read Directory",
            format_serror(&e.into()),
          );
          continue;
        }
      };
      if path.is_dir() {
        match tokio::fs::remove_dir_all(&path)
          .await
          .context("Failed to clear contents at path")
        {
          Ok(_) => {}
          Err(e) => {
            update.push_error_log(
              "Clear Directory",
              format_serror(&e.into()),
            );
          }
        };
      }
    }

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

//

/// Makes sure the method can only be called once at a time
fn backup_database_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(Default::default)
}

impl Resolve<ExecuteArgs> for BackupCoreDatabase {
  #[instrument(
    name = "BackupCoreDatabase",
    skip(user, update),
    fields(user_id = user.id, update_id = update.id)
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    if !user.admin {
      return Err(
        anyhow!("This method is admin only.")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let _lock = backup_database_lock()
      .try_lock()
      .context("Backup already in progress...")?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    let res = run_komodo_command(
      "Backup Core Database",
      None,
      "km database backup --yes",
    )
    .await;

    update.logs.push(res);
    update.finalize();

    update_update(update.clone()).await?;

    Ok(update)
  }
}

//

/// Makes sure the method can only be called once at a time
fn global_update_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
  LOCK.get_or_init(Default::default)
}

impl Resolve<ExecuteArgs> for GlobalAutoUpdate {
  #[instrument(
    name = "GlobalAutoUpdate",
    skip(user, update),
    fields(user_id = user.id, update_id = update.id)
  )]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    if !user.admin {
      return Err(
        anyhow!("This method is admin only.")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let _lock = global_update_lock()
      .try_lock()
      .context("Global update already in progress...")?;

    let mut update = update.clone();

    update_update(update.clone()).await?;

    // This is all done in sequence because there is no rush,
    // the pulls / deploys happen spaced out to ease the load on system.
    let servers = find_collect(&db_client().servers, None, None)
      .await
      .context("Failed to query for servers from database")?;

    let query = doc! {
      "$or": [
        { "config.poll_for_updates": true },
        { "config.auto_update": true }
      ]
    };

    let (stacks, repos) = tokio::try_join!(
      find_collect(&db_client().stacks, query.clone(), None),
      find_collect(&db_client().repos, None, None)
    )
    .context("Failed to query for resources from database")?;

    let server_status_cache = server_status_cache();
    let stack_status_cache = stack_status_cache();

    // Will be edited later at update.logs[0]
    update.push_simple_log("Auto Pull", String::new());

    for stack in stacks {
      let Some(status) = stack_status_cache.get(&stack.id).await
      else {
        continue;
      };
      // Only pull running stacks.
      if !matches!(status.curr.state, StackState::Running) {
        continue;
      }
      if let Some(server) =
        servers.iter().find(|s| s.id == stack.config.server_id)
        // This check is probably redundant along with running check
        // but shouldn't hurt
        && server_status_cache
          .get(&server.id)
          .await
          .map(|s| matches!(s.state, ServerState::Ok))
          .unwrap_or_default()
      {
        let name = stack.name.clone();
        let repo = if stack.config.linked_repo.is_empty() {
          None
        } else {
          let Some(repo) =
            repos.iter().find(|r| r.id == stack.config.linked_repo)
          else {
            update.push_error_log(
              &format!("Pull Stack {name}"),
              format!(
                "Did not find any Repo matching {}",
                stack.config.linked_repo
              ),
            );
            continue;
          };
          Some(repo.clone())
        };
        if let Err(e) =
          pull_stack_inner(stack, Vec::new(), server, repo, None)
            .await
        {
          update.push_error_log(
            &format!("Pull Stack {name}"),
            format_serror(&e.into()),
          );
        } else {
          if !update.logs[0].stdout.is_empty() {
            update.logs[0].stdout.push('\n');
          }
          update.logs[0]
            .stdout
            .push_str(&format!("Pulled Stack {} ✅", bold(name)));
        }
      }
    }

    let deployment_status_cache = deployment_status_cache();
    let deployments =
      find_collect(&db_client().deployments, query, None)
        .await
        .context("Failed to query for deployments from database")?;
    for deployment in deployments {
      let Some(status) =
        deployment_status_cache.get(&deployment.id).await
      else {
        continue;
      };
      // Only pull running deployments.
      if !matches!(status.curr.state, DeploymentState::Running) {
        continue;
      }
      if let Some(server) =
        servers.iter().find(|s| s.id == deployment.config.server_id)
        // This check is probably redundant along with running check
        // but shouldn't hurt
        && server_status_cache
          .get(&server.id)
          .await
          .map(|s| matches!(s.state, ServerState::Ok))
          .unwrap_or_default()
      {
        let name = deployment.name.clone();
        if let Err(e) =
          pull_deployment_inner(deployment, server).await
        {
          update.push_error_log(
            &format!("Pull Deployment {name}"),
            format_serror(&e.into()),
          );
        } else {
          if !update.logs[0].stdout.is_empty() {
            update.logs[0].stdout.push('\n');
          }
          update.logs[0].stdout.push_str(&format!(
            "Pulled Deployment {} ✅",
            bold(name)
          ));
        }
      }
    }

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
