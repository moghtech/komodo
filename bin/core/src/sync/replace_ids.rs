use komodo_client::entities::{
  action::Action,
  alerter::Alerter,
  build::Build,
  builder::{Builder, BuilderConfig},
  deployment::{Deployment, DeploymentImage},
  procedure::Procedure,
  repo::Repo,
  server::Server,
  stack::Stack,
  swarm::Swarm,
  sync::ResourceSync,
};

use crate::{
  helpers::procedure::replace_procedure_stage_ids_with_names,
  resource::KomodoResource, state::all_resources_cache,
};

/// Trait implementing replacing of linked resource ids with names for Resource Sync TOML.
pub trait ReplaceIds: KomodoResource {
  /// Replace linked ids (server_id, build_id, etc) with the resource name.
  fn replace_ids(_config: &mut Self::Config) {}
}

// These have no linked resource ids to replace
impl ReplaceIds for Server {}
impl ReplaceIds for Action {}

impl ReplaceIds for ResourceSync {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();
    config.linked_repo.clone_from(
      all
        .repos
        .get(&config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }
}

impl ReplaceIds for Swarm {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();

    config.server_ids.iter_mut().for_each(|server_id| {
      *server_id = all
        .servers
        .get(server_id)
        .map(|s| s.name.clone())
        .unwrap_or_default();
    });
    let mut res = Vec::with_capacity(config.server_ids.capacity());
    for server_id in &config.server_ids {
      res.push(
        all
          .servers
          .get(server_id)
          .map(|s| s.name.clone())
          .unwrap_or_default(),
      );
    }
  }
}

impl ReplaceIds for Stack {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();

    config.swarm_id.clone_from(
      all
        .swarms
        .get(&config.swarm_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );

    config.server_id.clone_from(
      all
        .servers
        .get(&config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );

    config.linked_repo.clone_from(
      all
        .repos
        .get(&config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }
}

impl ReplaceIds for Deployment {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();

    config.swarm_id.clone_from(
      all
        .swarms
        .get(&config.swarm_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );

    config.server_id.clone_from(
      all
        .servers
        .get(&config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );

    if let DeploymentImage::Build { build_id, .. } = &mut config.image
    {
      build_id.clone_from(
        all
          .builds
          .get(build_id)
          .map(|b| &b.name)
          .unwrap_or(&String::new()),
      );
    }
  }
}

impl ReplaceIds for Build {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();
    config.builder_id.clone_from(
      all
        .builders
        .get(&config.builder_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    config.linked_repo.clone_from(
      all
        .repos
        .get(&config.linked_repo)
        .map(|r| &r.name)
        .unwrap_or(&String::new()),
    );
  }
}

impl ReplaceIds for Repo {
  fn replace_ids(config: &mut Self::Config) {
    let all = all_resources_cache().load();
    config.server_id.clone_from(
      all
        .servers
        .get(&config.server_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
    config.builder_id.clone_from(
      all
        .builders
        .get(&config.builder_id)
        .map(|s| &s.name)
        .unwrap_or(&String::new()),
    );
  }
}

impl ReplaceIds for Builder {
  fn replace_ids(config: &mut Self::Config) {
    if let BuilderConfig::Server(config) = config {
      let all = all_resources_cache().load();
      for server_id in &mut config.server_ids {
        *server_id = all
          .servers
          .get(server_id)
          .map(|s| s.name.clone())
          .unwrap_or_default();
      }
    }
  }
}

impl ReplaceIds for Procedure {
  fn replace_ids(config: &mut Self::Config) {
    replace_procedure_stage_ids_with_names(&mut config.stages);
  }
}

/// Replaces the inner ids of [ResourceTarget] variants with the
/// referenced resource's name, looking each up in [AllResourcesById].
/// The `System` variant carries no id and is a no-op.
macro_rules! replace_resource_target_ids {
  ($target:expr, $all:expr, { $( $variant:ident => $field:ident ),* $(,)? }) => {
    match $target {
      komodo_client::entities::ResourceTarget::System(_) => {}
      $(
        komodo_client::entities::ResourceTarget::$variant(id) => {
          *id = $all
            .$field
            .get(id)
            .map(|r| r.name.clone())
            .unwrap_or_default();
        }
      )*
    }
  };
}

impl ReplaceIds for Alerter {
  fn replace_ids(config: &mut Self::Config) {
    if config.resources.is_empty()
      && config.except_resources.is_empty()
    {
      return;
    }

    let all = all_resources_cache().load();

    for resource in &mut config.resources {
      replace_resource_target_ids!(resource, all, {
        Swarm => swarms,
        Server => servers,
        Stack => stacks,
        Deployment => deployments,
        Build => builds,
        Repo => repos,
        Procedure => procedures,
        Action => actions,
        Builder => builders,
        Alerter => alerters,
        ResourceSync => syncs,
      });
    }
    for resource in &mut config.except_resources {
      replace_resource_target_ids!(resource, all, {
        Swarm => swarms,
        Server => servers,
        Stack => stacks,
        Deployment => deployments,
        Build => builds,
        Repo => repos,
        Procedure => procedures,
        Action => actions,
        Builder => builders,
        Alerter => alerters,
        ResourceSync => syncs,
      });
    }
  }
}
