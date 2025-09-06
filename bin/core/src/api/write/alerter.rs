use komodo_client::{
  api::write::*,
  entities::{
    alerter::Alerter, permission::PermissionLevel, update::Update,
  },
};
use resolver_api::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::{WriteArgs, handle_resource_creation_error};

impl Resolve<WriteArgs> for CreateAlerter {
  #[instrument(name = "CreateAlerter", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Alerter> {
    match resource::create::<Alerter>(&self.name, self.config, user).await {
      Ok(alerter) => Ok(alerter),
      Err(e) => Err(handle_resource_creation_error(e))
    }
  }
}

impl Resolve<WriteArgs> for CopyAlerter {
  #[instrument(name = "CopyAlerter", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Alerter> {
    let Alerter { config, .. } = get_check_permissions::<Alerter>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;
    match resource::create::<Alerter>(&self.name, config.into(), user).await {
      Ok(alerter) => Ok(alerter),
      Err(e) => Err(handle_resource_creation_error(e))
    }
  }
}

impl Resolve<WriteArgs> for DeleteAlerter {
  #[instrument(name = "DeleteAlerter", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<Alerter> {
    Ok(resource::delete::<Alerter>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateAlerter {
  #[instrument(name = "UpdateAlerter", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Alerter> {
    Ok(
      resource::update::<Alerter>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameAlerter {
  #[instrument(name = "RenameAlerter", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Alerter>(&self.id, &self.name, user).await?)
  }
}
