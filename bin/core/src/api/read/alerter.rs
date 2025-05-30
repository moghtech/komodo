use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    alerter::{Alerter, AlerterListItem},
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

impl Resolve<ReadArgs> for GetAlerter {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Alerter> {
    Ok(
      get_check_permissions::<Alerter>(
        &self.alerter,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListAlerters {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<AlerterListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Alerter>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullAlerters {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullAlertersResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Alerter>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetAlertersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetAlertersSummaryResponse> {
    let query = match resource::get_resource_object_ids_for_user::<
      Alerter,
    >(user)
    .await?
    {
      Some(ids) => doc! {
        "_id": { "$in": ids }
      },
      None => Document::new(),
    };
    let total = db_client()
      .alerters
      .count_documents(query)
      .await
      .context("failed to count all alerter documents")?;
    let res = GetAlertersSummaryResponse {
      total: total as u32,
    };
    Ok(res)
  }
}
