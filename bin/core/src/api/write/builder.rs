use komodo_client::{
  api::write::*,
  entities::{
    builder::Builder, permission::PermissionLevel, update::Update,
  },
};
use resolver_api::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateBuilder {
  #[instrument(name = "CreateBuilder", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Builder> {
    resource::create::<Builder>(&self.name, self.config, user).await
  }
}

impl Resolve<WriteArgs> for CopyBuilder {
  #[instrument(name = "CopyBuilder", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Builder> {
    let Builder { config, .. } = get_check_permissions::<Builder>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;
    resource::create::<Builder>(&self.name, config.into(), user).await
  }
}

impl Resolve<WriteArgs> for DeleteBuilder {
  #[instrument(name = "DeleteBuilder", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<Builder> {
    Ok(resource::delete::<Builder>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateBuilder {
  #[instrument(name = "UpdateBuilder", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Builder> {
    Ok(
      resource::update::<Builder>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameBuilder {
  #[instrument(name = "RenameBuilder", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Builder>(&self.id, &self.name, user).await?)
  }
}
