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
  pub server: Cache<String, Arc<ActionState<ServerActionState>>>,
  pub stack: Cache<String, Arc<ActionState<StackActionState>>>,
  pub deployment:
    Cache<String, Arc<ActionState<DeploymentActionState>>>,
  pub build: Cache<String, Arc<ActionState<BuildActionState>>>,
  pub repo: Cache<String, Arc<ActionState<RepoActionState>>>,
  pub procedure:
    Cache<String, Arc<ActionState<ProcedureActionState>>>,
  pub action: Cache<String, Arc<ActionState<ActionActionState>>>,
  pub sync: Cache<String, Arc<ActionState<ResourceSyncActionState>>>,
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
    update_fn: impl Fn(&mut States),
  ) -> anyhow::Result<UpdateGuard<'_, States>> {
    self.update_custom(
      update_fn,
      |states| *states = Default::default(),
      true,
    )
  }

  /// Will acquire lock, optionally check busy, and if not will
  /// run the provided update function on the states.
  /// Returns a guard that calls the provided return_fn when dropped.
  pub fn update_custom(
    &self,
    update_fn: impl Fn(&mut States),
    return_fn: impl Fn(&mut States) + Send + 'static,
    busy_check: bool,
  ) -> anyhow::Result<UpdateGuard<'_, States>> {
    let mut lock = self
      .0
      .lock()
      .map_err(|e| anyhow!("Action state lock poisoned | {e:?}"))?;
    if busy_check && lock.busy() {
      return Err(anyhow!("Resource is busy"));
    }
    update_fn(&mut *lock);
    Ok(UpdateGuard(&self.0, Box::new(return_fn)))
  }
}

/// When dropped will return the inner state to default.
/// The inner mutex guard must already be dropped BEFORE this is dropped,
/// which is guaranteed as the inner guard is dropped by all public methods before
/// user could drop UpdateGuard.
pub struct UpdateGuard<'a, States: Default + Send + 'static>(
  &'a Mutex<States>,
  Box<dyn Fn(&mut States) + Send>,
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
    self.1(&mut *lock);
  }
}
