//! # Action State Management
//!
//! This module provides thread-safe state management for resource exections.
//! It prevents concurrent execution of exections on the same resource using
//! a Mutex-based locking mechanism with RAII guards.
//!
//! ## Safety
//!
//! - Uses RAII pattern to ensure locks are always released
//! - Handles lock poisoning gracefully
//! - Prevents race conditions through per-resource locks
//! - No deadlock risk: each resource has independent locks

use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use mogh_cache::CloneCache;
use komodo_client::{
  busy::Busy,
  entities::{
    action::ActionActionState, build::BuildActionState,
    deployment::DeploymentActionState,
    procedure::ProcedureActionState, repo::RepoActionState,
    server::ServerActionState, stack::StackActionState,
    swarm::SwarmActionState, sync::ResourceSyncActionState,
  },
};

#[derive(Default)]
pub struct ActionStates {
  pub swarm: CloneCache<String, Arc<ActionState<SwarmActionState>>>,
  pub server: CloneCache<String, Arc<ActionState<ServerActionState>>>,
  pub stack: CloneCache<String, Arc<ActionState<StackActionState>>>,
  pub deployment:
    CloneCache<String, Arc<ActionState<DeploymentActionState>>>,
  pub build: CloneCache<String, Arc<ActionState<BuildActionState>>>,
  pub repo: CloneCache<String, Arc<ActionState<RepoActionState>>>,
  pub procedure:
    CloneCache<String, Arc<ActionState<ProcedureActionState>>>,
  pub action: CloneCache<String, Arc<ActionState<ActionActionState>>>,
  pub sync:
    CloneCache<String, Arc<ActionState<ResourceSyncActionState>>>,
}

/// Thread-safe state container for resource executions.
///
/// Uses a Mutex to prevent concurrent executions and provides
/// RAII-based locking through [UpdateGuard].
///
/// # Safety
///
/// - Each resource has its own ActionState instance
/// - State is reset to default when [UpdateGuard] is dropped
/// - Lock poisoning error handling is handled gracefully with anyhow::Error
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
        .map_err(|e| anyhow!("Action state lock poisoned | {e:?}"))?,
    )
  }

  pub fn busy(&self) -> anyhow::Result<bool> {
    Ok(
      self
        .0
        .lock()
        .map_err(|e| anyhow!("Action state lock poisoned | {e:?}"))?
        .busy(),
    )
  }

  /// Acquires lock, checks if resource is busy, and if not,
  /// runs the provided update function on the states.
  ///
  /// Returns an `UpdateGuard` that automatically resets the state
  /// to default (not busy) when dropped.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - The lock is poisoned
  /// - The resource is currently busy
  ///
  /// # Example
  ///
  /// ```rust
  /// let guard = action_state.update(|state| {
  ///   *state = SomeNewState;
  /// })?;
  /// // State is locked and marked as busy
  /// // ... perform work ...
  /// drop(guard) // Guard is dropped, state returns to default
  /// ```
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

/// RAII guard that automatically resets the action state when dropped.
///
/// # Safety
///
/// The inner mutex guard is guaranteed to be dropped before this guard
/// is dropped, preventing deadlocks. This is ensured by all public methods
/// that create UpdateGuard instances.
///
/// # Behavior
///
/// When dropped, this guard will:
/// 1. Re-acquire the lock
/// 2. Call the provided return function (typically resetting to default)
/// 3. Release the lock
///
/// If the lock is poisoned, an error is logged but the drop continues.
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
