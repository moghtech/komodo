use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use komodo_client::{
  busy::Busy,
  entities::{
    action::ActionActionState, build::BuildActionState,
    deployment::DeploymentActionState,
    procedure::ProcedureActionState, repo::RepoActionState,
    server::ServerActionState, stack::StackActionState,
    sync::ResourceSyncActionState,
  },
};

use super::cache::Cache;

#[derive(Default)]
pub struct ActionStates {
  pub build: Cache<String, Arc<ActionState<BuildActionState>>>,
  pub deployment:
    Cache<String, Arc<ActionState<DeploymentActionState>>>,
  pub server: Cache<String, Arc<ActionState<ServerActionState>>>,
  pub repo: Cache<String, Arc<ActionState<RepoActionState>>>,
  pub procedure:
    Cache<String, Arc<ActionState<ProcedureActionState>>>,
  pub action: Cache<String, Arc<ActionState<ActionActionState>>>,
  pub resource_sync:
    Cache<String, Arc<ActionState<ResourceSyncActionState>>>,
  pub stack: Cache<String, Arc<ActionState<StackActionState>>>,
}

/// Need to be able to check "busy" with write lock acquired.
#[derive(Default)]
pub struct ActionState<States: Default + Send + 'static>(
  Mutex<States>,
);

impl<States: Default + Busy + Copy + Send + 'static>
  ActionState<States>
{
  pub fn get(&self) -> anyhow::Result<States> {
    Ok(
      *self
        .0
        .lock()
        .map_err(|e| anyhow!("action state lock poisoned | {e:?}"))?,
    )
  }

  pub fn busy(&self) -> anyhow::Result<bool> {
    Ok(
      self
        .0
        .lock()
        .map_err(|e| anyhow!("action state lock poisoned | {e:?}"))?
        .busy(),
    )
  }

  /// Will acquire lock, check busy, and if not will
  /// run the provided update function on the states.
  /// Returns a guard that returns the states to default (not busy) when dropped.
  pub fn update(
    &self,
    handler: impl Fn(&mut States),
  ) -> anyhow::Result<UpdateGuard<States>> {
    let mut lock = self
      .0
      .lock()
      .map_err(|e| anyhow!("action state lock poisoned | {e:?}"))?;
    if lock.busy() {
      return Err(anyhow!("resource is busy"));
    }
    handler(&mut *lock);
    Ok(UpdateGuard(&self.0))
  }
}

/// When dropped will return the inner state to default.
/// The inner mutex guard must already be dropped BEFORE this is dropped,
/// which is guaranteed as the inner guard is dropped by all public methods before
/// user could drop UpdateGuard.
pub struct UpdateGuard<'a, States: Default + Send + 'static>(
  &'a Mutex<States>,
);

impl<States: Default + Send + 'static> Drop
  for UpdateGuard<'_, States>
{
  fn drop(&mut self) {
    let mut lock = match self.0.lock() {
      Ok(lock) => lock,
      Err(e) => {
        error!("CRITICAL: an action state lock is poisoned | {e:?}");
        return;
      }
    };
    *lock = States::default();
  }
}
