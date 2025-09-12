use komodo_client::{
  api::write::*,
  entities::{
    action::Action, permission::PermissionLevel, update::Update,
  },
};
use resolver_api::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateAction {
  #[instrument(name = "CreateAction", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Action> {
    resource::create::<Action>(&self.name, self.config, user).await
  }
}

impl Resolve<WriteArgs> for CopyAction {
  #[instrument(name = "CopyAction", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Action> {
    let Action { config, .. } = get_check_permissions::<Action>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;
    resource::create::<Action>(&self.name, config.into(), user).await
  }
}

impl Resolve<WriteArgs> for UpdateAction {
  #[instrument(name = "UpdateAction", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Action> {
    Ok(resource::update::<Action>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameAction {
  #[instrument(name = "RenameAction", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Action>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for DeleteAction {
  #[instrument(name = "DeleteAction", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Action> {
    Ok(resource::delete::<Action>(&self.id, args).await?)
  }
}
