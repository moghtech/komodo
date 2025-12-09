use anyhow::Context;
use database::mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use komodo_client::{
  api::read::{
    GetAlert, GetAlertResponse, ListAlerts, ListAlertsResponse,
  },
  entities::permission::PermissionLevel,
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

const NUM_ALERTS_PER_PAGE: u64 = 100;

impl Resolve<ReadArgs> for ListAlerts {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListAlertsResponse> {
    // Alerts
    let query = user_resource_target_query(user, self.query)
      .await?
      .unwrap_or_default();

    let alerts = find_collect(
      &db_client().alerts,
      query,
      FindOptions::builder()
        .sort(doc! { "ts": -1 })
        .limit(NUM_ALERTS_PER_PAGE as i64)
        .skip(self.page * NUM_ALERTS_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to get alerts from db")?;

    let next_page = if alerts.len() < NUM_ALERTS_PER_PAGE as usize {
      None
    } else {
      Some((self.page + 1) as i64)
    };

    let res = ListAlertsResponse { next_page, alerts };

    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetAlert {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetAlertResponse> {
    let alert = find_one_by_id(&db_client().alerts, &self.id)
      .await
      .context("failed to query db for alert")?
      .context("no alert found with given id")?;
    if user.admin || core_config().transparent_mode {
      return Ok(alert);
    }
    check_user_target_access(
      &alert.target,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    Ok(alert)
  }
}
