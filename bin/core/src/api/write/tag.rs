use std::str::FromStr;

use anyhow::{Context, anyhow};
use database::mungos::{
  by_id::{delete_one_by_id, update_one_by_id},
  mongodb::bson::{doc, oid::ObjectId},
};
use komodo_client::{
  api::write::{CreateTag, DeleteTag, RenameTag, UpdateTagColor},
  entities::{
    action::Action, alerter::Alerter, build::Build, builder::Builder,
    deployment::Deployment, procedure::Procedure, repo::Repo,
    server::Server, stack::Stack, sync::ResourceSync, tag::Tag,
  },
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;

use crate::{
  config::core_config,
  helpers::query::{get_tag, get_tag_check_owner},
  resource,
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateTag {
  #[instrument(name = "CreateTag", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Tag> {
    if core_config().disable_non_admin_create && !user.admin {
      return Err(
        anyhow!("Non admins cannot create tags")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    if ObjectId::from_str(&self.name).is_ok() {
      return Err(
        anyhow!("Tag name cannot be ObjectId")
          .status_code(StatusCode::BAD_REQUEST),
      );
    }

    let mut tag = Tag {
      id: Default::default(),
      name: self.name,
      color: self.color.unwrap_or_default(),
      owner: user.id.clone(),
    };

    tag.id = db_client()
      .tags
      .insert_one(&tag)
      .await
      .context("failed to create tag on db")?
      .inserted_id
      .as_object_id()
      .context("inserted_id is not ObjectId")?
      .to_string();

    Ok(tag)
  }
}

impl Resolve<WriteArgs> for RenameTag {
  #[instrument(name = "RenameTag", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Tag> {
    if ObjectId::from_str(&self.name).is_ok() {
      return Err(anyhow!("tag name cannot be ObjectId").into());
    }

    get_tag_check_owner(&self.id, user).await?;

    update_one_by_id(
      &db_client().tags,
      &self.id,
      doc! { "$set": { "name": self.name } },
      None,
    )
    .await
    .context("failed to rename tag on db")?;

    Ok(get_tag(&self.id).await?)
  }
}

impl Resolve<WriteArgs> for UpdateTagColor {
  #[instrument(name = "UpdateTagColor", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Tag> {
    let tag = get_tag_check_owner(&self.tag, user).await?;

    update_one_by_id(
      &db_client().tags,
      &tag.id,
      doc! { "$set": { "color": self.color.as_ref() } },
      None,
    )
    .await
    .context("failed to rename tag on db")?;

    Ok(get_tag(&self.tag).await?)
  }
}

impl Resolve<WriteArgs> for DeleteTag {
  #[instrument(name = "DeleteTag", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Tag> {
    let tag = get_tag_check_owner(&self.id, user).await?;

    tokio::try_join!(
      resource::remove_tag_from_all::<Server>(&self.id),
      resource::remove_tag_from_all::<Stack>(&self.id),
      resource::remove_tag_from_all::<Deployment>(&self.id),
      resource::remove_tag_from_all::<Build>(&self.id),
      resource::remove_tag_from_all::<Repo>(&self.id),
      resource::remove_tag_from_all::<Procedure>(&self.id),
      resource::remove_tag_from_all::<Action>(&self.id),
      resource::remove_tag_from_all::<ResourceSync>(&self.id),
      resource::remove_tag_from_all::<Builder>(&self.id),
      resource::remove_tag_from_all::<Alerter>(&self.id),
    )?;

    delete_one_by_id(&db_client().tags, &self.id, None).await?;

    Ok(tag)
  }
}
