use komodo_client::{
  api::write::*,
  entities::{
    permission::PermissionLevel, procedure::Procedure, update::Update,
  },
};
use mogh_resolver::Resolve;

use crate::{permission::get_check_permissions, resource};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateProcedure {
  #[instrument(
    "CreateProcedure",
    skip_all,
    fields(
      operator = user.id,
      procedure = self.name,
      config = serde_json::to_string(&self.config).unwrap()
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CreateProcedureResponse> {
    resource::create::<Procedure>(&self.name, self.config, None, user)
      .await
  }
}

impl Resolve<WriteArgs> for CopyProcedure {
  #[instrument(
    "CopyProcedure",
    skip_all,
    fields(
      operator = user.id,
      procedure = self.name,
      copy_procedure = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<CopyProcedureResponse> {
    let Procedure { config, .. } =
      get_check_permissions::<Procedure>(
        &self.id,
        user,
        PermissionLevel::Write.into(),
      )
      .await?;
    resource::create::<Procedure>(
      &self.name,
      config.into(),
      None,
      user,
    )
    .await
  }
}

impl Resolve<WriteArgs> for UpdateProcedure {
  #[instrument(
    "UpdateProcedure",
    skip_all,
    fields(
      operator = user.id,
      procedure = self.id,
      update = serde_json::to_string(&self.config).unwrap(),
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<UpdateProcedureResponse> {
    Ok(
      resource::update::<Procedure>(&self.id, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for RenameProcedure {
  #[instrument(
    "RenameProcedure",
    skip_all,
    fields(
      operator = user.id,
      procedure = self.id,
      new_name = self.name,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<Update> {
    Ok(
      resource::rename::<Procedure>(&self.id, &self.name, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for DeleteProcedure {
  #[instrument(
    "DeleteProcedure",
    skip_all,
    fields(
      operator = user.id,
      procedure = self.id
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> mogh_error::Result<DeleteProcedureResponse> {
    Ok(resource::delete::<Procedure>(&self.id, user).await?)
  }
}
