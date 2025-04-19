use komodo_client::entities::{
  MergePartial, Operation, ResourceTarget, ResourceTargetVariant,
  resource::Resource,
  server_template::{
    PartialServerTemplateConfig, ServerTemplate,
    ServerTemplateConfig, ServerTemplateConfigDiff,
    ServerTemplateConfigVariant, ServerTemplateListItem,
    ServerTemplateListItemInfo, ServerTemplateQuerySpecifics,
  },
  update::Update,
  user::User,
};
use mungos::mongodb::{
  Collection,
  bson::{Document, to_document},
};

use crate::state::db_client;

impl super::KomodoResource for ServerTemplate {
  type Config = ServerTemplateConfig;
  type PartialConfig = PartialServerTemplateConfig;
  type ConfigDiff = ServerTemplateConfigDiff;
  type Info = ();
  type ListItem = ServerTemplateListItem;
  type QuerySpecifics = ServerTemplateQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::ServerTemplate
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::ServerTemplate(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().server_templates
  }

  async fn to_list_item(
    server_template: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let (template_type, instance_type) = match server_template.config
    {
      ServerTemplateConfig::Aws(config) => (
        ServerTemplateConfigVariant::Aws.to_string(),
        Some(config.instance_type),
      ),
      ServerTemplateConfig::Hetzner(config) => (
        ServerTemplateConfigVariant::Hetzner.to_string(),
        Some(config.server_type.as_ref().to_string()),
      ),
    };
    ServerTemplateListItem {
      name: server_template.name,
      id: server_template.id,
      tags: server_template.tags,
      resource_type: ResourceTargetVariant::ServerTemplate,
      info: ServerTemplateListItemInfo {
        provider: template_type.to_string(),
        instance_type,
      },
    }
  }

  async fn busy(_id: &String) -> anyhow::Result<bool> {
    Ok(false)
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateServerTemplate
  }

  fn user_can_create(user: &User) -> bool {
    user.admin
  }

  async fn validate_create_config(
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_create(
    _created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateServerTemplate
  }

  async fn validate_update_config(
    _id: &str,
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  fn update_document(
    original: Resource<Self::Config, Self::Info>,
    config: Self::PartialConfig,
  ) -> Result<Document, mungos::mongodb::bson::ser::Error> {
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
    Operation::RenameServerTemplate
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteServerTemplate
  }

  async fn pre_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }
}
