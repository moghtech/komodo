use anyhow::Context;
use komodo_client::{
  api::read::{
    GetAlert, GetAlertResponse, ListAlerts, ListAlertsResponse,
  },
  entities::{
    deployment::Deployment, server::Server, stack::Stack,
    sync::ResourceSync,
  },
};
use mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use resolver_api::Resolve;

use crate::{
  config::core_config, permission::get_resource_ids_for_user,
  state::db_client,
};

use super::ReadArgs;

const NUM_ALERTS_PER_PAGE: u64 = 100;

impl Resolve<ReadArgs> for ListAlerts {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListAlertsResponse> {
    let mut query = self.query.unwrap_or_default();
    if !user.admin && !core_config().transparent_mode {
      let server_ids =
        get_resource_ids_for_user::<Server>(user).await?;
      let stack_ids =
        get_resource_ids_for_user::<Stack>(user).await?;
      let deployment_ids =
        get_resource_ids_for_user::<Deployment>(user).await?;
      let sync_ids =
        get_resource_ids_for_user::<ResourceSync>(user).await?;
      query.extend(doc! {
        "$or": [
          { "target.type": "Server", "target.id": { "$in": &server_ids } },
          { "target.type": "Stack", "target.id": { "$in": &stack_ids } },
          { "target.type": "Deployment", "target.id": { "$in": &deployment_ids } },
          { "target.type": "ResourceSync", "target.id": { "$in": &sync_ids } },
        ]
      });
    }

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
    _: &ReadArgs,
  ) -> serror::Result<GetAlertResponse> {
    Ok(
      find_one_by_id(&db_client().alerts, &self.id)
        .await
        .context("failed to query db for alert")?
        .context("no alert found with given id")?,
    )
  }
}
