use std::{collections::HashMap, path::Path, str::FromStr};

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::oid::ObjectId;
use komodo_client::entities::{
  ResourceTargetVariant,
  tag::Tag,
  toml::{ResourceToml, ResourcesToml},
};
use partial_derive2::PartialDiff as _;
use toml::ToToml;

use crate::resource::KomodoResource;

pub mod deploy;
pub mod execute;
pub mod file;
pub mod remote;
pub mod replace_ids;
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
  pub update_template: bool,
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

  // /// Apply any changes to incoming toml partial config
  // /// before it is diffed against existing config
  // fn validate_partial_config(_config: &mut Self::PartialConfig) {}

  /// Diffs the declared toml (partial) against the full existing config.
  /// Removes all fields from toml (partial) that haven't changed.
  fn get_diff(
    mut original: Self::Config,
    update: Self::PartialConfig,
  ) -> anyhow::Result<Self::ConfigDiff> {
    Self::replace_ids(&mut original);
    Ok(original.partial_diff(update))
  }

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

fn deserialize_ui_resources_toml(
  toml_str: &str,
) -> anyhow::Result<ResourcesToml> {
  deserialize_resources_toml(toml_str, |name, _| {
    Err(anyhow!(
      "config.file_path is not supported for UI defined syncs | action: {name}"
    ))
  })
}

fn deserialize_resources_toml(
  toml_str: &str,
  read_action_file: impl FnMut(&str, &str) -> anyhow::Result<String>,
) -> anyhow::Result<ResourcesToml> {
  let escaped = escape_between_triple_string(toml_str);
  let mut table = ::toml::from_str::<::toml::Table>(&escaped)
    .map_err(|e| anyhow!("{e:#}"))?;
  if resolve_action_file_paths(&mut table, read_action_file)? {
    table
      .try_into::<ResourcesToml>()
      .map_err(|e| anyhow!("{e:#}"))
  } else {
    // Deserialize from source to preserve type-error line and column numbers.
    ::toml::from_str::<ResourcesToml>(&escaped)
      // the error without this comes through with multiple lines (\n) and looks bad
      .map_err(|e| anyhow!("{e:#}"))
  }
}

/// Returns whether any action file paths were replaced with file contents.
fn resolve_action_file_paths(
  table: &mut ::toml::Table,
  mut read_action_file: impl FnMut(&str, &str) -> anyhow::Result<String>,
) -> anyhow::Result<bool> {
  let mut resolved = false;
  for key in ["action", "actions"] {
    let Some(::toml::Value::Array(actions)) = table.get_mut(key)
    else {
      continue;
    };
    for action in actions {
      let Some(action) = action.as_table_mut() else {
        continue;
      };
      let name = action
        .get("name")
        .and_then(|name| name.as_str())
        .unwrap_or_default()
        .to_string();
      let Some(config) =
        action.get_mut("config").and_then(|c| c.as_table_mut())
      else {
        continue;
      };
      let Some(file_path) = config.remove("file_path") else {
        continue;
      };
      let file_path = file_path.as_str().with_context(|| {
        format!("config.file_path must be a string | action: {name}")
      })?;
      if config.contains_key("file_contents") {
        return Err(anyhow!(
          "Cannot set both config.file_path and config.file_contents | action: {name}"
        ));
      }
      let contents = read_action_file(&name, file_path)?;
      resolved = true;
      config.insert(
        String::from("file_contents"),
        ::toml::Value::String(contents),
      );
    }
  }
  Ok(resolved)
}

/// Returns the contents of the file at `file_path` relative to `dir`, ensuring the resolved file is inside `root`.
fn read_file_in_root(
  dir: &Path,
  root: &Path,
  file_path: &str,
) -> anyhow::Result<String> {
  let root = root
    .canonicalize()
    .with_context(|| format!("Failed to resolve root {root:?}"))?;
  let path = dir.join(file_path);
  let full_path = path
    .canonicalize()
    .with_context(|| format!("Failed to resolve path {path:?}"))?;
  if !full_path.starts_with(&root) {
    return Err(anyhow!(
      "Path {full_path:?} is outside sync root {root:?}"
    ));
  }
  std::fs::read_to_string(&full_path)
    .with_context(|| format!("Failed to read {full_path:?}"))
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
