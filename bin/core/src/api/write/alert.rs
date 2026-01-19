use std::str::FromStr;

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::{doc, oid::ObjectId};
use komodo_client::{api::write::CloseAlert, entities::NoData};
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use mogh_error::AddStatusCodeError;

use crate::{api::write::WriteArgs, state::db_client};

impl Resolve<WriteArgs> for CloseAlert {
  #[instrument(
    "CloseAlert",
    skip_all,
    fields(
      operator = admin.id,
      alert_id = self.id,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> Result<Self::Response, Self::Error> {
    if !admin.admin {
      return Err(
        anyhow!("This call is admin only")
          .status_code(StatusCode::FORBIDDEN),
      );
    }
    db_client()
      .alerts
      .update_one(
        doc! { "_id": ObjectId::from_str(&self.id)? },
        doc! { "$set": { "resolved": true } },
      )
      .await
      .context("Failed to close Alert on database")?;
    Ok(NoData {})
  }
}
