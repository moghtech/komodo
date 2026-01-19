use komodo_client::{
  api::write::*,
  entities::{
    action::Action, permission::PermissionLevel, update::Update,
  },
};
use mogh_resolver::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateAction {
  #[instrument(
    "CreateAction",
    skip_all,
    fields(
      operator = user.id,
      action = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Action> {
    resource::create::<Action>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyAction {
  #[instrument(
    "CopyAction",
    skip_all,
    fields(
      operator = user.id,
      action = self.name,
      copy_action = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Action> {
    let Action { config, .. } = get_check_permissions::<Action>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;
    resource::create::<Action>(&self.name, config.into(), None, user)
      .await
  }
}

impl Resolve<WriteArgs> for UpdateAction {
  #[instrument(
    "UpdateAction",
    skip_all,
    fields(
      operator = user.id,
      action = self.id,
      update = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Action> {
    Ok(resource::update::<Action>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameAction {
  #[instrument(
    "RenameAction",
    skip_all,
    fields(
      operator = user.id,
      action = self.id,
      new_name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    Ok(resource::rename::<Action>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for DeleteAction {
  #[instrument(
    "DeleteAction",
    skip_all,
    fields(
      operator = user.id,
      action = self.id
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Action> {
    Ok(resource::delete::<Action>(&self.id, user).await?)
  }
}
