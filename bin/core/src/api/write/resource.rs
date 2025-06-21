use anyhow::anyhow;
use komodo_client::{
  api::write::{UpdateResourceMeta, UpdateResourceMetaResponse},
  entities::{
    ResourceTarget, action::Action, alerter::Alerter, build::Build,
    builder::Builder, deployment::Deployment, procedure::Procedure,
    repo::Repo, server::Server, stack::Stack, sync::ResourceSync,
  },
};
use resolver_api::Resolve;

use crate::resource::{self, ResourceMetaUpdate};

use super::WriteArgs;

impl Resolve<WriteArgs> for UpdateResourceMeta {
  #[instrument(name = "UpdateResourceMeta", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<UpdateResourceMetaResponse> {
    let meta = ResourceMetaUpdate {
      description: self.description,
      template: self.template,
      tags: self.tags,
    };
    match self.target {
      ResourceTarget::System(_) => {
        return Err(
          anyhow!("cannot update meta of System resource target")
            .into(),
        );
      }
      ResourceTarget::Server(id) => {
        resource::update_meta::<Server>(&id, meta, args).await?;
      }
      ResourceTarget::Deployment(id) => {
        resource::update_meta::<Deployment>(&id, meta, args).await?;
      }
      ResourceTarget::Build(id) => {
        resource::update_meta::<Build>(&id, meta, args).await?;
      }
      ResourceTarget::Repo(id) => {
        resource::update_meta::<Repo>(&id, meta, args).await?;
      }
      ResourceTarget::Builder(id) => {
        resource::update_meta::<Builder>(&id, meta, args).await?;
      }
      ResourceTarget::Alerter(id) => {
        resource::update_meta::<Alerter>(&id, meta, args).await?;
      }
      ResourceTarget::Procedure(id) => {
        resource::update_meta::<Procedure>(&id, meta, args).await?;
      }
      ResourceTarget::Action(id) => {
        resource::update_meta::<Action>(&id, meta, args).await?;
      }
      ResourceTarget::ResourceSync(id) => {
        resource::update_meta::<ResourceSync>(&id, meta, args)
          .await?;
      }
      ResourceTarget::Stack(id) => {
        resource::update_meta::<Stack>(&id, meta, args).await?;
      }
    }
    Ok(UpdateResourceMetaResponse {})
  }
}
