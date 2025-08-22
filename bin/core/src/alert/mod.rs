use ::slack::types::Block;
use anyhow::{Context, anyhow};
use database::mungos::{find::find_collect, mongodb::bson::doc};
use derive_variants::ExtractVariant;
use futures::future::join_all;
use interpolate::Interpolator;
use komodo_client::entities::{
  ResourceTargetVariant,
  alert::{Alert, AlertData, AlertDataVariant, SeverityLevel},
  alerter::*,
  deployment::DeploymentState,
  komodo_timestamp,
  stack::StackState,
};
use tracing::Instrument;

use crate::helpers::query::get_variables_and_secrets;
use crate::helpers::{
  maintenance::is_in_maintenance, query::VariablesAndSecrets,
};
use crate::{config::core_config, state::db_client};

mod discord;
mod ntfy;
mod pushover;
mod slack;

#[instrument(level = "debug")]
pub async fn send_alerts(alerts: &[Alert]) {
  if alerts.is_empty() {
    return;
  }

  let span =
    info_span!("send_alerts", alerts = format!("{alerts:?}"));
  async {
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
  .instrument(span)
  .await
}

#[instrument(level = "debug")]
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

  let alert_type = alert.data.extract_variant();

  // In the test case, we don't want the filters inside this
  // block to stop the test from being sent to the alerting endpoint.
  if alert_type != AlertDataVariant::Test {
    // Don't send if alert type not configured on the alerter
    if !alerter.config.alert_types.is_empty()
      && !alerter.config.alert_types.contains(&alert_type)
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
    AlerterEndpoint::Custom(CustomAlerterEndpoint { url }) => {
      send_custom_alert(url, alert).await.with_context(|| {
        format!(
          "Failed to send alert to Custom Alerter {}",
          alerter.name
        )
      })
    }
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

#[instrument(level = "debug")]
async fn send_custom_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let VariablesAndSecrets { variables, secrets } =
    get_variables_and_secrets().await?;
  let mut url_interpolated = url.to_string();

  let mut interpolator =
    Interpolator::new(Some(&variables), &secrets);

  interpolator.interpolate_string(&mut url_interpolated)?;

  let res = reqwest::Client::new()
    .post(url_interpolated)
    .json(alert)
    .send()
    .await
    .map_err(|e| {
      let replacers = interpolator
        .secret_replacers
        .into_iter()
        .collect::<Vec<_>>();
      let sanitized_error =
        svi::replace_in_string(&format!("{e:?}"), &replacers);
      anyhow::Error::msg(format!(
        "Error with request: {sanitized_error}"
      ))
    })
    .context("failed at post request to alerter")?;
  let status = res.status();
  if !status.is_success() {
    let text = res
      .text()
      .await
      .context("failed to get response text on alerter response")?;
    return Err(anyhow!(
      "post to alerter failed | {status} | {text}"
    ));
  }
  Ok(())
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
            "{level} | {name} ({region}) | Server version now matches core version ✅\n{link}"
          )
        }
        _ => {
          format!(
            "{level} | {name} ({region}) | Version mismatch detected ⚠️\nServer: {server_version} | Core: {core_version}\n{link}"
          )
        }
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
          format!("{level} | {name}{region} is now reachable\n{link}")
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
      server_id: _server_id,
      server_name,
      from,
      to,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      let to_state = fmt_docker_container_state(to);
      format!(
        "📦Deployment {name} is now {to_state}\nserver: {server_name}\nprevious: {from}\n{link}",
      )
    }
    AlertData::DeploymentImageUpdateAvailable {
      id,
      name,
      server_id: _server_id,
      server_name,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      format!(
        "⬆ Deployment {name} has an update available\nserver: {server_name}\nimage: {image}\n{link}",
      )
    }
    AlertData::DeploymentAutoUpdated {
      id,
      name,
      server_id: _server_id,
      server_name,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Deployment, id);
      format!(
        "⬆ Deployment {name} was updated automatically\nserver: {server_name}\nimage: {image}\n{link}",
      )
    }
    AlertData::StackStateChange {
      id,
      name,
      server_id: _server_id,
      server_name,
      from,
      to,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      let to_state = fmt_stack_state(to);
      format!(
        "🥞 Stack {name} is now {to_state}\nserver: {server_name}\nprevious: {from}\n{link}",
      )
    }
    AlertData::StackImageUpdateAvailable {
      id,
      name,
      server_id: _server_id,
      server_name,
      service,
      image,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      format!(
        "⬆ Stack {name} has an update available\nserver: {server_name}\nservice: {service}\nimage: {image}\n{link}",
      )
    }
    AlertData::StackAutoUpdated {
      id,
      name,
      server_id: _server_id,
      server_name,
      images,
    } => {
      let link = resource_link(ResourceTargetVariant::Stack, id);
      let images_label =
        if images.len() > 1 { "images" } else { "image" };
      let images_str = images.join(", ");
      format!(
        "⬆ Stack {name} was updated automatically ⏫\nserver: {server_name}\n{images_label}: {images_str}\n{link}",
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
          format_args!("")
        } else {
          format_args!("\n{details}")
        }
      )
    }
    AlertData::None {} => Default::default(),
  }
}
