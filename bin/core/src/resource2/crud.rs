use std::str::FromStr;

use anyhow::{anyhow, Context};
use komodo_client::entities::{
  komodo_timestamp,
  permission::{Permission, PermissionLevel, UserTarget},
  resource::Resource,
  to_komodo_name,
  update::Update,
  user::{system_user, User},
  Operation,
};
use mungos::mongodb::bson::{oid::ObjectId, to_document, Document};
use serde::Serialize;

use crate::{
  helpers::update::{add_update, make_update},
  state::db_client,
};

use super::{query::ResourceQuery, resource_target, ResourceBase};

pub trait ResourceCrud: ResourceQuery {
  type PartialConfig: Into<Self::Config> + Serialize;

  // ========
  //  CREATE
  // ========

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

  // ========
  //  UPDATE
  // ========

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
  ) -> Result<Document, mungos::mongodb::bson::ser::Error> {
    to_document(&config)
  }

  /// Run any required task after resource updated in database but
  /// before the request resolves.
  async fn post_update(
    updated: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()>;

  // ========
  //  RENAME
  // ========

  fn rename_operation() -> Operation;

  // ========
  //  DELETE
  // ========

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

// ========
//  CREATE
// ========

pub async fn create<R: ResourceCrud>(
  name: &str,
  mut config: R::PartialConfig,
  user: &User,
) -> anyhow::Result<Resource<R::Config, R::Info>> {
  if !R::user_can_create(user) {
    return Err(anyhow!(
      "User does not have permissions to create {}.",
      R::resource_type()
    ));
  }

  if name.is_empty() {
    return Err(anyhow!("Must provide non-empty name for resource."));
  }

  let name = to_komodo_name(name);

  if ObjectId::from_str(&name).is_ok() {
    return Err(anyhow!("valid ObjectIds cannot be used as names."));
  }

  // Ensure an existing resource with same name doesn't already exist
  // The database indexing also ensures this but doesn't give a good error message.
  if super::query::list_full_for_user::<R>(
    Default::default(),
    system_user(),
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

  R::validate_create_config(&mut config, user).await?;

  let resource = Resource::<R::Config, R::Info> {
    id: Default::default(),
    name,
    updated_at: start_ts,
    description: Default::default(),
    tags: Default::default(),
    config: config.into(),
    info: R::default_info().await?,
    base_permission: PermissionLevel::None,
  };

  let resource_id = R::coll()
    .insert_one(&resource)
    .await
    .with_context(|| {
      format!("failed to add {} to db", R::resource_type())
    })?
    .inserted_id
    .as_object_id()
    .context("inserted_id is not ObjectId")?
    .to_string();

  let resource = super::query::get::<R>(&resource_id).await?;

  create_permission::<R>(user, resource_id, PermissionLevel::Write)
    .await;

  let mut update = make_update(
    resource_target::<R>(resource.id.clone()),
    R::create_operation(),
    user,
  );
  update.start_ts = start_ts;
  update.push_simple_log(
    &format!("create {}", R::resource_type()),
    format!(
      "created {}\nid: {}\nname: {}",
      R::resource_type(),
      resource.id,
      resource.name
    ),
  );
  update.push_simple_log(
    "config",
    serde_json::to_string_pretty(&resource.config)
      .context("failed to serialize resource config to JSON")?,
  );

  R::post_create(&resource, &mut update).await?;

  update.finalize();
  add_update(update).await?;

  Ok(resource)
}

#[instrument]
pub async fn create_permission<R: ResourceBase>(
  user: &User,
  id: String,
  level: PermissionLevel,
) {
  // No need to actually create permissions for admins
  if user.admin {
    return;
  }
  let target = super::resource_target::<R>(id);
  if let Err(e) = db_client()
    .permissions
    .insert_one(Permission {
      id: Default::default(),
      user_target: UserTarget::User(user.id.clone()),
      resource_target: target.clone(),
      level,
    })
    .await
  {
    error!("failed to create permission for {target:?} | {e:#}");
  };
}

// ========
//  UPDATE
// ========

// ========
//  DELETE
// ========
