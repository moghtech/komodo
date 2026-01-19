use komodo_client::{
  api::write::*,
  entities::{
    permission::PermissionLevel, swarm::Swarm, update::Update,
  },
};
use mogh_resolver::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateSwarm {
  #[instrument(
    "CreateSwarm",
    skip_all,
    fields(
      operator = user.id,
      swarm = self.name,
      config = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Swarm> {
    resource::create::<Swarm>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopySwarm {
  #[instrument(
    "CopySwarm",
    skip_all,
    fields(
      operator = user.id,
      swarm = self.name,
      copy_swarm = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Swarm> {
    let Swarm { config, .. } = get_check_permissions::<Swarm>(
      &self.id,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    resource::create::<Swarm>(&self.name, config.into(), None, user)
      .await
  }
}

impl Resolve<WriteArgs> for DeleteSwarm {
  #[instrument(
    "DeleteSwarm",
    skip_all,
    fields(
      operator = user.id,
      swarm = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Swarm> {
    Ok(resource::delete::<Swarm>(&self.id, user).await?)
  }
}

impl Resolve<WriteArgs> for UpdateSwarm {
  #[instrument(
    "UpdateSwarm",
    skip_all,
    fields(
      operator = user.id,
      swarm = self.id,
      update = serde_json::to_string(&self.config).unwrap()
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Swarm> {
    Ok(resource::update::<Swarm>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameSwarm {
  #[instrument(
    "RenameSwarm",
    skip_all,
    fields(
      operator = user.id,
      swarm = self.id,
      new_name = self.name
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Swarm>(&self.id, &self.name, user).await?)
  }
}
