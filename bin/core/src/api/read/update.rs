use std::collections::HashMap;

use anyhow::Context;
use database::mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use komodo_client::{
  api::read::{GetUpdate, ListUpdates, ListUpdatesResponse},
  entities::{
    permission::PermissionLevel,
    update::{Update, UpdateListItem},
    user::User,
  },
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  permission::{
    check_user_target_access, user_resource_target_query,
  },
  state::db_client,
};

use super::ReadArgs;

const UPDATES_PER_PAGE: i64 = 100;

impl Resolve<ReadArgs> for ListUpdates {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListUpdatesResponse> {
    let query = user_resource_target_query(user, self.query).await?;

    let usernames = find_collect(&db_client().users, None, None)
      .await
      .context("failed to pull users from db")?
      .into_iter()
      .map(|u| (u.id, u.username))
      .collect::<HashMap<_, _>>();

    let updates = find_collect(
      &db_client().updates,
      query,
      FindOptions::builder()
        .sort(doc! { "start_ts": -1 })
        .skip(self.page as u64 * UPDATES_PER_PAGE as u64)
        .limit(UPDATES_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to pull updates from db")?
    .into_iter()
    .map(|u| {
      let username = if User::is_service_user(&u.operator) {
        u.operator.clone()
      } else {
        usernames
          .get(&u.operator)
          .cloned()
          .unwrap_or("unknown".to_string())
      };
      UpdateListItem {
        username,
        id: u.id,
        operation: u.operation,
        start_ts: u.start_ts,
        success: u.success,
        operator: u.operator,
        target: u.target,
        status: u.status,
        version: u.version,
        other_data: u.other_data,
      }
    })
    .collect::<Vec<_>>();

    let next_page = if updates.len() == UPDATES_PER_PAGE as usize {
      Some(self.page + 1)
    } else {
      None
    };

    Ok(ListUpdatesResponse { updates, next_page })
  }
}

impl Resolve<ReadArgs> for GetUpdate {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Update> {
    let update = find_one_by_id(&db_client().updates, &self.id)
      .await
      .context("failed to query to db")?
      .context("no update exists with given id")?;
    if user.admin || core_config().transparent_mode {
      return Ok(update);
    }
    check_user_target_access(
      &update.target,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    Ok(update)
  }
}
