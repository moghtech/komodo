use std::{collections::HashMap, str::FromStr, sync::OnceLock};

use anyhow::Context;
use database::mongo_indexed::Indexed;
use database::mungos::{
  bulk_update::{self, BulkUpdate},
  find::find_collect,
  mongodb::bson::{doc, oid::ObjectId, to_bson},
};
use komodo_client::entities::{
  ResourceTarget,
  alert::{Alert, AlertData, AlertDataVariant, SeverityLevel},
  komodo_timestamp,
  swarm::{Swarm, SwarmState},
};

use crate::{
  alert::send_alerts,
  helpers::maintenance::is_in_maintenance,
  monitor::alert::AlertBuffer,
  state::{db_client, swarm_status_cache},
};

type SendAlerts = bool;
type OpenAlertMap<T = AlertDataVariant> =
  HashMap<ResourceTarget, HashMap<T, Alert>>;

/// Global alert buffer instance
fn alert_buffer() -> &'static AlertBuffer {
  static BUFFER: OnceLock<AlertBuffer> = OnceLock::new();
  BUFFER.get_or_init(AlertBuffer::new)
}

pub async fn alert_swarms(
  ts: i64,
  mut swarms: HashMap<String, Swarm>,
) {
  let swarm_statuses = swarm_status_cache().get_values().await;

  let open_alerts = match get_open_alerts().await {
    Ok(alerts) => alerts,
    Err(e) => {
      error!("{e:#}");
      return;
    }
  };

  let mut alerts_to_open = Vec::<(Alert, SendAlerts)>::new();
  let mut alerts_to_update = Vec::<(Alert, SendAlerts)>::new();
  let mut alerts_to_close = Vec::<(Alert, SendAlerts)>::new();

  let buffer = alert_buffer();

  for swarm_status in swarm_statuses {
    let Some(swarm) = swarms.remove(&swarm_status.id) else {
      continue;
    };
    let swarm_alerts = open_alerts
      .get(&ResourceTarget::Swarm(swarm_status.id.clone()));

    // Check if swarm is in maintenance mode
    let in_maintenance =
      is_in_maintenance(&swarm.config.maintenance_windows, ts);

    // ===================
    // SWARM HEALTH
    // ===================
    let health_alert = swarm_alerts.as_ref().and_then(|alerts| {
      alerts.get(&AlertDataVariant::SwarmUnhealthy)
    });
    match (
      swarm_status.state,
      health_alert,
      swarm.config.send_unhealthy_alerts,
    ) {
      (
        SwarmState::Unhealthy
        | SwarmState::Down
        | SwarmState::Unknown,
        None,
        false,
      ) => {}
      (
        SwarmState::Unhealthy
        | SwarmState::Down
        | SwarmState::Unknown,
        None,
        true,
      ) => {
        // Only open unhealthy alert if unhealthy alerts enabled and not in maintenance and buffer is ready
        if !in_maintenance
          && buffer.ready_to_open(
            swarm_status.id.clone(),
            AlertDataVariant::SwarmUnhealthy,
          )
        {
          let alert = Alert {
            id: Default::default(),
            ts,
            resolved: false,
            resolved_ts: None,
            level: SeverityLevel::Critical,
            target: ResourceTarget::Swarm(swarm_status.id.clone()),
            data: AlertData::SwarmUnhealthy {
              id: swarm_status.id.clone(),
              name: swarm.name.clone(),
              err: swarm_status.err.clone(),
            },
          };
          alerts_to_open
            .push((alert, swarm.config.send_unhealthy_alerts))
        }
      }
      (
        SwarmState::Unhealthy
        | SwarmState::Down
        | SwarmState::Unknown,
        Some(alert),
        true,
      ) => {
        // update alert err
        let mut alert = alert.clone();
        let (id, name) = match alert.data {
          AlertData::SwarmUnhealthy { id, name, .. } => (id, name),
          data => {
            error!(
              "got incorrect alert data in SwarmStatus handler. got {data:?}"
            );
            continue;
          }
        };
        alert.data = AlertData::SwarmUnhealthy {
          id,
          name,
          err: swarm_status.err.clone(),
        };

        // Never send this alert, severity is always 'Critical'
        alerts_to_update.push((alert, false));
      }

      (
        SwarmState::Unhealthy
        | SwarmState::Down
        | SwarmState::Unknown,
        Some(alert),
        false,
      ) => {
        alerts_to_close
          .push((alert.clone(), swarm.config.send_unhealthy_alerts));
      }

      // Close an open alert
      (SwarmState::Healthy, Some(alert), _) => {
        alerts_to_close
          .push((alert.clone(), swarm.config.send_unhealthy_alerts));
      }
      (SwarmState::Healthy, None, _) => buffer.reset(
        swarm_status.id.clone(),
        AlertDataVariant::SwarmUnhealthy,
      ),
    }
  }

  tokio::join!(
    open_new_alerts(&alerts_to_open),
    update_alerts(&alerts_to_update),
    resolve_alerts(&alerts_to_close),
  );
}

async fn open_new_alerts(alerts: &[(Alert, SendAlerts)]) {
  if alerts.is_empty() {
    return;
  }

  let db = db_client();

  let open = || async {
    let ids = db
      .alerts
      .insert_many(alerts.iter().map(|(alert, _)| alert))
      .await?
      .inserted_ids
      .into_iter()
      .filter_map(|(index, id)| {
        alerts.get(index)?.1.then(|| id.as_object_id())
      })
      .flatten()
      .collect::<Vec<_>>();
    anyhow::Ok(ids)
  };

  let ids_to_send = match open().await {
    Ok(ids) => ids,
    Err(e) => {
      error!("failed to open alerts on db | {e:?}");
      return;
    }
  };

  let alerts = match find_collect(
    &db.alerts,
    doc! { "_id": { "$in": ids_to_send } },
    None,
  )
  .await
  {
    Ok(alerts) => alerts,
    Err(e) => {
      error!("failed to pull created alerts from mongo | {e:?}");
      return;
    }
  };

  send_alerts(&alerts).await
}

async fn update_alerts(alerts: &[(Alert, SendAlerts)]) {
  if alerts.is_empty() {
    return;
  }

  let open = || async {
    let updates = alerts.iter().map(|(alert, _)| {
        let update = BulkUpdate {
          query: doc! { "_id": ObjectId::from_str(&alert.id).context("failed to convert alert id to ObjectId")? },
          update: doc! { "$set": to_bson(alert).context("failed to convert alert to bson")? }
        };
        anyhow::Ok(update)
      })
      .filter_map(|update| match update {
        Ok(update) => Some(update),
        Err(e) => {
          warn!("failed to generate bulk update for alert | {e:#}");
          None
        }
      }).collect::<Vec<_>>();

    bulk_update::bulk_update(
      &db_client().db,
      Alert::default_collection_name(),
      &updates,
      false,
    )
    .await
    .context("failed to bulk update alerts")?;

    anyhow::Ok(())
  };

  let alerts = alerts
    .iter()
    .filter(|(_, send)| *send)
    .map(|(alert, _)| alert)
    .cloned()
    .collect::<Vec<_>>();

  let (res, _) = tokio::join!(open(), send_alerts(&alerts));

  if let Err(e) = res {
    error!("failed to create alerts on db | {e:#}");
  }
}

async fn resolve_alerts(alerts: &[(Alert, SendAlerts)]) {
  if alerts.is_empty() {
    return;
  }

  let close = || async move {
    let alert_ids = alerts
      .iter()
      .map(|(alert, _)| {
        ObjectId::from_str(&alert.id)
          .context("failed to convert alert id to ObjectId")
      })
      .collect::<anyhow::Result<Vec<_>>>()?;

    db_client()
      .alerts
      .update_many(
        doc! { "_id": { "$in": &alert_ids } },
        doc! {
          "$set": {
            "resolved": true,
            "resolved_ts": komodo_timestamp()
          }
        },
      )
      .await
      .context("failed to resolve alerts on db")
      .inspect_err(|e| warn!("{e:#}"))
      .ok();

    let ts = komodo_timestamp();

    let closed = alerts
      .iter()
      .filter(|(_, send)| *send)
      .map(|(alert, _)| {
        let mut alert = alert.clone();

        alert.resolved = true;
        alert.resolved_ts = Some(ts);
        alert.level = SeverityLevel::Ok;

        alert
      })
      .collect::<Vec<_>>();

    send_alerts(&closed).await;

    anyhow::Ok(())
  };

  if let Err(e) = close().await {
    error!("failed to resolve alerts | {e:#?}");
  }
}

async fn get_open_alerts() -> anyhow::Result<OpenAlertMap> {
  let alerts = find_collect(
    &db_client().alerts,
    doc! { "resolved": false },
    None,
  )
  .await
  .context("failed to get open alerts from db")?;

  let mut map = OpenAlertMap::new();

  for alert in alerts {
    let inner = map.entry(alert.target.clone()).or_default();
    inner.insert((&alert.data).into(), alert);
  }

  Ok(map)
}
