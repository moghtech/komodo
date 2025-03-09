use anyhow::Context;
use komodo_client::{
  api::read::{GetTag, ListTags},
  entities::tag::Tag,
};
use mongo_indexed::doc;
use mungos::{find::find_collect, mongodb::options::FindOptions};
use resolver_api::Resolve;

use crate::{helpers::query::get_tag, state::db_client};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetTag {
  async fn resolve(self, _: &ReadArgs) -> serror::Result<Tag> {
    Ok(get_tag(&self.tag).await?)
  }
}

impl Resolve<ReadArgs> for ListTags {
  async fn resolve(self, _: &ReadArgs) -> serror::Result<Vec<Tag>> {
    let res = find_collect(
      &db_client().tags,
      self.query,
      FindOptions::builder().sort(doc! { "name": 1 }).build(),
    )
    .await
    .context("failed to get tags from db")?;
    Ok(res)
  }
}
