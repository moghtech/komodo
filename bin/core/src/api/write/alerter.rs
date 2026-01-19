use komodo_client::{
  api::write::*,
  entities::{
    alerter::Alerter, permission::PermissionLevel, update::Update,
  },
};
use mogh_resolver::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateAlerter {
  #[instrument(
    "CreateAlerter",
    skip_all,
    fields(
      operator = user.id,
      alerter = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Alerter> {
    resource::create::<Alerter>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyAlerter {
  #[instrument(
    "CopyAlerter",
    skip_all,
    fields(
      operator = user.id,
      alerter = self.name,
      copy_alerter = self.id,
    )
  )]
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
    resource::create::<Alerter>(&self.name, config.into(), None, user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteAlerter {
  #[instrument(
    "DeleteAlerter",
    skip_all,
    fields(
      operator = user.id,
      alerter = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Alerter> {
    Ok(resource::delete::<Alerter>(&self.id, user).await?)
  }
}

impl Resolve<WriteArgs> for UpdateAlerter {
  #[instrument(
    "UpdateAlerter",
    skip_all,
    fields(
      operator = user.id,
      alerter = self.id,
      update = serde_json::to_string(&self.config).unwrap()
    )
  )]
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
  #[instrument(
    "RenameAlerter",
    skip_all,
    fields(
      operator = user.id,
      alerter = self.id,
      new_name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Alerter>(&self.id, &self.name, user).await?)
  }
}
