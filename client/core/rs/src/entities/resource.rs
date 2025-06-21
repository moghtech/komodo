use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
  deserializers::string_list_deserializer,
  entities::{I64, MongoId},
};

use super::{
  ResourceTargetVariant, permission::PermissionLevelAndSpecifics,
};

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Resource<Config: Default, Info: Default = ()> {
  /// The Mongo ID of the resource.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of serialized Resource<T>) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  #[builder(setter(skip))]
  pub id: MongoId,

  /// The resource name.
  /// This is guaranteed unique among others of the same resource type.
  pub name: String,

  /// A description for the resource
  #[serde(default)]
  #[builder(default)]
  pub description: String,

  /// Mark resource as a template
  #[serde(default)]
  #[builder(default)]
  pub template: bool,

  /// Tag Ids
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[builder(default)]
  pub tags: Vec<String>,

  /// Resource-specific information (not user configurable).
  #[serde(default)]
  #[builder(setter(skip))]
  pub info: Info,

  /// Resource-specific configuration.
  #[serde(default)]
  #[builder(default)]
  pub config: Config,

  /// Set a base permission level that all users will have on the
  /// resource.
  #[serde(default)]
  #[builder(default)]
  pub base_permission: PermissionLevelAndSpecifics,

  /// When description last updated
  #[serde(default)]
  #[builder(setter(skip))]
  pub updated_at: I64,
}

impl<C: Default, I: Default> Default for Resource<C, I> {
  fn default() -> Self {
    Self {
      id: String::new(),
      name: String::from("temp-resource"),
      description: String::new(),
      template: Default::default(),
      tags: Vec::new(),
      info: I::default(),
      config: C::default(),
      base_permission: Default::default(),
      updated_at: 0,
    }
  }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceListItem<Info> {
  /// The resource id
  pub id: String,
  /// The resource type, ie `Server` or `Deployment`
  #[serde(rename = "type")]
  pub resource_type: ResourceTargetVariant,
  /// The resource name
  pub name: String,
  /// Whether resource is a template
  pub template: bool,
  /// Tag Ids
  pub tags: Vec<String>,
  /// Resource specific info
  pub info: Info,
}

/// Passing empty Vec is the same as not filtering by that field
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct ResourceQuery<T: Default> {
  #[serde(default)]
  pub names: Vec<String>,
  #[serde(default)]
  pub templates: TemplatesQueryBehavior,
  /// Pass Vec of tag ids or tag names
  #[serde(default, deserialize_with = "string_list_deserializer")]
  pub tags: Vec<String>,
  /// 'All' or 'Any'
  #[serde(default)]
  pub tag_behavior: TagQueryBehavior,
  #[serde(default)]
  pub specific: T,
}

#[typeshare]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum TemplatesQueryBehavior {
  /// Include templates in results. Default.
  #[default]
  Include,
  /// Exclude templates from results.
  Exclude,
  /// Results *only* includes templates.
  Only,
}

#[typeshare]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum TagQueryBehavior {
  /// Returns resources which have strictly all the tags
  #[default]
  All,
  /// Returns resources which have one or more of the tags
  Any,
}

pub trait AddFilters {
  fn add_filters(&self, _filters: &mut Document) {}
}

impl AddFilters for () {}

impl<T: AddFilters + Default> AddFilters for ResourceQuery<T> {
  fn add_filters(&self, filters: &mut Document) {
    if !self.names.is_empty() {
      filters.insert("name", doc! { "$in": &self.names });
    }
    match self.templates {
      TemplatesQueryBehavior::Exclude => {
        filters.insert("template", doc! { "$ne": true });
      }
      TemplatesQueryBehavior::Only => {
        filters.insert("template", true);
      }
      TemplatesQueryBehavior::Include => {
        // No query on template field necessary
      }
    };
    if !self.tags.is_empty() {
      match self.tag_behavior {
        TagQueryBehavior::All => {
          filters.insert("tags", doc! { "$all": &self.tags });
        }
        TagQueryBehavior::Any => {
          let ors = self
            .tags
            .iter()
            .map(|tag| doc! { "tags": tag })
            .collect::<Vec<_>>();
          filters.insert("$or", ors);
        }
      }
    }
    self.specific.add_filters(filters);
  }
}
