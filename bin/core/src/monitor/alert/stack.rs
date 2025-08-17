use std::collections::HashMap;

use komodo_client::entities::{
  ResourceTarget,
  alert::{Alert, AlertData, SeverityLevel},
  stack::{Stack, StackState},
};

use crate::{
  alert::send_alerts,
  resource,
  state::{action_states, db_client, stack_status_cache},
};

#[instrument(level = "debug")]
pub async fn alert_stacks(
  ts: i64,
  server_names: &HashMap<String, String>,
) {
  let action_states = action_states();
  let mut alerts = Vec::<Alert>::new();
  for status in stack_status_cache().get_list().await {
    // Don't alert if prev None
    let Some(prev) = status.prev else {
      continue;
    };

    // Don't alert if either prev or curr is Unknown.
    // This will happen if server is unreachable, so this would be redundant.
    if status.curr.state == StackState::Unknown
      || prev == StackState::Unknown
    {
      continue;
    }

    // Don't alert if deploying
    if action_states
      .stack
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
      let Ok(stack) =
        resource::get::<Stack>(&status.curr.id).await.inspect_err(
          |e| error!("failed to get stack from db | {e:#?}"),
        )
      else {
        continue;
      };
      if !stack.config.send_alerts {
        continue;
      }
      let target: ResourceTarget = (&stack).into();
      let data = AlertData::StackStateChange {
        id: status.curr.id.clone(),
        name: stack.name,
        server_name: server_names
          .get(&stack.config.server_id)
          .cloned()
          .unwrap_or(String::from("unknown")),
        server_id: stack.config.server_id,
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
    error!("failed to record stack status alerts to db | {e:#}");
  }
}
