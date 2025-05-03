use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use komodo_client::entities::{
  ResourceTargetVariant,
  action::Action,
  alerter::Alerter,
  build::Build,
  builder::Builder,
  deployment::Deployment,
  procedure::Procedure,
  repo::Repo,
  server::Server,
  stack::Stack,
  sync::ResourceSync,
  tag::Tag,
  toml::{ResourceToml, ResourcesToml},
};
use mungos::mongodb::bson::oid::ObjectId;
use toml::ToToml;

use crate::resource::KomodoResource;

pub mod deploy;
pub mod execute;
pub mod file;
pub mod remote;
pub mod resources;
pub mod toml;
pub mod user_groups;
pub mod variables;
pub mod view;

#[derive(Default)]
pub struct SyncDeltas<T: Default> {
  pub to_create: Vec<ResourceToml<T>>,
  pub to_update: Vec<ToUpdateItem<T>>,
  pub to_delete: Vec<String>,
}

impl<T: Default> SyncDeltas<T> {
  pub fn no_changes(&self) -> bool {
    self.to_create.is_empty()
      && self.to_update.is_empty()
      && self.to_delete.is_empty()
  }
}

pub struct ToUpdateItem<T: Default> {
  pub id: String,
  pub resource: ResourceToml<T>,
  pub update_description: bool,
  pub update_tags: bool,
}

pub trait ResourceSyncTrait: ToToml + Sized {
  /// To exclude resource syncs with "file_contents" (they aren't compatible)
  fn include_resource(
    name: &String,
    _config: &Self::Config,
    match_resource_type: Option<ResourceTargetVariant>,
    match_resources: Option<&[String]>,
    resource_tags: &[String],
    id_to_tags: &HashMap<String, Tag>,
    match_tags: &[String],
  ) -> bool {
    include_resource_by_resource_type_and_name::<Self>(
      match_resource_type,
      match_resources,
      name,
    ) && include_resource_by_tags(
      resource_tags,
      id_to_tags,
      match_tags,
    )
  }

  /// To exclude resource syncs with "file_contents" (they aren't compatible)
  fn include_resource_partial(
    name: &String,
    _config: &Self::PartialConfig,
    match_resource_type: Option<ResourceTargetVariant>,
    match_resources: Option<&[String]>,
    resource_tags: &[String],
    id_to_tags: &HashMap<String, Tag>,
    match_tags: &[String],
  ) -> bool {
    include_resource_by_resource_type_and_name::<Self>(
      match_resource_type,
      match_resources,
      name,
    ) && include_resource_by_tags(
      resource_tags,
      id_to_tags,
      match_tags,
    )
  }

  /// Apply any changes to incoming toml partial config
  /// before it is diffed against existing config
  fn validate_partial_config(_config: &mut Self::PartialConfig) {}

  /// Diffs the declared toml (partial) against the full existing config.
  /// Removes all fields from toml (partial) that haven't changed.
  fn get_diff(
    original: Self::Config,
    update: Self::PartialConfig,
    resources: &AllResourcesById,
  ) -> anyhow::Result<Self::ConfigDiff>;

  /// Apply any changes to computed config diff
  /// before logging
  fn validate_diff(_diff: &mut Self::ConfigDiff) {}
}

pub fn include_resource_by_tags(
  resource_tags: &[String],
  id_to_tags: &HashMap<String, Tag>,
  match_tags: &[String],
) -> bool {
  let tag_names = resource_tags
    .iter()
    .filter_map(|resource_tag| {
      match ObjectId::from_str(resource_tag) {
        Ok(_) => id_to_tags.get(resource_tag).map(|tag| &tag.name),
        Err(_) => Some(resource_tag),
      }
    })
    .collect::<Vec<_>>();
  match_tags.iter().all(|tag| tag_names.contains(&tag))
}

pub fn include_resource_by_resource_type_and_name<
  T: KomodoResource,
>(
  resource_type: Option<ResourceTargetVariant>,
  resources: Option<&[String]>,
  name: &String,
) -> bool {
  match (resource_type, resources) {
    (Some(resource_type), Some(resources)) => {
      if T::resource_type() != resource_type {
        return false;
      }
      resources.contains(name)
    }
    (Some(resource_type), None) => {
      if T::resource_type() != resource_type {
        return false;
      }
      true
    }
    (None, Some(resources)) => resources.contains(name),
    (None, None) => true,
  }
}

pub struct AllResourcesById {
  pub servers: HashMap<String, Server>,
  pub deployments: HashMap<String, Deployment>,
  pub stacks: HashMap<String, Stack>,
  pub builds: HashMap<String, Build>,
  pub repos: HashMap<String, Repo>,
  pub procedures: HashMap<String, Procedure>,
  pub actions: HashMap<String, Action>,
  pub builders: HashMap<String, Builder>,
  pub alerters: HashMap<String, Alerter>,
  pub syncs: HashMap<String, ResourceSync>,
}

impl AllResourcesById {
  /// Use `match_tags` to filter resources by tag.
  pub async fn load() -> anyhow::Result<Self> {
    let map = HashMap::new();
    let id_to_tags = &map;
    let match_tags = &[];
    Ok(Self {
      servers: crate::resource::get_id_to_resource_map::<Server>(
        id_to_tags, match_tags,
      )
      .await?,
      deployments: crate::resource::get_id_to_resource_map::<
        Deployment,
      >(id_to_tags, match_tags)
      .await?,
      builds: crate::resource::get_id_to_resource_map::<Build>(
        id_to_tags, match_tags,
      )
      .await?,
      repos: crate::resource::get_id_to_resource_map::<Repo>(
        id_to_tags, match_tags,
      )
      .await?,
      procedures:
        crate::resource::get_id_to_resource_map::<Procedure>(
          id_to_tags, match_tags,
        )
        .await?,
      actions: crate::resource::get_id_to_resource_map::<Action>(
        id_to_tags, match_tags,
      )
      .await?,
      builders: crate::resource::get_id_to_resource_map::<Builder>(
        id_to_tags, match_tags,
      )
      .await?,
      alerters: crate::resource::get_id_to_resource_map::<Alerter>(
        id_to_tags, match_tags,
      )
      .await?,
      syncs: crate::resource::get_id_to_resource_map::<ResourceSync>(
        id_to_tags, match_tags,
      )
      .await?,
      stacks: crate::resource::get_id_to_resource_map::<Stack>(
        id_to_tags, match_tags,
      )
      .await?,
    })
  }
}

fn deserialize_resources_toml(
  toml_str: &str,
) -> anyhow::Result<ResourcesToml> {
  ::toml::from_str::<ResourcesToml>(&escape_between_triple_string(
    toml_str,
  ))
  // the error without this comes through with multiple lines (\n) and looks bad
  .map_err(|e| anyhow!("{e:#}"))
}

fn escape_between_triple_string(toml_str: &str) -> String {
  toml_str
    .split(r#"""""#)
    .enumerate()
    .map(|(i, section)| {
      // The odd entries are between triple string,
      // and the \ need to be escaped.
      if i % 2 == 0 {
        section.to_string()
      } else {
        section.replace(r#"\"#, r#"\\"#)
      }
    })
    .collect::<Vec<_>>()
    .join(r#"""""#)
}
