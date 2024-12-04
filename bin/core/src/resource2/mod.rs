use komodo_client::entities::{
  action::Action, build::Build, builder::Builder,
  deployment::Deployment, procedure::Procedure, repo::Repo,
  server::Server, server_template::ServerTemplate, stack::Stack,
  sync::ResourceSync, ResourceTarget, ResourceTargetVariant,
};

pub mod crud;
pub mod query;

pub trait ResourceBase {
  fn resource_type() -> ResourceTargetVariant;
}

pub fn resource_target<R: ResourceBase>(
  id: String,
) -> ResourceTarget {
  match R::resource_type() {
    ResourceTargetVariant::System => ResourceTarget::System(id),
    ResourceTargetVariant::Build => ResourceTarget::Build(id),
    ResourceTargetVariant::Builder => ResourceTarget::Builder(id),
    ResourceTargetVariant::Deployment => {
      ResourceTarget::Deployment(id)
    }
    ResourceTargetVariant::Server => ResourceTarget::Server(id),
    ResourceTargetVariant::Repo => ResourceTarget::Repo(id),
    ResourceTargetVariant::Alerter => ResourceTarget::Alerter(id),
    ResourceTargetVariant::Procedure => ResourceTarget::Procedure(id),
    ResourceTargetVariant::ServerTemplate => {
      ResourceTarget::ServerTemplate(id)
    }
    ResourceTargetVariant::ResourceSync => {
      ResourceTarget::ResourceSync(id)
    }
    ResourceTargetVariant::Stack => ResourceTarget::Stack(id),
    ResourceTargetVariant::Action => ResourceTarget::Action(id),
  }
}

// =================
//  IMPLEMENTATIONS
// =================

impl ResourceBase for Server {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Server
  }
}

impl ResourceBase for Stack {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Stack
  }
}

impl ResourceBase for Deployment {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Deployment
  }
}

impl ResourceBase for Build {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Build
  }
}

impl ResourceBase for Repo {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Repo
  }
}

impl ResourceBase for Procedure {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Procedure
  }
}

impl ResourceBase for Action {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Action
  }
}

impl ResourceBase for Builder {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Builder
  }
}

impl ResourceBase for ServerTemplate {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::ServerTemplate
  }
}

impl ResourceBase for ResourceSync {
  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::ResourceSync
  }
}
