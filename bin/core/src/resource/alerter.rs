use derive_variants::ExtractVariant;
use komodo_client::entities::{
  alerter::{
    Alerter, AlerterConfig, AlerterConfigDiff, AlerterListItem,
    AlerterListItemInfo, AlerterQuerySpecifics, PartialAlerterConfig,
  }, resource::Resource, update::Update, user::User, Operation, ResourceTarget, ResourceTargetVariant
};
use mungos::mongodb::Collection;

use crate::state::db_client;

impl super::KomodoResource for Alerter {
  type Config = AlerterConfig;
  type PartialConfig = PartialAlerterConfig;
  type ConfigDiff = AlerterConfigDiff;
  type Info = ();
  type ListItem = AlerterListItem;
  type QuerySpecifics = AlerterQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Alerter
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Alerter(id.into())
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().alerters
  }

  async fn to_list_item(
    alerter: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    AlerterListItem {
      name: alerter.name,
      id: alerter.id,
      tags: alerter.tags,
      resource_type: ResourceTargetVariant::Alerter,
      info: AlerterListItemInfo {
        endpoint_type: alerter.config.endpoint.extract_variant(),
        enabled: alerter.config.enabled,
      },
    }
  }

  async fn busy(_id: &String) -> anyhow::Result<bool> {
    Ok(false)
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateAlerter
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
    Operation::UpdateAlerter
  }

  async fn validate_update_config(
    _id: &str,
    _config: &mut Self::PartialConfig,
    _user: &User,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  async fn post_update(
    _updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameAlerter
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteAlerter
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
