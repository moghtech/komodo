use std::collections::HashMap;

use formatting::{Color, bold, colored, muted};
use komodo_client::entities::{
  ResourceTargetVariant,
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
  tag::Tag,
  update::Log,
  user::sync_user,
};
use partial_derive2::{MaybeNone, PartialDiff};

use crate::{
  helpers::procedure::replace_procedure_stage_ids_with_names,
  resource::{KomodoResource, ResourceMetaUpdate},
  state::all_resources_cache,
  sync::{ToUpdateItem, execute::run_update_meta},
};

use super::{
  ResourceSyncTrait, SyncDeltas, execute::ExecuteResourceSync,
  include_resource_by_resource_type_and_name,
  include_resource_by_tags,
};

impl ResourceSyncTrait for Server {
  fn get_diff(
    original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Server {}

impl ResourceSyncTrait for Swarm {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();

    original.server_ids.iter_mut().for_each(|server_id| {
      *server_id = all
        .servers
        .get(server_id)
        .map(|s| s.name.clone())
        .unwrap_or_default();
    });

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Swarm {}

impl ResourceSyncTrait for Deployment {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();

    original.swarm_id = all
      .swarms
      .get(&original.swarm_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    original.server_id = all
      .servers
      .get(&original.server_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    if let DeploymentImage::Build { build_id, version } =
      &original.image
    {
      original.image = DeploymentImage::Build {
        build_id: all
          .builds
          .get(build_id)
          .map(|b| b.name.clone())
          .unwrap_or_default(),
        version: *version,
      };
    }

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Deployment {}

impl ResourceSyncTrait for Stack {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();

    original.swarm_id = all
      .swarms
      .get(&original.swarm_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    original.server_id = all
      .servers
      .get(&original.server_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    original.linked_repo = all
      .repos
      .get(&original.linked_repo)
      .map(|r| r.name.clone())
      .unwrap_or_default();

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Stack {}

impl ResourceSyncTrait for Build {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();
    original.builder_id = all
      .builders
      .get(&original.builder_id)
      .map(|b| b.name.clone())
      .unwrap_or_default();
    original.linked_repo = all
      .repos
      .get(&original.linked_repo)
      .map(|r| r.name.clone())
      .unwrap_or_default();

    Ok(original.partial_diff(update))
  }

  fn validate_diff(diff: &mut Self::ConfigDiff) {
    if let Some((_, to)) = &diff.version {
      // When setting a build back to "latest" version,
      // Don't actually set version to None.
      // You can do this on the db, or set it to 0.0.1
      if to.is_none() {
        diff.version = None;
      }
    }
  }
}

impl ExecuteResourceSync for Build {}

impl ResourceSyncTrait for Repo {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();
    // Need to replace server id with name
    original.server_id = all
      .servers
      .get(&original.server_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    // Need to replace builder id with name
    original.builder_id = all
      .builders
      .get(&original.builder_id)
      .map(|s| s.name.clone())
      .unwrap_or_default();

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Repo {}

impl ResourceSyncTrait for Alerter {
  fn get_diff(
    original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Alerter {}

impl ResourceSyncTrait for Builder {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    // need to replace server builder id with name
    if let BuilderConfig::Server(config) = &mut original {
      let all = all_resources_cache().load();
      config.server_id = all
        .servers
        .get(&config.server_id)
        .map(|s| s.name.clone())
        .unwrap_or_default();
    }

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Builder {}

impl ResourceSyncTrait for Action {
  fn get_diff(
    original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Action {}

impl ResourceSyncTrait for ResourceSync {
  fn include_resource(
    name: &String,
    config: &Self::Config,
    match_resource_type: Option<ResourceTargetVariant>,
    match_resources: Option<&[String]>,
    resource_tags: &[String],
    id_to_tags: &HashMap<String, Tag>,
    match_tags: &[String],
  ) -> bool {
    if !include_resource_by_resource_type_and_name::<ResourceSync>(
      match_resource_type,
      match_resources,
      name,
    ) || !include_resource_by_tags(
      resource_tags,
      id_to_tags,
      match_tags,
    ) {
      return false;
    }
    // don't include fresh sync
    let contents_empty = config.file_contents.is_empty();
    if contents_empty
      && !config.files_on_host
      && config.repo.is_empty()
      && config.linked_repo.is_empty()
    {
      return false;
    }
    // The file contents MUST be empty
    contents_empty &&
    // The sync must be files on host mode OR git repo mode
    (config.files_on_host || !config.repo.is_empty() || !config.linked_repo.is_empty())
  }

  fn include_resource_partial(
    name: &String,
    config: &Self::PartialConfig,
    match_resource_type: Option<ResourceTargetVariant>,
    match_resources: Option<&[String]>,
    resource_tags: &[String],
    id_to_tags: &HashMap<String, Tag>,
    match_tags: &[String],
  ) -> bool {
    if !include_resource_by_resource_type_and_name::<ResourceSync>(
      match_resource_type,
      match_resources,
      name,
    ) || !include_resource_by_tags(
      resource_tags,
      id_to_tags,
      match_tags,
    ) {
      return false;
    }
    // don't include fresh sync
    let contents_empty = config
      .file_contents
      .as_ref()
      .map(String::is_empty)
      .unwrap_or(true);
    let files_on_host = config.files_on_host.unwrap_or_default();
    if contents_empty
      && !files_on_host
      && config.repo.as_ref().map(String::is_empty).unwrap_or(true)
      && config
        .linked_repo
        .as_ref()
        .map(String::is_empty)
        .unwrap_or(true)
    {
      return false;
    }
    // The file contents MUST be empty
    contents_empty &&
    // The sync must be files on host mode OR git repo mode
    (files_on_host || !config.repo.as_deref().unwrap_or_default().is_empty() || !config.linked_repo.as_deref().unwrap_or_default().is_empty())
  }

  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    let all = all_resources_cache().load();
    original.linked_repo = all
      .repos
      .get(&original.linked_repo)
      .map(|r| r.name.clone())
      .unwrap_or_default();

    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for ResourceSync {}

impl ResourceSyncTrait for Procedure {
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    replace_procedure_stage_ids_with_names(&mut original.stages);
    Ok(original.partial_diff(update))
  }
}

impl ExecuteResourceSync for Procedure {
  async fn execute_sync_updates(
    SyncDeltas {
      mut to_create,
      mut to_update,
      to_delete,
    }: SyncDeltas<Self::PartialConfig>,
  ) -> Option<Log> {
    if to_create.is_empty()
      && to_update.is_empty()
      && to_delete.is_empty()
    {
      return None;
    }

    let mut has_error = false;
    let mut log =
      format!("running updates on {}s", Self::resource_type());

    for name in to_delete {
      if let Err(e) =
        crate::resource::delete::<Procedure>(&name, sync_user()).await
      {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to delete {} '{}' | {e:#}",
          colored("ERROR", Color::Red),
          Self::resource_type(),
          bold(&name),
        ))
      } else {
        log.push_str(&format!(
          "\n{}: {} {} '{}'",
          muted("INFO"),
          colored("deleted", Color::Red),
          Self::resource_type(),
          bold(&name)
        ));
      }
    }

    if to_update.is_empty() && to_create.is_empty() {
      let stage = "Update Procedures";
      return Some(if has_error {
        Log::error(stage, log)
      } else {
        Log::simple(stage, log)
      });
    }

    for i in 0..10 {
      let mut to_pull = Vec::new();
      for ToUpdateItem {
        id,
        resource,
        update_description,
        update_template,
        update_tags,
      } in &to_update
      {
        let name = resource.name.clone();

        let meta = ResourceMetaUpdate {
          description: update_description
            .then(|| resource.description.clone()),
          template: update_template.then(|| resource.template),
          tags: update_tags.then(|| resource.tags.clone()),
        };

        if !meta.is_none() {
          run_update_meta::<Procedure>(
            id.clone(),
            &name,
            meta,
            &mut log,
            &mut has_error,
          )
          .await;
        }
        if !resource.config.is_none()
          && let Err(e) = crate::resource::update::<Procedure>(
            id,
            resource.config.clone(),
            sync_user(),
          )
          .await
        {
          if i == 9 {
            has_error = true;
            log.push_str(&format!(
              "\n{}: failed to update {} '{}' | {e:#}",
              colored("ERROR", Color::Red),
              Self::resource_type(),
              bold(&name)
            ));
          }
          continue;
        }

        log.push_str(&format!(
          "\n{}: {} '{}' updated",
          muted("INFO"),
          Self::resource_type(),
          bold(&name)
        ));
        // have to clone out so to_update is mutable
        to_pull.push(id.clone());
      }
      //
      to_update.retain(|resource| !to_pull.contains(&resource.id));

      let mut to_pull = Vec::new();
      for resource in &to_create {
        let name = resource.name.clone();
        let id = match crate::resource::create::<Procedure>(
          &name,
          resource.config.clone(),
          None,
          sync_user(),
        )
        .await
        .map_err(|e| e.error)
        {
          Ok(resource) => resource.id,
          Err(e) => {
            if i == 9 {
              has_error = true;
              log.push_str(&format!(
                "\n{}: failed to create {} '{}' | {e:#}",
                colored("ERROR", Color::Red),
                Self::resource_type(),
                bold(&name)
              ));
            }
            continue;
          }
        };
        run_update_meta::<Procedure>(
          id.clone(),
          &name,
          ResourceMetaUpdate {
            description: Some(resource.description.clone()),
            template: Some(resource.template),
            tags: Some(resource.tags.clone()),
          },
          &mut log,
          &mut has_error,
        )
        .await;
        log.push_str(&format!(
          "\n{}: {} {} '{}'",
          muted("INFO"),
          colored("created", Color::Green),
          Self::resource_type(),
          bold(&name)
        ));
        to_pull.push(name);
      }
      to_create.retain(|resource| !to_pull.contains(&resource.name));

      if to_update.is_empty() && to_create.is_empty() {
        let stage = "Update Procedures";
        return Some(if has_error {
          Log::error(stage, log)
        } else {
          Log::simple(stage, log)
        });
      }
    }
    warn!("procedure sync loop exited after max iterations");

    Some(Log::error(
      "run procedure",
      String::from("procedure sync loop exited after max iterations"),
    ))
  }
}
