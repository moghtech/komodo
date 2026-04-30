use std::collections::HashMap;

use anyhow::{Context, anyhow};
use database::mungos::{find::find_collect, mongodb::bson::doc};
use futures_util::future::join_all;
use interpolate::Interpolator;
use komodo_client::entities::{
  ResourceTargetVariant,
  alert::{Alert, AlertData, AlertDataVariant, SeverityLevel},
  alerter::*,
  deployment::DeploymentState,
  komodo_timestamp,
  stack::StackState,
};

use crate::helpers::query::get_variables_and_secrets;
use crate::helpers::{
  maintenance::is_in_maintenance, query::VariablesAndSecrets,
};
use crate::{config::core_config, state::db_client};

mod discord;
mod ntfy;
mod pushover;
mod slack;

pub async fn send_alerts(alerts: &[Alert]) {
  if alerts.is_empty() {
    return;
  }

  let Ok(alerters) = find_collect(
    &db_client().alerters,
    doc! { "config.enabled": true },
    None,
  )
  .await
  .inspect_err(|e| {
    error!(
      "ERROR sending alerts | failed to get alerters from db | {e:#}"
    )
  }) else {
    return;
  };

  let handles = alerts
    .iter()
    .map(|alert| send_alert_to_alerters(&alerters, alert));

  join_all(handles).await;
}

async fn send_alert_to_alerters(alerters: &[Alerter], alert: &Alert) {
  if alerters.is_empty() {
    return;
  }

  let handles = alerters
    .iter()
    .map(|alerter| send_alert_to_alerter(alerter, alert));

  join_all(handles)
    .await
    .into_iter()
    .filter_map(|res| res.err())
    .for_each(|e| error!("{e:#}"));
}

pub async fn send_alert_to_alerter(
  alerter: &Alerter,
  alert: &Alert,
) -> anyhow::Result<()> {
  // Don't send if not enabled
  if !alerter.config.enabled {
    return Ok(());
  }

  if is_in_maintenance(
    &alerter.config.maintenance_windows,
    komodo_timestamp(),
  ) {
    return Ok(());
  }

  let alert_variant: AlertDataVariant = (&alert.data).into();

  // In the test case, we don't want the filters inside this
  // block to stop the test from being sent to the alerting endpoint.
  if alert_variant != AlertDataVariant::Test {
    // Don't send if alert type not configured on the alerter
    if !alerter.config.alert_types.is_empty()
      && !alerter.config.alert_types.contains(&alert_variant)
    {
      return Ok(());
    }

    // Don't send if resource is in the blacklist
    if alerter.config.except_resources.contains(&alert.target) {
      return Ok(());
    }

    // Don't send if whitelist configured and target is not included
    if !alerter.config.resources.is_empty()
      && !alerter.config.resources.contains(&alert.target)
    {
      return Ok(());
    }
  }

  match &alerter.config.endpoint {
    AlerterEndpoint::Custom(CustomAlerterEndpoint {
      url,
      headers,
    }) => send_custom_alert(url, headers, alert).await.with_context(
      || {
        format!(
          "Failed to send alert to Custom Alerter {}",
          alerter.name
        )
      },
    ),
    AlerterEndpoint::Slack(SlackAlerterEndpoint { url }) => {
      slack::send_alert(url, alert).await.with_context(|| {
        format!(
          "Failed to send alert to Slack Alerter {}",
          alerter.name
        )
      })
    }
    AlerterEndpoint::Discord(DiscordAlerterEndpoint { url }) => {
      discord::send_alert(url, alert).await.with_context(|| {
        format!(
          "Failed to send alert to Discord Alerter {}",
          alerter.name
        )
      })
    }
    AlerterEndpoint::Ntfy(NtfyAlerterEndpoint { url, email }) => {
      ntfy::send_alert(url, email.as_deref(), alert)
        .await
        .with_context(|| {
          format!(
            "Failed to send alert to ntfy Alerter {}",
            alerter.name
          )
        })
    }
    AlerterEndpoint::Pushover(PushoverAlerterEndpoint { url }) => {
      pushover::send_alert(url, alert).await.with_context(|| {
        format!(
          "Failed to send alert to Pushover Alerter {}",
          alerter.name
        )
      })
    }
  }
}

async fn send_custom_alert(
  url: &str,
  headers: &HashMap<String, String>,
  alert: &Alert,
) -> anyhow::Result<()> {
  let VariablesAndSecrets { variables, secrets } =
    get_variables_and_secrets().await?;
  send_custom_alert_inner(url, headers, alert, &variables, &secrets)
    .await
}

async fn send_custom_alert_inner(
  url: &str,
  headers: &HashMap<String, String>,
  alert: &Alert,
  variables: &HashMap<String, String>,
  secrets: &HashMap<String, String>,
) -> anyhow::Result<()> {
  let mut interpolator = Interpolator::new(Some(variables), secrets);
  let (url_interpolated, headers) = interpolate_custom_request_parts(
    url,
    headers,
    &mut interpolator,
  )?;

  let mut request =
    reqwest::Client::new().post(url_interpolated).json(alert);

  for (header, value) in headers {
    request = request.header(&header, &value);
  }

  let res = request
    .send()
    .await
    .map_err(|e| sanitize_request_error(&interpolator, &e))
    .context("failed at post request to alerter")?;
  let status = res.status();
  if !status.is_success() {
    let text = res
      .text()
      .await
      .context("failed to get response text on alerter response")?;
    let text = sanitize_interpolated_text(&interpolator, &text);
    return Err(anyhow!(
      "post to alerter failed | {status} | {text}"
    ));
  }
  Ok(())
}

fn interpolate_custom_request_parts(
  url: &str,
  headers: &HashMap<String, String>,
  interpolator: &mut Interpolator,
) -> anyhow::Result<(String, Vec<(String, String)>)> {
  let mut url_interpolated = url.to_string();
  interpolator.interpolate_string(&mut url_interpolated)?;

  let headers = headers
    .iter()
    .map(|(header, value)| {
      let mut header = header.to_string();
      let mut value = value.to_string();
      interpolator.interpolate_string(&mut header)?;
      interpolator.interpolate_string(&mut value)?;
      anyhow::Ok((header, value))
    })
    .collect::<anyhow::Result<Vec<_>>>()?;
  let mut headers = headers;
  headers.sort_by(|left, right| left.0.cmp(&right.0));

  Ok((url_interpolated, headers))
}

fn sanitize_request_error(
  interpolator: &Interpolator,
  error: &impl std::fmt::Debug,
) -> anyhow::Error {
  let sanitized_error =
    sanitize_interpolated_text(interpolator, &format!("{error:?}"));
  anyhow::Error::msg(format!("Error with request: {sanitized_error}"))
}

fn sanitize_interpolated_text(
  interpolator: &Interpolator,
  text: &str,
) -> String {
  let replacers = interpolator
    .secret_replacers
    .iter()
    .cloned()
    .collect::<Vec<_>>();
  svi::replace_in_string(text, &replacers)
}

fn fmt_region(region: &Option<String>) -> String {
  match region {
    Some(region) => format!(" ({region})"),
    None => String::new(),
  }
}

fn fmt_docker_container_state(state: &DeploymentState) -> String {
  match state {
    DeploymentState::Running => String::from("Running ▶️"),
    DeploymentState::Exited => String::from("Exited 🛑"),
    DeploymentState::Restarting => String::from("Restarting 🔄"),
    DeploymentState::NotDeployed => String::from("Not Deployed"),
    _ => state.to_string(),
  }
}

fn fmt_stack_state(state: &StackState) -> String {
  match state {
    StackState::Running => String::from("Running ▶️"),
    StackState::Stopped => String::from("Stopped 🛑"),
    StackState::Restarting => String::from("Restarting 🔄"),
    StackState::Down => String::from("Down ⬇️"),
    _ => state.to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_alert_interpolates_headers_and_url() {
    let mut headers = HashMap::new();
    headers.insert(
      String::from("authorization"),
      String::from("Bearer [[TOKEN]]"),
    );
    headers
      .insert(String::from("x-komodo-env"), String::from("[[ENV]]"));

    let mut variables = HashMap::new();
    variables.insert(String::from("ENV"), String::from("dev"));

    let mut secrets = HashMap::new();
    secrets
      .insert(String::from("TOKEN"), String::from("super-secret"));

    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);
    let (url, headers) = interpolate_custom_request_parts(
      "https://example.com/[[ENV]]",
      &headers,
      &mut interpolator,
    )
    .unwrap();

    assert_eq!(url, "https://example.com/dev");
    assert_eq!(
      headers,
      vec![
        (
          String::from("authorization"),
          String::from("Bearer super-secret"),
        ),
        (String::from("x-komodo-env"), String::from("dev")),
      ]
    );
  }

  #[test]
  fn custom_alert_sanitizes_secret_on_request_error() {
    let mut headers = HashMap::new();
    headers.insert(
      String::from("authorization"),
      String::from("Bearer [[TOKEN]]"),
    );

    let variables = HashMap::new();
    let secrets = HashMap::from([(
      String::from("TOKEN"),
      String::from("super-secret"),
    )]);
    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolate_custom_request_parts(
      "https://example.com",
      &headers,
      &mut interpolator,
    )
    .unwrap();

    let err = sanitize_request_error(
      &interpolator,
      &"authorization: Bearer super-secret",
    )
    .to_string();

    assert!(err.contains("Error with request:"));
    assert!(!err.contains("super-secret"));
  }

  #[test]
  fn custom_alert_sanitizes_secret_in_response_body() {
    let variables = HashMap::new();
    let secrets = HashMap::from([(
      String::from("TOKEN"),
      String::from("super-secret"),
    )]);
    let interpolator =
      Interpolator::new(Some(&variables), &secrets);
    let mut interpolator = interpolator;
    let mut token = String::from("[[TOKEN]]");

    interpolator.interpolate_string(&mut token).unwrap();

    let sanitized = sanitize_interpolated_text(
      &interpolator,
      "authorization: Bearer super-secret",
    );

    assert!(!sanitized.contains("super-secret"));
  }
}

fn fmt_level(level: SeverityLevel) -> &'static str {
  match level {
    SeverityLevel::Critical => "CRITICAL 🚨",
    SeverityLevel::Warning => "WARNING ‼️",
    SeverityLevel::Ok => "OK ✅",
  }
}

fn resource_link(
  resource_type: ResourceTargetVariant,
  id: &str,
) -> String {
  komodo_client::entities::resource_link(
    &core_config().host,
    resource_type,
    id,
  )
}

/// Standard message content format
/// used by Ntfy, Pushover.
fn standard_alert_content(alert: &Alert) -> String {
  let level = fmt_level(alert.level);
  match &alert.data {
    AlertData::Test { id, name } => {
      let link = resource_link(ResourceTargetVariant::Alerter, id);
      format!(
        "{level} | If you see this message, then Alerter {name} is working\n{link}",
      )
    }
    AlertData::SwarmUnhealthy { id, name, err } => {
      let link = resource_link(ResourceTargetVariant::Swarm, id);
      match alert.level {
        SeverityLevel::Ok => {
          format!("{level} | Swarm {name} is now healthy\n{link}")
        }
        SeverityLevel::Critical => {
          let err = err
            .as_ref()
            .map(|e| format!("\nerror: {e:#?}"))
            .unwrap_or_default();
          format!(
            "{level} | Swarm {name} is unhealthy ❌\n{link}{err}"
          )
        }
        _ => unreachable!(),
      }
    }
    AlertData::ServerUnreachable {
      id,
      name,
      region,
      err,
    } => {
      let region = fmt_region(region);
      let link = resource_link(ResourceTargetVariant::Server, id);
      match alert.level {
        SeverityLevel::Ok => {
          format!("{level} | {name}{region} is now connected\n{link}")
        }
        SeverityLevel::Critical => {
          let err = err
            .as_ref()
            .map(|e| format!("\nerror: {e:#?}"))
            .unwrap_or_default();
          format!(
            "{level} | {name}{region} is unreachable ❌\n{link}{err}"
          )
        }
        _ => unreachable!(),
      }
    }
    AlertData::ServerVersionMismatch {
      id,
      name,
      region,
      server_version,
      core_version,
    } => {
      let region = fmt_region(region);
      let link = resource_link(ResourceTargetVariant::Server, id);
      match alert.level {
        SeverityLevel::Ok => {
          format!(
            "{level} | {name}{region} | Periphery version now matches Core version ✅\n{link}"
          )
        }
        _ => {
          format!(
            "{level} | {name}{region} | Version mismatch detected ⚠️\nPeriphery: {server_version} | Core: {core_version}\n{link}"
          )
        }
      }
    }
    AlertData::ServerCpu {
      id,
      name,
      region,
      percentage,
    } => {
      let region = fmt_region(region);
      let link = resource_link(ResourceTargetVariant::Server, id);
      format!(
        "{level} | {name}{region} cpu usage at {percentage:.1}%\n{link}",
      )
    }
    AlertData::ServerMem {
      id,
      name,
      region,
      used_gb,
      total_gb,
    } => {
      let region = fmt_region(region);
      let link = resource_link(ResourceTargetVariant::Server, id);
      let percentage = 100.0 * used_gb / total_gb;
      format!(
        "{level} | {name}{region} memory usage at {percentage:.1}%💾\n\nUsing {used_gb:.1} GiB / {total_gb:.1} GiB\n{link}",
      )
    }
    AlertData::ServerDisk {
      id,
      name,
      region,
      path,
      used_gb,
      total_gb,
    } => {
      let region = fmt_region(region);
      let link = resource_link(ResourceTargetVariant::Server, id);
      let percentage = 100.0 * used_gb / total_gb;
      format!(
        "{level} | {name}{region} disk usage at {percentage:.1}%💿\nmount point: {path:?}\nusing {used_gb:.1} GiB / {total_gb:.1} GiB\n{link}",
      )
    }
    AlertData::ContainerStateChange {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      from,
      to,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      let to_state = fmt_docker_container_state(to);
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "📦Deployment {name} is now {to_state}{target}\nprevious: {from}\n{link}",
      )
    }
    AlertData::DeploymentImageUpdateAvailable {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "⬆ Deployment {name} has an update available{target}\nimage: {image}\n{link}",
      )
    }
    AlertData::DeploymentAutoUpdated {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "⬆ Deployment {name} was updated automatically{target}\nimage: {image}\n{link}",
      )
    }
    AlertData::StackStateChange {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      from,
      to,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      let to_state = fmt_stack_state(to);
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "🥞 Stack {name} is now {to_state}{target}\nprevious: {from}\n{link}",
      )
    }
    AlertData::StackImageUpdateAvailable {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      service,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "⬆ Stack {name} has an update available{target}\nservice: {service}\nimage: {image}\n{link}",
      )
    }
    AlertData::StackAutoUpdated {
      id,
      name,
      swarm_id: _swarm_id,
      swarm_name,
      server_id: _server_id,
      server_name,
      images,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      let images_label =
        if images.len() > 1 { "images" } else { "image" };
      let images_str = images.join(", ");
      let target = if let Some(swarm) = swarm_name {
        format!("\nswarm: {swarm}")
      } else if let Some(server) = server_name {
        format!("\nserver: {server}")
      } else {
        String::new()
      };
      format!(
        "⬆ Stack {name} was updated automatically ⏫{target}\n{images_label}: {images_str}\n{link}",
      )
    }
    AlertData::AwsBuilderTerminationFailed {
      instance_id,
      message,
    } => {
      format!(
        "{level} | Failed to terminate AWS builder instance\ninstance id: {instance_id}\n{message}",
      )
    }
    AlertData::ResourceSyncPendingUpdates { id, name } => {
      let link =
        resource_link(ResourceTargetVariant::ResourceSync, id);
      format!(
        "{level} | Pending resource sync updates on {name}\n{link}",
      )
    }
    AlertData::BuildFailed { id, name, version } => {
      let link = resource_link(ResourceTargetVariant::Build, id);
      format!(
        "{level} | Build {name} failed\nversion: v{version}\n{link}",
      )
    }
    AlertData::RepoBuildFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Repo, id);
      format!("{level} | Repo build for {name} failed\n{link}",)
    }
    AlertData::ProcedureFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Procedure, id);
      format!("{level} | Procedure {name} failed\n{link}")
    }
    AlertData::ActionFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Action, id);
      format!("{level} | Action {name} failed\n{link}")
    }
    AlertData::ScheduleRun {
      resource_type,
      id,
      name,
    } => {
      let link = resource_link(*resource_type, id);
      format!(
        "{level} | {name} ({resource_type}) | Scheduled run started 🕝\n{link}"
      )
    }
    AlertData::Custom { message, details } => {
      format!(
        "{level} | {message}{}",
        if details.is_empty() {
          String::new()
        } else {
          format!("\n{details}")
        }
      )
    }
    AlertData::None {} => Default::default(),
  }
}
