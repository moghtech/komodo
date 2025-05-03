use anyhow::anyhow;
use komodo_client::{
  api::write::{UpdateDescription, UpdateDescriptionResponse},
  entities::{
    ResourceTarget, action::Action, alerter::Alerter, build::Build,
    builder::Builder, deployment::Deployment, procedure::Procedure,
    repo::Repo, server::Server, stack::Stack, sync::ResourceSync,
  },
};
use resolver_api::Resolve;

use crate::resource;

use super::WriteArgs;

impl Resolve<WriteArgs> for UpdateDescription {
  #[instrument(name = "UpdateDescription", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateDescriptionResponse> {
    match self.target {
      ResourceTarget::System(_) => {
        return Err(
          anyhow!(
            "cannot update description of System resource target"
          )
          .into(),
        );
      }
      ResourceTarget::Server(id) => {
        resource::update_description::<Server>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Deployment(id) => {
        resource::update_description::<Deployment>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Build(id) => {
        resource::update_description::<Build>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Repo(id) => {
        resource::update_description::<Repo>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Builder(id) => {
        resource::update_description::<Builder>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Alerter(id) => {
        resource::update_description::<Alerter>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Procedure(id) => {
        resource::update_description::<Procedure>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Action(id) => {
        resource::update_description::<Action>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::ResourceSync(id) => {
        resource::update_description::<ResourceSync>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
      ResourceTarget::Stack(id) => {
        resource::update_description::<Stack>(
          &id,
          &self.description,
          user,
        )
        .await?;
      }
    }
    Ok(UpdateDescriptionResponse {})
  }
}
