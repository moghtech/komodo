use std::{
  collections::{HashMap, HashSet},
  str::FromStr,
};

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::{delete_one_by_id, update_one_by_id},
  find::find_collect,
  mongodb::{
    Collection,
    bson::{Document, doc, oid::ObjectId, to_document},
    options::FindOptions,
  },
};
use formatting::format_serror;
use futures::future::join_all;
use indexmap::IndexSet;
use komodo_client::{
  api::{read::ExportResourcesToToml, write::CreateTag},
  entities::{
    Operation, ResourceTarget, ResourceTargetVariant,
    komodo_timestamp,
    permission::{
      PermissionLevel, PermissionLevelAndSpecifics,
      SpecificPermission,
    },
    resource::{AddFilters, Resource, ResourceQuery},
    tag::Tag,
    to_general_name,
    update::Update,
    user::{User, system_user},
  },
  parsers::parse_string_list,
};
use partial_derive2::{Diff, MaybeNone, PartialDiff};
use resolver_api::Resolve;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
  api::{read::ReadArgs, write::WriteArgs},
  helpers::{
    create_permission, flatten_document,
    query::{get_tag, id_or_name_filter},
    update::{add_update, make_update},
  },
  permission::{get_check_permissions, get_resource_ids_for_user},
  state::db_client,
};

mod action;
mod alerter;
mod build;
mod builder;
mod deployment;
mod procedure;
mod refresh;
mod repo;
mod server;
mod stack;
mod sync;

pub use action::{
  refresh_action_state_cache, spawn_action_state_refresh_loop,
};
pub use build::{
  refresh_build_state_cache, spawn_build_state_refresh_loop,
};
pub use procedure::{
  refresh_procedure_state_cache, spawn_procedure_state_refresh_loop,
};
pub use refresh::{
  refresh_all_resources_cache,
  spawn_all_resources_cache_refresh_loop,
  spawn_resource_refresh_loop,
};
pub use repo::{
  refresh_repo_state_cache, spawn_repo_state_refresh_loop,
};

/// Implement on each Komodo resource for common methods
pub trait KomodoResource {
  type ListItem: Serialize + Send;
  type Config: Clone
    + Default
    + Send
    + Sync
    + Unpin
    + Serialize
    + DeserializeOwned
    + From<Self::PartialConfig>
    + PartialDiff<Self::PartialConfig, Self::ConfigDiff>
    + 'static;
  type PartialConfig: Clone
    + Default
    + From<Self::Config>
    + Serialize
    + MaybeNone;
  type ConfigDiff: Into<Self::PartialConfig>
    + Serialize
    + Diff
    + MaybeNone;
  type Info: Clone
    + Send
    + Sync
    + Unpin
    + Default
    + Serialize
    + DeserializeOwned
    + 'static;
  type QuerySpecifics: AddFilters + Default + std::fmt::Debug;

  fn resource_type() -> ResourceTargetVariant;
  fn resource_target(id: impl Into<String>) -> ResourceTarget;

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>;

  async fn to_list_item(
    resource: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem;

  #[allow(clippy::ptr_arg)]
  async fn busy(id: &String) -> anyhow::Result<bool>;

  /// Some resource types have restrictions on the allowed formatting for names.
  /// Stacks, Builds, and Deployments all require names to be "docker compatible",
  /// which means all lowercase, and no spaces or dots.
  fn validated_name(name: &str) -> String {
    to_general_name(name)
  }

  /// These permissions go to the creator of the resource,
  /// and include full access to the resource.
  fn creator_specific_permissions() -> IndexSet<SpecificPermission> {
    IndexSet::new()
  }

  /// For Stacks / Deployments, they should inherit specific
  /// permissions like `Logs`, `Inspect`, and `Terminal`
  /// from their attached Server.
  fn inherit_specific_permissions_from(
    _self: &Resource<Self::Config, Self::Info>,
  ) -> Option<ResourceTarget> {
    None
  }

  // =======
  // CREATE
  // =======

  fn create_operation() -> Operation;

  fn user_can_create(user: &User) -> bool;

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()>;

  async fn default_info() -> anyhow::Result<Self::Info> {
    Ok(Default::default())
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()>;

  // =======
  // UPDATE
  // =======

  fn update_operation() -> Operation;

  async fn validate_update_config(
    id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()>;

  /// Should be overridden for enum configs, eg Alerter, Builder, ...
  fn update_document(
    _original: Resource<Self::Config, Self::Info>,
    config: Self::PartialConfig,
  ) -> Result<Document, database::mungos::mongodb::bson::ser::Error>
  {
    to_document(&config)
  }

  /// Run any required task after resource updated in database but
  /// before the request resolves.
  async fn post_update(
    updated: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()>;

  // =======
  // RENAME
  // =======

  fn rename_operation() -> Operation;

  // =======
  // DELETE
  // =======

  fn delete_operation() -> Operation;

  /// Clean up all links to this resource before deleting it.
  async fn pre_delete(
    resource: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()>;

  /// Run any required task after resource deleted from database but
  /// before the request resolves.
  async fn post_delete(
    resource: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()>;
}

// Methods

// ======
// GET
// ======

pub async fn get<T: KomodoResource>(
  id_or_name: &str,
) -> anyhow::Result<Resource<T::Config, T::Info>> {
  if id_or_name.is_empty() {
    return Err(anyhow!(
      "Cannot find {} with empty name / id",
      T::resource_type()
    ));
  }
  T::coll()
    .find_one(id_or_name_filter(id_or_name))
    .await
    .context("failed to query db for resource")?
    .with_context(|| {
      format!(
        "did not find any {} matching {id_or_name}",
        T::resource_type()
      )
    })
}

// ======
// LIST
// ======

/// Returns None if still no need to filter by resource id (eg transparent mode, group membership with all access).
#[instrument(level = "debug")]
pub async fn get_resource_object_ids_for_user<T: KomodoResource>(
  user: &User,
) -> anyhow::Result<Option<Vec<ObjectId>>> {
  get_resource_ids_for_user::<T>(user).await.map(|ids| {
    ids.map(|ids| {
      ids
        .into_iter()
        .flat_map(|id| ObjectId::from_str(&id))
        .collect()
    })
  })
}

#[instrument(level = "debug")]
pub async fn list_for_user<T: KomodoResource>(
  mut query: ResourceQuery<T::QuerySpecifics>,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<T::ListItem>> {
  validate_resource_query_tags(&mut query, all_tags)?;
  let mut filters = Document::new();
  query.add_filters(&mut filters);
  list_for_user_using_document::<T>(filters, user, permissions).await
}

// #[instrument(level = "debug")]
// pub async fn list_for_user_using_pattern<T: KomodoResource>(
//   pattern: &str,
//   query: ResourceQuery<T::QuerySpecifics>,
//   user: &User,
//   permissions: PermissionLevelAndSpecifics,
//   all_tags: &[Tag],
// ) -> anyhow::Result<Vec<T::ListItem>> {
//   let list = list_full_for_user_using_pattern::<T>(
//     pattern,
//     query,
//     user,
//     permissions,
//     all_tags,
//   )
//   .await?
//   .into_iter()
//   .map(|resource| T::to_list_item(resource));
//   Ok(join_all(list).await)
// }

#[instrument(level = "debug")]
pub async fn list_for_user_using_document<T: KomodoResource>(
  filters: Document,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
) -> anyhow::Result<Vec<T::ListItem>> {
  let list = list_full_for_user_using_document::<T>(filters, user)
    .await?
    .into_iter()
    .map(|resource| T::to_list_item(resource));
  Ok(join_all(list).await)
}

/// Lists full resource matching wildcard syntax,
/// or regex if wrapped with "\\"
///
/// ## Example
/// ```
/// let items = list_full_for_user_using_match_string::<Build>("foo-*", Default::default(), user, all_tags).await?;
/// let items = list_full_for_user_using_match_string::<Build>("\\^foo-.*$\\", Default::default(), user, all_tags).await?;
/// ```
#[instrument(level = "debug")]
pub async fn list_full_for_user_using_pattern<T: KomodoResource>(
  pattern: &str,
  query: ResourceQuery<T::QuerySpecifics>,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>> {
  let resources =
    list_full_for_user::<T>(query, user, permissions, all_tags)
      .await?;

  let patterns = parse_string_list(pattern);
  let mut names = HashSet::<String>::new();

  for pattern in patterns {
    if pattern.starts_with('\\') && pattern.ends_with('\\') {
      let regex = regex::Regex::new(&pattern[1..(pattern.len() - 1)])
        .context("Regex matching string invalid")?;
      for resource in &resources {
        if regex.is_match(&resource.name) {
          names.insert(resource.name.clone());
        }
      }
    } else {
      let wildcard = wildcard::Wildcard::new(pattern.as_bytes())
        .context("Wildcard matching string invalid")?;
      for resource in &resources {
        if wildcard.is_match(resource.name.as_bytes()) {
          names.insert(resource.name.clone());
        }
      }
    };
  }

  Ok(
    resources
      .into_iter()
      .filter(|resource| names.contains(resource.name.as_str()))
      .collect(),
  )
}

#[instrument(level = "debug")]
pub async fn list_full_for_user<T: KomodoResource>(
  mut query: ResourceQuery<T::QuerySpecifics>,
  user: &User,
  permissions: PermissionLevelAndSpecifics,
  all_tags: &[Tag],
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>> {
  validate_resource_query_tags(&mut query, all_tags)?;
  let mut filters = Document::new();
  query.add_filters(&mut filters);
  list_full_for_user_using_document::<T>(filters, user).await
}

#[instrument(level = "debug")]
pub async fn list_full_for_user_using_document<T: KomodoResource>(
  mut filters: Document,
  user: &User,
) -> anyhow::Result<Vec<Resource<T::Config, T::Info>>> {
  if let Some(ids) =
    get_resource_object_ids_for_user::<T>(user).await?
  {
    filters.insert("_id", doc! { "$in": ids });
  }
  find_collect(
    T::coll(),
    filters,
    FindOptions::builder().sort(doc! { "name": 1 }).build(),
  )
  .await
  .with_context(|| {
    format!("failed to pull {}s from mongo", T::resource_type())
  })
}

pub type IdResourceMap<T> = HashMap<
  String,
  Resource<
    <T as KomodoResource>::Config,
    <T as KomodoResource>::Info,
  >,
>;

#[instrument(level = "debug")]
pub async fn get_id_to_resource_map<T: KomodoResource>(
  id_to_tags: &HashMap<String, Tag>,
  match_tags: &[String],
) -> anyhow::Result<IdResourceMap<T>> {
  let res = find_collect(T::coll(), None, None)
    .await
    .with_context(|| {
      format!("failed to pull {}s from mongo", T::resource_type())
    })?
    .into_iter()
    .filter(|resource| {
      if match_tags.is_empty() {
        return true;
      }
      for tag in match_tags.iter() {
        for resource_tag in &resource.tags {
          match ObjectId::from_str(resource_tag) {
            Ok(_) => match id_to_tags
              .get(resource_tag)
              .map(|tag| tag.name.as_str())
            {
              Some(name) => {
                if tag != name {
                  return false;
                }
              }
              None => return false,
            },
            Err(_) => {
              if resource_tag != tag {
                return false;
              }
            }
          }
        }
      }
      true
    })
    .map(|r| (r.id.clone(), r))
    .collect();
  Ok(res)
}

// =======
// CREATE
// =======

pub async fn create<T: KomodoResource>(
  name: &str,
  mut config: T::PartialConfig,
  user: &User,
) -> anyhow::Result<Resource<T::Config, T::Info>> {
  if !T::user_can_create(user) {
    return Err(anyhow!(
      "User does not have permissions to create {}.",
      T::resource_type()
    ));
  }

  if name.is_empty() {
    return Err(anyhow!("Must provide non-empty name for resource."));
  }

  let name = T::validated_name(name);

  if ObjectId::from_str(&name).is_ok() {
    return Err(anyhow!("valid ObjectIds cannot be used as names."));
  }

  // Ensure an existing resource with same name doesn't already exist
  // The database indexing also ensures this but doesn't give a good error message.
  if list_full_for_user::<T>(
    Default::default(),
    system_user(),
    PermissionLevel::Read.into(),
    &[],
  )
  .await
  .context("Failed to list all resources for duplicate name check")?
  .into_iter()
  .any(|r| r.name == name)
  {
    return Err(anyhow!("Must provide unique name for resource."));
  }

  let start_ts = komodo_timestamp();

  T::validate_create_config(&mut config, user).await?;

  let resource = Resource::<T::Config, T::Info> {
    id: Default::default(),
    name,
    description: Default::default(),
    template: Default::default(),
    tags: Default::default(),
    config: config.into(),
    info: T::default_info().await?,
    base_permission: PermissionLevel::None.into(),
    updated_at: start_ts,
  };

  let resource_id = T::coll()
    .insert_one(&resource)
    .await
    .with_context(|| {
      format!("failed to add {} to db", T::resource_type())
    })?
    .inserted_id
    .as_object_id()
    .context("inserted_id is not ObjectId")?
    .to_string();

  let resource = get::<T>(&resource_id).await?;
  let target = resource_target::<T>(resource_id);

  create_permission(
    user,
    target.clone(),
    PermissionLevel::Write,
    T::creator_specific_permissions(),
  )
  .await;

  let mut update = make_update(target, T::create_operation(), user);
  update.start_ts = start_ts;
  update.push_simple_log(
    &format!("create {}", T::resource_type()),
    format!(
      "created {}\nid: {}\nname: {}",
      T::resource_type(),
      resource.id,
      resource.name
    ),
  );
  update.push_simple_log(
    "config",
    serde_json::to_string_pretty(&resource.config)
      .context("failed to serialize resource config to JSON")?,
  );

  T::post_create(&resource, &mut update).await?;

  refresh_all_resources_cache().await;

  update.finalize();
  add_update(update).await?;

  Ok(resource)
}

// =======
// UPDATE
// =======

pub async fn update<T: KomodoResource>(
  id_or_name: &str,
  mut config: T::PartialConfig,
  user: &User,
) -> anyhow::Result<Resource<T::Config, T::Info>> {
  let resource = get_check_permissions::<T>(
    id_or_name,
    user,
    PermissionLevel::Write.into(),
  )
  .await?;

  if T::busy(&resource.id).await? {
    return Err(anyhow!("{} busy", T::resource_type()));
  }

  T::validate_update_config(&resource.id, &mut config, user).await?;

  // Gets a diff object.
  let diff = resource.config.partial_diff(config);

  if diff.is_none() {
    return Ok(resource);
  }

  // Leave this Result unhandled for now
  let prev_toml = ExportResourcesToToml {
    targets: vec![T::resource_target(&resource.id)],
    ..Default::default()
  }
  .resolve(&ReadArgs {
    user: system_user().to_owned(),
  })
  .await
  .map_err(|e| e.error)
  .context("Failed to export resource toml before update");

  // This minimizes the update against the existing config
  let config: T::PartialConfig = diff.into();

  let id = resource.id.clone();

  let config_doc = T::update_document(resource, config)
    .context("failed to serialize config to bson document")?;

  let update_doc = flatten_document(doc! { "config": config_doc });

  update_one_by_id(T::coll(), &id, doc! { "$set": update_doc }, None)
    .await
    .context("failed to update resource on database")?;

  let curr_toml = ExportResourcesToToml {
    targets: vec![T::resource_target(&id)],
    ..Default::default()
  }
  .resolve(&ReadArgs {
    user: system_user().to_owned(),
  })
  .await
  .map_err(|e| e.error)
  .context("Failed to export resource toml after update");

  let mut update = make_update(
    resource_target::<T>(id),
    T::update_operation(),
    user,
  );

  match prev_toml {
    Ok(res) => update.prev_toml = res.toml,
    Err(e) => update
      // These logs are pushed with success == true, so user still knows the update was succesful.
      .push_simple_log("Failed export", format_serror(&e.into())),
  }
  match curr_toml {
    Ok(res) => update.current_toml = res.toml,
    Err(e) => update
      // These logs are pushed with success == true, so user still knows the update was succesful.
      .push_simple_log("Failed export", format_serror(&e.into())),
  }

  let updated = get::<T>(id_or_name).await?;

  T::post_update(&updated, &mut update).await?;

  refresh_all_resources_cache().await;

  update.finalize();
  add_update(update).await?;

  Ok(updated)
}

fn resource_target<T: KomodoResource>(id: String) -> ResourceTarget {
  match T::resource_type() {
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
    ResourceTargetVariant::ResourceSync => {
      ResourceTarget::ResourceSync(id)
    }
    ResourceTargetVariant::Stack => ResourceTarget::Stack(id),
    ResourceTargetVariant::Action => ResourceTarget::Action(id),
  }
}

pub struct ResourceMetaUpdate {
  pub description: Option<String>,
  pub template: Option<bool>,
  pub tags: Option<Vec<String>>,
}

impl ResourceMetaUpdate {
  pub fn is_none(&self) -> bool {
    self.description.is_none()
      && self.template.is_none()
      && self.tags.is_none()
  }
}

pub async fn update_meta<T: KomodoResource>(
  id_or_name: &str,
  meta: ResourceMetaUpdate,
  args: &WriteArgs,
) -> anyhow::Result<()> {
  get_check_permissions::<T>(
    id_or_name,
    &args.user,
    PermissionLevel::Write.into(),
  )
  .await?;
  let mut set = Document::new();
  if let Some(description) = meta.description {
    set.insert("description", description);
  }
  if let Some(template) = meta.template {
    set.insert("template", template);
  }
  if let Some(tags) = meta.tags {
    // First normalize to tag ids only
    let futures = tags.iter().map(|tag| async {
      match get_tag(tag).await {
        Ok(tag) => Ok(tag.id),
        Err(_) => CreateTag {
          name: tag.to_string(),
          color: None,
        }
        .resolve(args)
        .await
        .map(|tag| tag.id),
      }
    });
    let tags = join_all(futures)
      .await
      .into_iter()
      .flatten()
      .collect::<Vec<_>>();
    set.insert("tags", tags);
  }
  T::coll()
    .update_one(id_or_name_filter(id_or_name), doc! { "$set": set })
    .await?;
  refresh_all_resources_cache().await;
  Ok(())
}

pub async fn remove_tag_from_all<T: KomodoResource>(
  tag_id: &str,
) -> anyhow::Result<()> {
  T::coll()
    .update_many(doc! {}, doc! { "$pull": { "tags": tag_id } })
    .await
    .context("failed to remove tag from resources")?;
  Ok(())
}

// =======
// RENAME
// =======

pub async fn rename<T: KomodoResource>(
  id_or_name: &str,
  name: &str,
  user: &User,
) -> anyhow::Result<Update> {
  let resource = get_check_permissions::<T>(
    id_or_name,
    user,
    PermissionLevel::Write.into(),
  )
  .await?;

  let mut update = make_update(
    resource_target::<T>(resource.id.clone()),
    T::rename_operation(),
    user,
  );

  let name = T::validated_name(name);

  update_one_by_id(
    T::coll(),
    &resource.id,
    database::mungos::update::Update::Set(
      doc! { "name": &name, "updated_at": komodo_timestamp() },
    ),
    None,
  )
  .await
  .with_context(|| {
    format!(
      "Failed to update {ty} on db. This name may already be taken.",
      ty = T::resource_type()
    )
  })?;

  update.push_simple_log(
    &format!("Rename {}", T::resource_type()),
    format!(
      "Renamed {ty} {id} from {prev_name} to {name}",
      ty = T::resource_type(),
      id = resource.id,
      prev_name = resource.name
    ),
  );

  refresh_all_resources_cache().await;

  update.finalize();
  update.id = add_update(update.clone()).await?;

  Ok(update)
}

// =======
// DELETE
// =======

pub async fn delete<T: KomodoResource>(
  id_or_name: &str,
  args: &WriteArgs,
) -> anyhow::Result<Resource<T::Config, T::Info>> {
  let resource = get_check_permissions::<T>(
    id_or_name,
    &args.user,
    PermissionLevel::Write.into(),
  )
  .await?;

  if T::busy(&resource.id).await? {
    return Err(anyhow!("{} busy", T::resource_type()));
  }

  let target = resource_target::<T>(resource.id.clone());
  let toml = ExportResourcesToToml {
    targets: vec![target.clone()],
    ..Default::default()
  }
  .resolve(&ReadArgs {
    user: args.user.clone(),
  })
  .await
  .map_err(|e| e.error)?
  .toml;

  let mut update =
    make_update(target.clone(), T::delete_operation(), &args.user);

  T::pre_delete(&resource, &mut update).await?;

  delete_all_permissions_on_resource(target.clone()).await;
  remove_from_recently_viewed(target.clone()).await;

  delete_one_by_id(T::coll(), &resource.id, None)
    .await
    .with_context(|| {
      format!("Failed to delete {} from database", T::resource_type())
    })?;

  update.push_simple_log(
    &format!("Delete {}", T::resource_type()),
    format!("Deleted {} {}", T::resource_type(), resource.name),
  );
  update.push_simple_log("Deleted Toml", toml);

  tokio::join!(
    async {
      if let Err(e) = T::post_delete(&resource, &mut update).await {
        update
          .push_error_log("post delete", format_serror(&e.into()));
      }
    },
    delete_from_alerters::<T>(&resource.id)
  );

  refresh_all_resources_cache().await;

  update.finalize();
  add_update(update).await?;

  Ok(resource)
}

async fn delete_from_alerters<T: KomodoResource>(id: &str) {
  let target_bson = doc! {
    "type": T::resource_type().as_ref(),
    "id": id,
  };
  if let Err(e) = db_client()
    .alerters
    .update_many(Document::new(), doc! {
      "$pull": {
        "config.resources": &target_bson,
        "config.except_resources": target_bson,
      }
    })
    .await
    .context("Failed to clear deleted resource from alerter whitelist / blacklist")
  {
    warn!("{e:#}");
  }
}

// =======

#[instrument(level = "debug")]
pub fn validate_resource_query_tags<T: Default + std::fmt::Debug>(
  query: &mut ResourceQuery<T>,
  all_tags: &[Tag],
) -> anyhow::Result<()> {
  query.tags = query
    .tags
    .iter()
    .map(|tag| {
      all_tags
        .iter()
        .find(|t| t.name == *tag || t.id == *tag)
        .map(|tag| tag.id.clone())
        .with_context(|| {
          format!("No tag found matching name or id: {tag}")
        })
    })
    .collect::<anyhow::Result<Vec<_>>>()?;
  Ok(())
}

#[instrument]
pub async fn delete_all_permissions_on_resource<T>(target: T)
where
  T: Into<ResourceTarget> + std::fmt::Debug,
{
  let target: ResourceTarget = target.into();
  let (variant, id) = target.extract_variant_id();
  if let Err(e) = db_client()
    .permissions
    .delete_many(doc! {
      "resource_target.type": variant.as_ref(),
      "resource_target.id": &id
    })
    .await
  {
    warn!(
      "failed to delete_many permissions matching target {target:?} | {e:#}"
    );
  }
}

#[instrument]
pub async fn remove_from_recently_viewed<T>(resource: T)
where
  T: Into<ResourceTarget> + std::fmt::Debug,
{
  let resource: ResourceTarget = resource.into();
  let (recent_field, id) = match resource {
    ResourceTarget::Server(id) => ("recents.Server", id),
    ResourceTarget::Deployment(id) => ("recents.Deployment", id),
    ResourceTarget::Build(id) => ("recents.Build", id),
    ResourceTarget::Repo(id) => ("recents.Repo", id),
    ResourceTarget::Procedure(id) => ("recents.Procedure", id),
    ResourceTarget::Action(id) => ("recents.Action", id),
    ResourceTarget::Stack(id) => ("recents.Stack", id),
    ResourceTarget::Builder(id) => ("recents.Builder", id),
    ResourceTarget::Alerter(id) => ("recents.Alerter", id),
    ResourceTarget::ResourceSync(id) => ("recents.ResourceSync", id),
    ResourceTarget::System(_) => return,
  };
  if let Err(e) = db_client()
    .users
    .update_many(
      doc! {},
      doc! {
        "$pull": {
          recent_field: id
        }
      },
    )
    .await
    .context("failed to remove resource from users recently viewed")
  {
    warn!("{e:#}");
  }
}
