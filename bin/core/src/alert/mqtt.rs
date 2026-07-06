use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use komodo_client::entities::alert::AlertDataVariant;
use rumqttc::{AsyncClient, MqttOptions, Packet, QoS};
use url::Url;

use super::*;

pub async fn send_alert(
  endpoint: &MqttAlerterEndpoint,
  alert: &Alert,
) -> anyhow::Result<()> {
  let VariablesAndSecrets { variables, secrets } =
    get_variables_and_secrets().await?;

  let mut broker_url = endpoint.broker_url.clone();
  let mut topic = endpoint.topic.clone();
  let mut username = endpoint.username.clone();
  let mut password = endpoint.password.clone();
  let mut client_id = endpoint.client_id.clone();

  let mut interpolator =
    Interpolator::new(Some(&variables), &secrets);

  let res = async {
    interpolator.interpolate_string(&mut broker_url)?;
    interpolator.interpolate_string(&mut topic)?;
    if let Some(value) = username.as_mut() {
      interpolator.interpolate_string(value)?;
    }
    if let Some(value) = password.as_mut() {
      interpolator.interpolate_string(value)?;
    }
    if let Some(value) = client_id.as_mut() {
      interpolator.interpolate_string(value)?;
    }
    let topic = topic
      .replace("{data.type}", &AlertDataVariant::from(&alert.data).to_string());

    send_message(
      &broker_url,
      &topic,
      username.as_deref(),
      password.as_deref(),
      client_id.as_deref(),
      endpoint.qos,
      endpoint.retain,
      alert,
    )
    .await
  }
  .await;

  res.map_err(|e| {
    let replacers = interpolator
      .secret_replacers
      .into_iter()
      .collect::<Vec<_>>();
    let sanitized_error =
      svi::replace_in_string(&format!("{e:?}"), &replacers);
    anyhow!("Error with publish to MQTT: {sanitized_error}")
  })
}

async fn send_message(
  broker_url: &str,
  topic: &str,
  username: Option<&str>,
  password: Option<&str>,
  client_id: Option<&str>,
  qos: u8,
  retain: bool,
  alert: &Alert,
) -> anyhow::Result<()> {
  if topic.trim().is_empty() {
    bail!("MQTT topic cannot be empty");
  }

  let parsed = Url::parse(broker_url).with_context(|| {
    format!("Invalid MQTT broker URL: {broker_url}")
  })?;
  let scheme = parsed.scheme();
  if !matches!(scheme, "mqtt" | "tcp") {
    bail!(
      "Unsupported MQTT broker URL scheme `{scheme}`. Use mqtt:// or tcp://"
    );
  }

  let host = parsed
    .host_str()
    .context("MQTT broker URL must include a host")?;
  let port = parsed.port().unwrap_or(1883);

  let client_id = client_id
    .filter(|id| !id.trim().is_empty())
    .map(ToOwned::to_owned)
    .unwrap_or_else(|| format!("komodo-alert-{}", uuid::Uuid::new_v4()));

  let mut options = MqttOptions::new(client_id, host, port);
  options.set_keep_alive(Duration::from_secs(10));

  let username = username.filter(|value| !value.trim().is_empty());
  let password = password.filter(|value| !value.trim().is_empty());
  let url_username = parsed.username();
  let url_password = parsed.password().filter(|value| !value.is_empty());
  match (username, password) {
    (Some(user), Some(pass)) => {
      options.set_credentials(user, pass);
    }
    (Some(user), None) => {
      options.set_credentials(user, "");
    }
    (None, Some(pass)) => {
      options.set_credentials("", pass);
    }
    (None, None) if !url_username.is_empty() => {
      options.set_credentials(url_username, url_password.unwrap_or(""));
    }
    _ => {}
  }

  let payload = serde_json::to_vec(alert)
    .context("Failed to serialize alert payload to JSON")?;
  let qos = to_qos(qos)?;

  let (client, mut eventloop) = AsyncClient::new(options, 10);
  client
    .publish(topic, qos, retain, payload)
    .await
    .context("Failed queuing MQTT publish")?;

  let poll_count = if qos == QoS::AtMostOnce { 2 } else { 4 };
  for _ in 0..poll_count {
    let event = tokio::time::timeout(
      Duration::from_secs(5),
      eventloop.poll(),
    )
    .await
    .context("Timed out waiting for MQTT broker response")?
    .context("MQTT event loop failed while publishing")?;

    if matches!(
      event,
      rumqttc::Event::Incoming(Packet::PubAck(_))
        | rumqttc::Event::Incoming(Packet::PubComp(_))
    ) {
      break;
    }
  }

  client.disconnect().await.ok();
  Ok(())
}

fn to_qos(qos: u8) -> anyhow::Result<QoS> {
  match qos {
    0 => Ok(QoS::AtMostOnce),
    1 => Ok(QoS::AtLeastOnce),
    2 => Ok(QoS::ExactlyOnce),
    _ => bail!("Invalid MQTT QoS `{qos}`. Must be 0, 1, or 2"),
  }
}
