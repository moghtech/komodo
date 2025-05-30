use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    builder::{Builder, BuilderListItem},
    permission::PermissionLevel,
  },
};
use mongo_indexed::Document;
use mungos::mongodb::bson::doc;
use resolver_api::Resolve;

use crate::{
  helpers::query::get_all_tags, permission::get_check_permissions,
  resource, state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetBuilder {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Builder> {
    Ok(
      get_check_permissions::<Builder>(
        &self.builder,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListBuilders {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<BuilderListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Builder>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullBuilders {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullBuildersResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Builder>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetBuildersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetBuildersSummaryResponse> {
    let query = match resource::get_resource_object_ids_for_user::<
      Builder,
    >(user)
    .await?
    {
      Some(ids) => doc! {
        "_id": { "$in": ids }
      },
      None => Document::new(),
    };
    let total = db_client()
      .builders
      .count_documents(query)
      .await
      .context("failed to count all builder documents")?;
    let res = GetBuildersSummaryResponse {
      total: total as u32,
    };
    Ok(res)
  }
}
