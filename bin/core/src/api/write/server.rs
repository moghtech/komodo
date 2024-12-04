use formatting::format_serror;
use komodo_client::{
  api::write::*,
  entities::{
    permission::PermissionLevel,
    server::Server,
    update::{Update, UpdateStatus},
    Operation,
  },
};
use periphery_client::api;
use resolver_api::Resolve;

use crate::{
  helpers::{
    periphery_client,
    update::{add_update, make_update, update_update},
  },
  resource,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateServer {
  #[instrument(name = "CreateServer", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Server> {
    Ok(
      resource::create::<Server>(&self.name, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for DeleteServer {
  #[instrument(name = "DeleteServer", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Server> {
    Ok(resource::delete::<Server>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateServer {
  #[instrument(name = "UpdateServer", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Server> {
    Ok(resource::update::<Server>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameServer {
  #[instrument(name = "RenameServer", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Server>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for CreateNetwork {
  #[instrument(name = "CreateNetwork", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    let server = resource::get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Write,
    )
    .await?;

    let periphery = periphery_client(&server)?;

    let mut update =
      make_update(&server, Operation::CreateNetwork, &user);
    update.status = UpdateStatus::InProgress;
    update.id = add_update(update.clone()).await?;

    match periphery
      .request(api::network::CreateNetwork {
        name: self.name,
        driver: None,
      })
      .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "create network",
        format_serror(&e.context("failed to create network").into()),
      ),
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
