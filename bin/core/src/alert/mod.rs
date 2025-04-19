use ::slack::types::Block;
use anyhow::{Context, anyhow};
use derive_variants::ExtractVariant;
use futures::future::join_all;
use komodo_client::entities::{
  ResourceTargetVariant,
  alert::{Alert, AlertData, AlertDataVariant, SeverityLevel},
  alerter::*,
  deployment::DeploymentState,
  stack::StackState,
};
use mungos::{find::find_collect, mongodb::bson::doc};
use std::collections::HashSet;
use tracing::Instrument;

use crate::helpers::interpolate::interpolate_variables_secrets_into_string;
use crate::helpers::query::get_variables_and_secrets;
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

    let handles =
      alerts.iter().map(|alert| send_alert(&alerters, alert));

    join_all(handles).await;
  }
  .instrument(span)
  .await
}

#[instrument(level = "debug")]
async fn send_alert(alerters: &[Alerter], alert: &Alert) {
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
    AlerterEndpoint::Ntfy(NtfyAlerterEndpoint { url }) => {
      ntfy::send_alert(url, alert).await.with_context(|| {
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
  let vars_and_secrets = get_variables_and_secrets().await?;
  let mut global_replacers = HashSet::new();
  let mut secret_replacers = HashSet::new();
  let mut url_interpolated = url.to_string();

  // interpolate variables and secrets into the url
  interpolate_variables_secrets_into_string(
    &vars_and_secrets,
    &mut url_interpolated,
    &mut global_replacers,
    &mut secret_replacers,
  )?;

  let res = reqwest::Client::new()
    .post(url_interpolated)
    .json(alert)
    .send()
    .await
    .map_err(|e| {
      let replacers =
        secret_replacers.into_iter().collect::<Vec<_>>();
      let sanitized_error =
        svi::replace_in_string(&format!("{e:?}"), &replacers);
      anyhow::Error::msg(format!(
        "Error with request: {}",
        sanitized_error
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
  let path = match resource_type {
    ResourceTargetVariant::System => unreachable!(),
    ResourceTargetVariant::Build => format!("/builds/{id}"),
    ResourceTargetVariant::Builder => {
      format!("/builders/{id}")
    }
    ResourceTargetVariant::Deployment => {
      format!("/deployments/{id}")
    }
    ResourceTargetVariant::Stack => {
      format!("/stacks/{id}")
    }
    ResourceTargetVariant::Server => {
      format!("/servers/{id}")
    }
    ResourceTargetVariant::Repo => format!("/repos/{id}"),
    ResourceTargetVariant::Alerter => {
      format!("/alerters/{id}")
    }
    ResourceTargetVariant::Procedure => {
      format!("/procedures/{id}")
    }
    ResourceTargetVariant::Action => {
      format!("/actions/{id}")
    }
    ResourceTargetVariant::ServerTemplate => {
      format!("/server-templates/{id}")
    }
    ResourceTargetVariant::ResourceSync => {
      format!("/resource-syncs/{id}")
    }
  };

  format!("{}{path}", core_config().host)
}
