use komodo_client::{
  api::write::*,
  entities::{
    permission::PermissionLevel, procedure::Procedure, update::Update,
  },
};
use resolver_api::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateProcedure {
  #[instrument(name = "CreateProcedure", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateProcedureResponse> {
    resource::create::<Procedure>(&self.name, self.config, user).await
  }
}

impl Resolve<WriteArgs> for CopyProcedure {
  #[instrument(name = "CopyProcedure", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CopyProcedureResponse> {
    let Procedure { config, .. } =
      get_check_permissions::<Procedure>(
        &self.id,
        user,
        PermissionLevel::Write.into(),
      )
      .await?;
    resource::create::<Procedure>(&self.name, config.into(), user)
      .await
  }
}

impl Resolve<WriteArgs> for UpdateProcedure {
  #[instrument(name = "UpdateProcedure", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateProcedureResponse> {
    Ok(
      resource::update::<Procedure>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameProcedure {
  #[instrument(name = "RenameProcedure", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(
      resource::rename::<Procedure>(&self.id, &self.name, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for DeleteProcedure {
  #[instrument(name = "DeleteProcedure", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<DeleteProcedureResponse> {
    Ok(resource::delete::<Procedure>(&self.id, args).await?)
  }
}
