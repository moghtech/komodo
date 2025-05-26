use formatting::format_serror;
use komodo_client::{
  api::execute::TestAlerter,
  entities::{
    alert::{Alert, AlertData, SeverityLevel},
    alerter::Alerter,
    komodo_timestamp,
    permission::PermissionLevel,
  },
};
use resolver_api::Resolve;

use crate::{
  alert::send_alert_to_alerter, helpers::update::update_update, permission::get_check_permissions,
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
