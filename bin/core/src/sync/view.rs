use std::collections::HashMap;

use anyhow::Context;
use komodo_client::entities::{
  ResourceTargetVariant,
  sync::{DiffData, ResourceDiff},
  tag::Tag,
  toml::ResourceToml,
};
use mungos::find::find_collect;
use partial_derive2::MaybeNone;

use super::ResourceSyncTrait;

#[allow(clippy::too_many_arguments)]
pub async fn push_updates_for_view<Resource: ResourceSyncTrait>(
  resources: Vec<ResourceToml<Resource::PartialConfig>>,
  delete: bool,
  match_resource_type: Option<ResourceTargetVariant>,
  match_resources: Option<&[String]>,
  id_to_tags: &HashMap<String, Tag>,
  match_tags: &[String],
  diffs: &mut Vec<ResourceDiff>,
) -> anyhow::Result<()> {
  let current_map = find_collect(Resource::coll(), None, None)
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

  if delete {
    for current_resource in current_map.values() {
      if !resources.iter().any(|r| r.name == current_resource.name) {
        diffs.push(ResourceDiff {
          target: Resource::resource_target(
            current_resource.id.clone(),
          ),
          data: DiffData::Delete {
            current: super::toml::resource_to_toml::<Resource>(
              current_resource.clone(),
              false,
              vec![],
              id_to_tags,
            )?,
          },
        });
      }
    }
  }

  for mut proposed_resource in resources {
    match current_map.get(&proposed_resource.name) {
      Some(current_resource) => {
        // First merge toml resource config (partial) onto default resource config.
        // Makes sure things that aren't defined in toml (come through as None) actually get removed.
        let propsed_config: Resource::Config =
          proposed_resource.config.into();
        proposed_resource.config = propsed_config.into();

        Resource::validate_partial_config(
          &mut proposed_resource.config,
        );

        let proposed = super::toml::resource_toml_to_toml_string::<
          Resource,
        >(proposed_resource.clone())?;

        let mut diff = Resource::get_diff(
          current_resource.config.clone(),
          proposed_resource.config,
        )?;

        Resource::validate_diff(&mut diff);

        let current_tags = current_resource
          .tags
          .iter()
          .filter_map(|id| id_to_tags.get(id).map(|t| t.name.clone()))
          .collect::<Vec<_>>();

        // Only proceed if there are any fields to update,
        // or a change to tags / description
        if diff.is_none()
          && proposed_resource.description
            == current_resource.description
          && proposed_resource.tags == current_tags
        {
          continue;
        }

        diffs.push(ResourceDiff {
          target: Resource::resource_target(
            current_resource.id.clone(),
          ),
          data: DiffData::Update {
            current: super::toml::resource_to_toml::<Resource>(
              current_resource.clone(),
              proposed_resource.deploy,
              proposed_resource.after,
              id_to_tags,
            )?,
            proposed,
          },
        });
      }
      None => {
        diffs.push(ResourceDiff {
          // resources to Create don't have ids yet.
          target: Resource::resource_target(String::new()),

          data: DiffData::Create {
            name: proposed_resource.name.clone(),
            proposed: super::toml::resource_toml_to_toml_string::<
              Resource,
            >(proposed_resource)?,
          },
        });
      }
    }
  }

  Ok(())
}
