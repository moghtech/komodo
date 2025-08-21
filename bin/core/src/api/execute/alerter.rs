use anyhow::{Context, anyhow};
use formatting::format_serror;
use futures::{TryStreamExt, stream::FuturesUnordered};
use komodo_client::{
  api::execute::{SendAlert, TestAlerter},
  entities::{
    alert::{Alert, AlertData, AlertDataVariant, SeverityLevel},
    alerter::Alerter,
    komodo_timestamp,
    permission::PermissionLevel,
  },
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;

use crate::{
  alert::send_alert_to_alerter, helpers::update::update_update,
  permission::get_check_permissions, resource::list_full_for_user,
};

use super::ExecuteArgs;

impl Resolve<ExecuteArgs> for TestAlerter {
  #[instrument(name = "TestAlerter", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let alerter = get_check_permissions::<Alerter>(
      &self.alerter,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    let mut update = update.clone();

    if !alerter.config.enabled {
      update.push_error_log(
        "Test Alerter",
        String::from(
          "Alerter is disabled. Enable the Alerter to send alerts.",
        ),
      );
      update.finalize();
      update_update(update.clone()).await?;
      return Ok(update);
    }

    let ts = komodo_timestamp();

    let alert = Alert {
      id: Default::default(),
      ts,
      resolved: true,
      level: SeverityLevel::Ok,
      target: update.target.clone(),
      data: AlertData::Test {
        id: alerter.id.clone(),
        name: alerter.name.clone(),
      },
      resolved_ts: Some(ts),
    };

    if let Err(e) = send_alert_to_alerter(&alerter, &alert).await {
      update.push_error_log("Test Alerter", format_serror(&e.into()));
    } else {
      update.push_simple_log("Test Alerter", String::from("Alert sent successfully. It should be visible at your alerting destination."));
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}

//

impl Resolve<ExecuteArgs> for SendAlert {
  #[instrument(name = "SendAlert", skip(user, update), fields(user_id = user.id, update_id = update.id))]
  async fn resolve(
    self,
    ExecuteArgs { user, update }: &ExecuteArgs,
  ) -> Result<Self::Response, Self::Error> {
    let alerters = list_full_for_user::<Alerter>(
      Default::default(),
      user,
      PermissionLevel::Execute.into(),
      &[],
    )
    .await?
    .into_iter()
    .filter(|a| {
      a.config.enabled
        && (self.alerters.is_empty()
          || self.alerters.contains(&a.name)
          || self.alerters.contains(&a.id))
        && (a.config.alert_types.is_empty()
          || a.config.alert_types.contains(&AlertDataVariant::Custom))
    })
    .collect::<Vec<_>>();

    if alerters.is_empty() {
      return Err(anyhow!(
        "Could not find any valid alerters to send to, this required Execute permissions on the Alerter"
      ).status_code(StatusCode::BAD_REQUEST));
    }

    let mut update = update.clone();

    let ts = komodo_timestamp();

    let alert = Alert {
      id: Default::default(),
      ts,
      resolved: true,
      level: self.level,
      target: update.target.clone(),
      data: AlertData::Custom {
        message: self.message,
        details: self.details,
      },
      resolved_ts: Some(ts),
    };

    update.push_simple_log(
      "Send alert",
      serde_json::to_string_pretty(&alert)
        .context("Failed to serialize alert to JSON")?,
    );

    if let Err(e) = alerters
      .iter()
      .map(|alerter| send_alert_to_alerter(alerter, &alert))
      .collect::<FuturesUnordered<_>>()
      .try_collect::<Vec<_>>()
      .await
    {
      update.push_error_log("Send Error", format_serror(&e.into()));
    };

    update.finalize();
    update_update(update.clone()).await?;

    Ok(update)
  }
}
