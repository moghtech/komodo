use crate::entities::{
  action::ActionActionState, build::BuildActionState,
  deployment::DeploymentActionState, procedure::ProcedureActionState,
  repo::RepoActionState, server::ServerActionState,
  stack::StackActionState, swarm::SwarmActionState,
  sync::ResourceSyncActionState,
};

pub trait Busy {
  fn busy(&self) -> bool;
}

impl Busy for SwarmActionState {
  fn busy(&self) -> bool {
    false
  }
}

impl Busy for ServerActionState {
  fn busy(&self) -> bool {
    self.pruning_containers
      || self.pruning_images
      || self.pruning_networks
      || self.pruning_volumes
      || self.starting_containers
      || self.restarting_containers
      || self.pausing_containers
      || self.unpausing_containers
      || self.stopping_containers
  }
}

impl Busy for DeploymentActionState {
  fn busy(&self) -> bool {
    self.deploying
      || self.starting
      || self.restarting
      || self.pausing
      || self.unpausing
      || self.stopping
      || self.destroying
      || self.renaming
  }
}

impl Busy for StackActionState {
  fn busy(&self) -> bool {
    self.deploying
      || self.starting
      || self.restarting
      || self.pausing
      || self.unpausing
      || self.stopping
      || self.destroying
  }
}

impl Busy for BuildActionState {
  fn busy(&self) -> bool {
    self.building
  }
}

impl Busy for RepoActionState {
  fn busy(&self) -> bool {
    self.cloning || self.pulling || self.building
  }
}

impl Busy for ProcedureActionState {
  fn busy(&self) -> bool {
    self.running
  }
}

impl Busy for ActionActionState {
  fn busy(&self) -> bool {
    self.running > 0
  }
}

impl Busy for ResourceSyncActionState {
  fn busy(&self) -> bool {
    self.syncing
  }
}
