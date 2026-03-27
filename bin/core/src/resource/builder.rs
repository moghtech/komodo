use anyhow::Context;
use database::mungos::mongodb::{
  Collection,
  bson::{Document, doc, to_document},
};
use indexmap::IndexSet;
use komodo_client::entities::{
  MergePartial, Operation, ResourceTarget, ResourceTargetVariant,
  builder::{
    Builder, BuilderConfig, BuilderConfigDiff, BuilderConfigVariant,
    BuilderListItem, BuilderListItemInfo, BuilderQuerySpecifics,
    PartialBuilderConfig, PartialServerBuilderConfig,
  },
  permission::{PermissionLevel, SpecificPermission},
  resource::Resource,
  server::Server,
  update::Update,
  user::User,
};

use crate::state::db_client;

impl super::KomodoResource for Builder {
  type Config = BuilderConfig;
  type PartialConfig = PartialBuilderConfig;
  type ConfigDiff = BuilderConfigDiff;
  type Info = ();
  type ListItem = BuilderListItem;
  type QuerySpecifics = BuilderQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Builder
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Builder(id.into())
  }

  fn creator_specific_permissions() -> IndexSet<SpecificPermission> {
    [SpecificPermission::Attach].into_iter().collect()
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().builders
  }

  async fn to_list_item(
    builder: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (builder_type, instance_type) = match builder.config {
      BuilderConfig::Url(_) => {
        (BuilderConfigVariant::Url.to_string(), None)
      }
      BuilderConfig::Server(config) => (
        BuilderConfigVariant::Server.to_string(),
        Some(config.server_id),
      ),
      BuilderConfig::Aws(config) => (
        BuilderConfigVariant::Aws.to_string(),
        Some(config.instance_type),
      ),
    };
    BuilderListItem {
      name: builder.name,
      id: builder.id,
      template: builder.template,
      tags: builder.tags,
      resource_type: ResourceTargetVariant::Builder,
      info: BuilderListItemInfo {
        builder_type,
        instance_type,
      },
    }
  }

  async fn busy(_id: &String) -> anyhow::Result<bool> {
    Ok(false)
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateBuilder
  }

  fn user_can_create(user: &User) -> bool {
    user.admin
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_create(
    _created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateBuilder
  }

  async fn validate_update_config(
    _id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  fn update_document(
    original: Resource<Self::Config, Self::Info>,
    config: Self::PartialConfig,
  ) -> Result<Document, database::mungos::mongodb::bson::ser::Error>
  {
    let config = original.config.merge_partial(config);
    to_document(&config)
  }

  async fn post_update(
    _updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameBuilder
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteBuilder
  }

  async fn pre_delete(
    resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    db_client()
      .builds
      .update_many(
        doc! { "config.builder_id": &resource.id },
        database::mungos::update::Update::Set(
          doc! { "config.builder_id": "" },
        ),
      )
      .await
      .context("failed to update_many builds on database")?;
    db_client()
      .repos
      .update_many(
        doc! { "config.builder_id": &resource.id },
        database::mungos::update::Update::Set(
          doc! { "config.builder_id": "" },
        ),
      )
      .await
      .context("failed to update_many repos on database")?;
    Ok(())
  }

  async fn post_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }
}

#[instrument("ValidateBuilderConfig", skip_all)]
async fn validate_config(
  config: &mut PartialBuilderConfig,
  user: &User,
) -> anyhow::Result<()> {
  match config {
    PartialBuilderConfig::Server(PartialServerBuilderConfig {
      server_id: Some(server_id),
    }) if !server_id.is_empty() => {
      let server = super::get_check_permissions::<Server>(
        server_id,
        user,
        PermissionLevel::Read.attach(),
      )
      .await?;
      *server_id = server.id;
    }
    _ => {}
  }
  Ok(())
}
