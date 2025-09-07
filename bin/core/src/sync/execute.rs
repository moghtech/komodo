use std::collections::HashMap;

use anyhow::Context;
use database::mungos::find::find_collect;
use formatting::{Color, bold, colored, muted};
use komodo_client::entities::{
  ResourceTargetVariant, tag::Tag, toml::ResourceToml, update::Log,
  user::sync_user,
};
use partial_derive2::MaybeNone;

use crate::{api::write::WriteArgs, resource::ResourceMetaUpdate};

use super::{ResourceSyncTrait, SyncDeltas, ToUpdateItem};

/// Gets all the resources to update. For use in sync execution.
pub async fn get_updates_for_execution<
  Resource: ResourceSyncTrait,
>(
  resources: Vec<ResourceToml<Resource::PartialConfig>>,
  delete: bool,
  match_resource_type: Option<ResourceTargetVariant>,
  match_resources: Option<&[String]>,
  id_to_tags: &HashMap<String, Tag>,
  match_tags: &[String],
) -> anyhow::Result<SyncDeltas<Resource::PartialConfig>> {
  let map = find_collect(Resource::coll(), None, None)
    .await
    .context("failed to get resources from db")?
    .into_iter()
    .filter(|r| {
      Resource::include_resource(
        &r.name,
        &r.config,
        match_resource_type,
        match_resources,
        &r.tags,
        id_to_tags,
        match_tags,
      )
    })
    .map(|r| (r.name.clone(), r))
    .collect::<HashMap<_, _>>();
  let resources = resources
    .into_iter()
    .filter(|r| {
      Resource::include_resource_partial(
        &r.name,
        &r.config,
        match_resource_type,
        match_resources,
        &r.tags,
        id_to_tags,
        match_tags,
      )
    })
    .collect::<Vec<_>>();

  let mut deltas = SyncDeltas::<Resource::PartialConfig>::default();

  if delete {
    for resource in map.values() {
      if !resources.iter().any(|r| r.name == resource.name) {
        deltas.to_delete.push(resource.name.clone());
      }
    }
  }

  for mut resource in resources {
    match map.get(&resource.name) {
      Some(original) => {
        // First merge toml resource config (partial) onto default resource config.
        // Makes sure things that aren't defined in toml (come through as None) actually get removed.
        let config: Resource::Config = resource.config.into();
        resource.config = config.into();

        Resource::validate_partial_config(&mut resource.config);

        let mut diff = Resource::get_diff(
          original.config.clone(),
          resource.config,
        )?;

        Resource::validate_diff(&mut diff);

        let original_tags = original
          .tags
          .iter()
          .filter_map(|id| id_to_tags.get(id).map(|t| t.name.clone()))
          .collect::<Vec<_>>();

        // Only proceed if there are any fields to update,
        // or a change to tags / description
        if diff.is_none()
          && resource.description == original.description
          && resource.template == original.template
          && resource.tags == original_tags
        {
          continue;
        }

        // Minimizes updates through diffing.
        resource.config = diff.into();

        let update = ToUpdateItem {
          id: original.id.clone(),
          update_description: resource.description
            != original.description,
          update_template: resource.template != original.template,
          update_tags: resource.tags != original_tags,
          resource,
        };

        deltas.to_update.push(update);
      }
      None => deltas.to_create.push(resource),
    }
  }

  Ok(deltas)
}

pub trait ExecuteResourceSync: ResourceSyncTrait {
  async fn execute_sync_updates(
    SyncDeltas {
      to_create,
      to_update,
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

    for resource in to_create {
      let name = resource.name.clone();
      let id = match crate::resource::create::<Self>(
        &resource.name,
        resource.config,
        sync_user(),
      )
      .await
      .map_err(|e| e.error)
      {
        Ok(resource) => resource.id,
        Err(e) => {
          has_error = true;
          log.push_str(&format!(
            "\n{}: failed to create {} '{}' | {e:#}",
            colored("ERROR", Color::Red),
            Self::resource_type(),
            bold(&name)
          ));
          continue;
        }
      };
      run_update_meta::<Self>(
        id.clone(),
        &name,
        ResourceMetaUpdate {
          description: Some(resource.description),
          template: Some(resource.template),
          tags: Some(resource.tags),
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
    }

    for ToUpdateItem {
      id,
      resource,
      update_description,
      update_template,
      update_tags,
    } in to_update
    {
      let name = resource.name.clone();

      let meta = ResourceMetaUpdate {
        description: update_description
          .then(|| resource.description.clone()),
        template: update_template.then_some(resource.template),
        tags: update_tags.then(|| resource.tags.clone()),
      };

      if !meta.is_none() {
        run_update_meta::<Self>(
          id.clone(),
          &name,
          meta,
          &mut log,
          &mut has_error,
        )
        .await;
      }

      if !resource.config.is_none() {
        if let Err(e) = crate::resource::update::<Self>(
          &id,
          resource.config,
          sync_user(),
        )
        .await
        {
          has_error = true;
          log.push_str(&format!(
            "\n{}: failed to update config on {} '{}' | {e:#}",
            colored("ERROR", Color::Red),
            Self::resource_type(),
            bold(&name),
          ))
        } else {
          log.push_str(&format!(
            "\n{}: {} {} '{}' configuration",
            muted("INFO"),
            colored("updated", Color::Blue),
            Self::resource_type(),
            bold(&name)
          ));
        }
      }
    }

    for resource in to_delete {
      if let Err(e) = crate::resource::delete::<Self>(
        &resource,
        &WriteArgs {
          user: sync_user().to_owned(),
        },
      )
      .await
      {
        has_error = true;
        log.push_str(&format!(
          "\n{}: failed to delete {} '{}' | {e:#}",
          colored("ERROR", Color::Red),
          Self::resource_type(),
          bold(&resource),
        ))
      } else {
        log.push_str(&format!(
          "\n{}: {} {} '{}'",
          muted("INFO"),
          colored("deleted", Color::Red),
          Self::resource_type(),
          bold(&resource)
        ));
      }
    }

    let stage = format!("Update {}s", Self::resource_type());
    Some(if has_error {
      Log::error(&stage, log)
    } else {
      Log::simple(&stage, log)
    })
  }
}

pub async fn run_update_meta<Resource: ResourceSyncTrait>(
  id: String,
  name: &str,
  meta: ResourceMetaUpdate,
  log: &mut String,
  has_error: &mut bool,
) {
  if let Err(e) = crate::resource::update_meta::<Resource>(
    &id,
    meta,
    &WriteArgs {
      user: sync_user().to_owned(),
    },
  )
  .await
  {
    *has_error = true;
    log.push_str(&format!(
      "\n{}: failed to update tags on {} '{}' | {:#}",
      colored("ERROR", Color::Red),
      Resource::resource_type(),
      bold(name),
      e
    ))
  } else {
    log.push_str(&format!(
      "\n{}: {} {} '{}' meta",
      muted("INFO"),
      colored("updated", Color::Blue),
      Resource::resource_type(),
      bold(name)
    ));
  }
}
