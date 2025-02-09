use komodo_client::{
  api::write::{
    CopyServerTemplate, CreateServerTemplate, DeleteServerTemplate,
    RenameServerTemplate, UpdateServerTemplate,
  },
  entities::{
    permission::PermissionLevel, server_template::ServerTemplate,
    update::Update,
  },
};
use resolver_api::Resolve;

use crate::resource;

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateServerTemplate {
  #[instrument(name = "CreateServerTemplate", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ServerTemplate> {
    Ok(
      resource::create::<ServerTemplate>(
        &self.name,
        self.config,
        &user,
      )
      .await?,
    )
  }
}

impl Resolve<WriteArgs> for CopyServerTemplate {
  #[instrument(name = "CopyServerTemplate", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ServerTemplate> {
    let ServerTemplate { config, .. } =
      resource::get_check_permissions::<ServerTemplate>(
        &self.id,
        &user,
        PermissionLevel::Write,
      )
      .await?;
    Ok(
      resource::create::<ServerTemplate>(
        &self.name,
        config.into(),
        &user,
      )
      .await?,
    )
  }
}

impl Resolve<WriteArgs> for DeleteServerTemplate {
  #[instrument(name = "DeleteServerTemplate", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<ServerTemplate> {
    Ok(resource::delete::<ServerTemplate>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateServerTemplate {
  #[instrument(name = "UpdateServerTemplate", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<ServerTemplate> {
    Ok(
      resource::update::<ServerTemplate>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameServerTemplate {
  #[instrument(name = "RenameServerTemplate", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(
      resource::rename::<ServerTemplate>(&self.id, &self.name, user)
        .await?,
    )
  }
}
