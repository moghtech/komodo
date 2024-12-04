use std::sync::OnceLock;

use anyhow::anyhow;
use komodo_client::{
  api::{
    execute::*,
    write::{RefreshResourceSyncPending, RefreshStackCache},
  },
  entities::{
    action::Action, build::Build, procedure::Procedure, repo::Repo,
    stack::Stack, sync::ResourceSync, user::git_webhook_user,
  },
};
use resolver_api::Resolve;
use serde::Deserialize;

use crate::{
  api::{
    execute::{ExecuteArgs, ExecuteRequest},
    write::WriteArgs,
  },
  helpers::update::init_execution_update,
};

use super::{ListenerLockCache, ANY_BRANCH};

// =======
//  BUILD
// =======

impl super::CustomSecret for Build {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn build_locks() -> &'static ListenerLockCache {
  static BUILD_LOCKS: OnceLock<ListenerLockCache> = OnceLock::new();
  BUILD_LOCKS.get_or_init(Default::default)
}

pub async fn handle_build_webhook<B: super::VerifyBranch>(
  build: Build,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through from action state busy.
  let lock = build_locks().get_or_insert_default(&build.id).await;
  let _lock = lock.lock().await;

  if !build.config.webhook_enabled {
    return Err(anyhow!("build does not have webhook enabled"));
  }

  B::verify_branch(&body, &build.config.branch)?;

  let user = git_webhook_user().to_owned();
  let req = ExecuteRequest::RunBuild(RunBuild { build: build.id });
  let update = init_execution_update(&req, &user).await?;
  let ExecuteRequest::RunBuild(req) = req else {
    unreachable!()
  };
  req
    .resolve(&ExecuteArgs { user, update })
    .await
    .map_err(|e| e.error)?;
  Ok(())
}

// ======
//  REPO
// ======

impl super::CustomSecret for Repo {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn repo_locks() -> &'static ListenerLockCache {
  static REPO_LOCKS: OnceLock<ListenerLockCache> = OnceLock::new();
  REPO_LOCKS.get_or_init(Default::default)
}

pub trait RepoExecution {
  async fn resolve(repo: Repo) -> anyhow::Result<()>;
}

impl RepoExecution for CloneRepo {
  async fn resolve(repo: Repo) -> anyhow::Result<()> {
    let user = git_webhook_user().to_owned();
    let req =
      crate::api::execute::ExecuteRequest::CloneRepo(CloneRepo {
        repo: repo.id,
      });
    let update = init_execution_update(&req, &user).await?;
    let crate::api::execute::ExecuteRequest::CloneRepo(req) = req
    else {
      unreachable!()
    };
    req
      .resolve(&ExecuteArgs { user, update })
      .await
      .map_err(|e| e.error)?;
    Ok(())
  }
}

impl RepoExecution for PullRepo {
  async fn resolve(repo: Repo) -> anyhow::Result<()> {
    let user = git_webhook_user().to_owned();
    let req =
      crate::api::execute::ExecuteRequest::PullRepo(PullRepo {
        repo: repo.id,
      });
    let update = init_execution_update(&req, &user).await?;
    let crate::api::execute::ExecuteRequest::PullRepo(req) = req
    else {
      unreachable!()
    };
    req
      .resolve(&ExecuteArgs { user, update })
      .await
      .map_err(|e| e.error)?;
    Ok(())
  }
}

impl RepoExecution for BuildRepo {
  async fn resolve(repo: Repo) -> anyhow::Result<()> {
    let user = git_webhook_user().to_owned();
    let req =
      crate::api::execute::ExecuteRequest::BuildRepo(BuildRepo {
        repo: repo.id,
      });
    let update = init_execution_update(&req, &user).await?;
    let crate::api::execute::ExecuteRequest::BuildRepo(req) = req
    else {
      unreachable!()
    };
    req
      .resolve(&ExecuteArgs { user, update })
      .await
      .map_err(|e| e.error)?;
    Ok(())
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoWebhookOption {
  Clone,
  Pull,
  Build,
}

pub async fn handle_repo_webhook<B: super::VerifyBranch>(
  option: RepoWebhookOption,
  repo: Repo,
  body: String,
) -> anyhow::Result<()> {
  match option {
    RepoWebhookOption::Clone => {
      handle_repo_webhook_inner::<B, CloneRepo>(repo, body).await
    }
    RepoWebhookOption::Pull => {
      handle_repo_webhook_inner::<B, PullRepo>(repo, body).await
    }
    RepoWebhookOption::Build => {
      handle_repo_webhook_inner::<B, BuildRepo>(repo, body).await
    }
  }
}

async fn handle_repo_webhook_inner<
  B: super::VerifyBranch,
  E: RepoExecution,
>(
  repo: Repo,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through from action state busy.
  let lock = repo_locks().get_or_insert_default(&repo.id).await;
  let _lock = lock.lock().await;

  if !repo.config.webhook_enabled {
    return Err(anyhow!("repo does not have webhook enabled"));
  }

  B::verify_branch(&body, &repo.config.branch)?;

  E::resolve(repo).await
}

// =======
//  STACK
// =======

impl super::CustomSecret for Stack {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn stack_locks() -> &'static ListenerLockCache {
  static STACK_LOCKS: OnceLock<ListenerLockCache> = OnceLock::new();
  STACK_LOCKS.get_or_init(Default::default)
}

pub trait StackExecution {
  async fn resolve(stack: Stack) -> serror::Result<()>;
}

impl StackExecution for RefreshStackCache {
  async fn resolve(stack: Stack) -> serror::Result<()> {
    RefreshStackCache { stack: stack.id }
      .resolve(&WriteArgs {
        user: git_webhook_user().to_owned(),
      })
      .await?;
    Ok(())
  }
}

impl StackExecution for DeployStack {
  async fn resolve(stack: Stack) -> serror::Result<()> {
    let user = git_webhook_user().to_owned();
    if stack.config.webhook_force_deploy {
      let req = ExecuteRequest::DeployStack(DeployStack {
        stack: stack.id,
        service: None,
        stop_time: None,
      });
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeployStack(req) = req else {
        unreachable!()
      };
      req
        .resolve(&ExecuteArgs { user, update })
        .await
        .map_err(|e| e.error)?;
    } else {
      let req =
        ExecuteRequest::DeployStackIfChanged(DeployStackIfChanged {
          stack: stack.id,
          stop_time: None,
        });
      let update = init_execution_update(&req, &user).await?;
      let ExecuteRequest::DeployStackIfChanged(req) = req else {
        unreachable!()
      };
      req
        .resolve(&ExecuteArgs { user, update })
        .await
        .map_err(|e| e.error)?;
    }

    Ok(())
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackWebhookOption {
  Refresh,
  Deploy,
}

pub async fn handle_stack_webhook<B: super::VerifyBranch>(
  option: StackWebhookOption,
  stack: Stack,
  body: String,
) -> anyhow::Result<()> {
  match option {
    StackWebhookOption::Refresh => {
      handle_stack_webhook_inner::<B, RefreshStackCache>(stack, body)
        .await
    }
    StackWebhookOption::Deploy => {
      handle_stack_webhook_inner::<B, DeployStack>(stack, body).await
    }
  }
}

pub async fn handle_stack_webhook_inner<
  B: super::VerifyBranch,
  E: StackExecution,
>(
  stack: Stack,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through, from "action state busy".
  let lock = stack_locks().get_or_insert_default(&stack.id).await;
  let _lock = lock.lock().await;

  if !stack.config.webhook_enabled {
    return Err(anyhow!("stack does not have webhook enabled"));
  }

  B::verify_branch(&body, &stack.config.branch)?;

  E::resolve(stack).await.map_err(|e| e.error)
}

// ======
//  SYNC
// ======

impl super::CustomSecret for ResourceSync {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn sync_locks() -> &'static ListenerLockCache {
  static SYNC_LOCKS: OnceLock<ListenerLockCache> = OnceLock::new();
  SYNC_LOCKS.get_or_init(Default::default)
}

pub trait SyncExecution {
  async fn resolve(sync: ResourceSync) -> anyhow::Result<()>;
}

impl SyncExecution for RefreshResourceSyncPending {
  async fn resolve(sync: ResourceSync) -> anyhow::Result<()> {
    RefreshResourceSyncPending { sync: sync.id }
      .resolve(&WriteArgs {
        user: git_webhook_user().to_owned(),
      })
      .await
      .map_err(|e| e.error)?;
    Ok(())
  }
}

impl SyncExecution for RunSync {
  async fn resolve(sync: ResourceSync) -> anyhow::Result<()> {
    let user = git_webhook_user().to_owned();
    let req = ExecuteRequest::RunSync(RunSync {
      sync: sync.id,
      resource_type: None,
      resources: None,
    });
    let update = init_execution_update(&req, &user).await?;
    let ExecuteRequest::RunSync(req) = req else {
      unreachable!()
    };
    req
      .resolve(&ExecuteArgs { user, update })
      .await
      .map_err(|e| e.error)?;
    Ok(())
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncWebhookOption {
  Refresh,
  Sync,
}

pub async fn handle_sync_webhook<B: super::VerifyBranch>(
  option: SyncWebhookOption,
  sync: ResourceSync,
  body: String,
) -> anyhow::Result<()> {
  match option {
    SyncWebhookOption::Refresh => {
      handle_sync_webhook_inner::<B, RefreshResourceSyncPending>(
        sync, body,
      )
      .await
    }
    SyncWebhookOption::Sync => {
      handle_sync_webhook_inner::<B, RunSync>(sync, body).await
    }
  }
}

async fn handle_sync_webhook_inner<
  B: super::VerifyBranch,
  E: SyncExecution,
>(
  sync: ResourceSync,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through from action state busy.
  let lock = sync_locks().get_or_insert_default(&sync.id).await;
  let _lock = lock.lock().await;

  if !sync.config.webhook_enabled {
    return Err(anyhow!("sync does not have webhook enabled"));
  }

  B::verify_branch(&body, &sync.config.branch)?;

  E::resolve(sync).await
}

// ===========
//  PROCEDURE
// ===========

impl super::CustomSecret for Procedure {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn procedure_locks() -> &'static ListenerLockCache {
  static PROCEDURE_LOCKS: OnceLock<ListenerLockCache> =
    OnceLock::new();
  PROCEDURE_LOCKS.get_or_init(Default::default)
}

pub async fn handle_procedure_webhook<B: super::VerifyBranch>(
  procedure: Procedure,
  target_branch: &str,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through from action state busy.
  let lock =
    procedure_locks().get_or_insert_default(&procedure.id).await;
  let _lock = lock.lock().await;

  if !procedure.config.webhook_enabled {
    return Err(anyhow!("procedure does not have webhook enabled"));
  }

  if target_branch != ANY_BRANCH {
    B::verify_branch(&body, target_branch)?;
  }

  let user = git_webhook_user().to_owned();
  let req = ExecuteRequest::RunProcedure(RunProcedure {
    procedure: procedure.id,
  });
  let update = init_execution_update(&req, &user).await?;
  let ExecuteRequest::RunProcedure(req) = req else {
    unreachable!()
  };
  req
    .resolve(&ExecuteArgs { user, update })
    .await
    .map_err(|e| e.error)?;
  Ok(())
}

// ========
//  ACTION
// ========

impl super::CustomSecret for Action {
  fn custom_secret(resource: &Self) -> &str {
    &resource.config.webhook_secret
  }
}

fn action_locks() -> &'static ListenerLockCache {
  static ACTION_LOCKS: OnceLock<ListenerLockCache> = OnceLock::new();
  ACTION_LOCKS.get_or_init(Default::default)
}

pub async fn handle_action_webhook<B: super::VerifyBranch>(
  action: Action,
  target_branch: &str,
  body: String,
) -> anyhow::Result<()> {
  // Acquire and hold lock to make a task queue for
  // subsequent listener calls on same resource.
  // It would fail if we let it go through from action state busy.
  let lock = action_locks().get_or_insert_default(&action.id).await;
  let _lock = lock.lock().await;

  if !action.config.webhook_enabled {
    return Err(anyhow!("action does not have webhook enabled"));
  }

  if target_branch != ANY_BRANCH {
    B::verify_branch(&body, target_branch)?;
  }

  let user = git_webhook_user().to_owned();
  let req =
    ExecuteRequest::RunAction(RunAction { action: action.id });
  let update = init_execution_update(&req, &user).await?;
  let ExecuteRequest::RunAction(req) = req else {
    unreachable!()
  };
  req
    .resolve(&ExecuteArgs { user, update })
    .await
    .map_err(|e| e.error)?;
  Ok(())
}
