use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel, server_template::ServerTemplate,
  },
};
use mongo_indexed::Document;
use mungos::mongodb::bson::doc;
use resolver_api::Resolve;

use crate::{
  helpers::query::get_all_tags, resource, state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetServerTemplate {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetServerTemplateResponse> {
    Ok(
      resource::get_check_permissions::<ServerTemplate>(
        &self.server_template,
        user,
        PermissionLevel::Read,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListServerTemplates {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListServerTemplatesResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<ServerTemplate>(
        self.query, user, &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullServerTemplates {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullServerTemplatesResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<ServerTemplate>(
        self.query, user, &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetServerTemplatesSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetServerTemplatesSummaryResponse> {
    let query = match resource::get_resource_object_ids_for_user::<
      ServerTemplate,
    >(&user)
    .await?
    {
      Some(ids) => doc! {
        "_id": { "$in": ids }
      },
      None => Document::new(),
    };
    let total = db_client()
      .server_templates
      .count_documents(query)
      .await
      .context("failed to count all server template documents")?;
    let res = GetServerTemplatesSummaryResponse {
      total: total as u32,
    };
    Ok(res)
  }
}
