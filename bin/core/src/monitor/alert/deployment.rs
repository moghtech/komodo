use std::collections::HashMap;

use komodo_client::entities::{
  ResourceTarget,
  alert::{Alert, AlertData, SeverityLevel},
  deployment::{Deployment, DeploymentState},
};

use crate::{
  alert::send_alerts,
  monitor::deployment_status_cache,
  resource,
  state::{action_states, db_client},
};

#[instrument(level = "debug")]
pub async fn alert_deployments(
  ts: i64,
  server_names: &HashMap<String, String>,
) {
  let mut alerts = Vec::<Alert>::new();
  let action_states = action_states();
  for status in deployment_status_cache().get_list().await {
    // Don't alert if prev None
    let Some(prev) = status.prev else {
      continue;
    };

    // Don't alert if either prev or curr is Unknown.
    // This will happen if server is unreachable, so this would be redundant.
    if status.curr.state == DeploymentState::Unknown
      || prev == DeploymentState::Unknown
    {
      continue;
    }

    // Don't alert if deploying
    if action_states
      .deployment
      .get(&status.curr.id)
      .await
      .map(|s| s.get().map(|s| s.deploying))
      .transpose()
      .ok()
      .flatten()
      .unwrap_or_default()
    {
      continue;
    }

    if status.curr.state != prev {
      // send alert
      let Ok(deployment) =
        resource::get::<Deployment>(&status.curr.id)
          .await
          .inspect_err(|e| {
            error!("failed to get deployment from db | {e:#?}")
          })
      else {
        continue;
      };
      if !deployment.config.send_alerts {
        continue;
      }
      let target: ResourceTarget = (&deployment).into();
      let data = AlertData::ContainerStateChange {
        id: status.curr.id.clone(),
        name: deployment.name,
        server_name: server_names
          .get(&deployment.config.server_id)
          .cloned()
          .unwrap_or(String::from("unknown")),
        server_id: deployment.config.server_id,
        from: prev,
        to: status.curr.state,
      };
      let alert = Alert {
        id: Default::default(),
        level: SeverityLevel::Warning,
        resolved: true,
        resolved_ts: ts.into(),
        target,
        data,
        ts,
      };
      alerts.push(alert);
    }
  }
  if alerts.is_empty() {
    return;
  }
  send_alerts(&alerts).await;
  let res = db_client().alerts.insert_many(alerts).await;
  if let Err(e) = res {
    error!("failed to record deployment status alerts to db | {e:#}");
  }
}
