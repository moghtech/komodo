use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{Map, Value, json};

use super::*;

/// Pinglet renders at most 3 badges, with values up to 32
/// characters. Over-length values are rejected by the API
/// (not truncated), so trim them here.
const MAX_BADGE_VALUE_LEN: usize = 32;
/// Pinglet `data` values are capped at 256 characters,
/// over-length values are rejected by the API.
const MAX_DATA_VALUE_LEN: usize = 256;

/// The body of a Pinglet publish request.
#[derive(Serialize)]
struct PingletMessage {
  #[serde(skip_serializing_if = "Option::is_none")]
  title: Option<String>,
  message: String,
  /// `silent` | `normal` | `urgent`
  priority: &'static str,
  /// `info` | `success` | `warning` | `error`
  level: &'static str,
  /// Key / value pills shown on the notification card.
  #[serde(skip_serializing_if = "Option::is_none")]
  badges: Option<Value>,
  /// Key / value metadata shown in the notification detail sheet.
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<Value>,
}

pub async fn send_alert(
  url: &str,
  token: Option<&str>,
  alert: &Alert,
) -> anyhow::Result<()> {
  let Some(message) = build_message(alert) else {
    return Ok(());
  };

  let VariablesAndSecrets { variables, secrets } =
    get_variables_and_secrets().await?;

  let mut url_interpolated = url.to_string();
  let mut token_interpolated = token.unwrap_or_default().to_string();

  let mut interpolator =
    Interpolator::new(Some(&variables), &secrets);

  interpolator.interpolate_string(&mut url_interpolated)?;
  interpolator.interpolate_string(&mut token_interpolated)?;

  send_message(&url_interpolated, &token_interpolated, &message)
    .await
    .map_err(|e| {
      let replacers = interpolator
        .secret_replacers
        .into_iter()
        .collect::<Vec<_>>();
      let sanitized_error =
        svi::replace_in_string(&format!("{e:?}"), &replacers);
      anyhow::Error::msg(format!(
        "Error with request to Pinglet: {sanitized_error}"
      ))
    })
}

async fn send_message(
  url: &str,
  token: &str,
  message: &PingletMessage,
) -> anyhow::Result<()> {
  let mut request = http_client().post(url).json(message);

  if !token.is_empty() {
    request = request.bearer_auth(token);
  }

  let response =
    request.send().await.context("Failed to send message")?;

  let status = response.status();
  if status.is_success() {
    debug!("pinglet alert sent successfully: {}", status);
    Ok(())
  } else {
    let text = response.text().await.with_context(|| {
      format!(
        "Failed to send message to Pinglet | {status} | failed to get response text"
      )
    })?;
    Err(anyhow!(
      "Failed to send message to Pinglet | {status} | {text}",
    ))
  }
}

fn build_message(alert: &Alert) -> Option<PingletMessage> {
  let rendered = render(alert)?;

  let title = if alert.resolved {
    rendered.title.map(|title| format!("Resolved: {title}"))
  } else {
    rendered.title
  };

  Some(PingletMessage {
    title,
    message: rendered.message,
    priority: priority(alert),
    level: level(alert),
    badges: rendered
      .badge
      .map(|(key, value)| json!({ key: truncate(&value, MAX_BADGE_VALUE_LEN) })),
    data: data_bag(alert, rendered.data),
  })
}

/// A resolved alert is good news regardless of its original
/// severity, so it always shows as `success`.
fn level(alert: &Alert) -> &'static str {
  if alert.resolved {
    return "success";
  }
  match alert.level {
    SeverityLevel::Critical => "error",
    SeverityLevel::Warning => "warning",
    SeverityLevel::Ok => "info",
  }
}

/// Only an unresolved CRITICAL is intrusive enough to break
/// through Do Not Disturb, everything else is a standard
/// notification.
fn priority(alert: &Alert) -> &'static str {
  if !alert.resolved && alert.level == SeverityLevel::Critical {
    "urgent"
  } else {
    "normal"
  }
}

/// The per-variant parts of the notification, before the
/// alert-level fields (resolution prefix, level, priority)
/// are applied.
struct Rendered {
  title: Option<String>,
  message: String,
  /// At most one metric pill per alert, eg `("CPU", "95%")`.
  badge: Option<(&'static str, String)>,
  /// Variant specific detail-sheet entries.
  data: Map<String, Value>,
}

fn render(alert: &Alert) -> Option<Rendered> {
  let mut data = Map::new();
  let rendered = match &alert.data {
    AlertData::None {} => return None,
    AlertData::Test { id, name } => {
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Alerter, id),
      );
      Rendered {
        title: Some(String::from("Test alert")),
        message: format!(
          "If you see this message, then Alerter {name} is working"
        ),
        badge: None,
        data,
      }
    }
    AlertData::SwarmUnhealthy { id, name, err } => {
      insert(&mut data, "name", name);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Swarm, id),
      );
      if let Some(err) = err {
        insert(&mut data, "error", format!("{err:#?}"));
      }
      Rendered {
        title: Some(String::from("Swarm unhealthy")),
        message: if alert.resolved {
          format!("Swarm {name} is now healthy")
        } else {
          format!("Swarm {name} is unhealthy")
        },
        badge: None,
        data,
      }
    }
    AlertData::ServerUnreachable {
      id,
      name,
      region,
      err,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "region", region);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Server, id),
      );
      if let Some(err) = err {
        insert(&mut data, "error", format!("{err:#?}"));
      }
      let region = fmt_region(region);
      Rendered {
        title: Some(String::from("Server unreachable")),
        message: if alert.resolved {
          format!("Server {name}{region} is now connected")
        } else {
          format!("Server {name}{region} is unreachable")
        },
        badge: None,
        data,
      }
    }
    AlertData::ServerCpu {
      id,
      name,
      region,
      percentage,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "region", region);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Server, id),
      );
      let region = fmt_region(region);
      Rendered {
        title: Some(String::from("High CPU usage")),
        message: format!(
          "Server {name}{region} CPU usage at {percentage:.1}%"
        ),
        badge: Some(("CPU", format!("{percentage:.0}%"))),
        data,
      }
    }
    AlertData::ServerMem {
      id,
      name,
      region,
      used_gb,
      total_gb,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "region", region);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Server, id),
      );
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      Rendered {
        title: Some(String::from("High memory usage")),
        message: format!(
          "Server {name}{region} memory usage at {percentage:.1}%, using {used_gb:.1} GiB / {total_gb:.1} GiB"
        ),
        badge: Some(("Memory", format!("{percentage:.0}%"))),
        data,
      }
    }
    AlertData::ServerDisk {
      id,
      name,
      region,
      path,
      used_gb,
      total_gb,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "region", region);
      insert(&mut data, "path", format!("{path:?}"));
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Server, id),
      );
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      Rendered {
        title: Some(String::from("High disk usage")),
        message: format!(
          "Server {name}{region} disk usage at {percentage:.1}% on {path:?}, using {used_gb:.1} GiB / {total_gb:.1} GiB"
        ),
        badge: Some(("Disk", format!("{percentage:.0}%"))),
        data,
      }
    }
    AlertData::ServerVersionMismatch {
      id,
      name,
      region,
      server_version,
      core_version,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "region", region);
      insert(&mut data, "server_version", server_version);
      insert(&mut data, "core_version", core_version);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Server, id),
      );
      let region = fmt_region(region);
      Rendered {
        title: Some(String::from("Server version mismatch")),
        message: if alert.resolved {
          format!(
            "Server {name}{region} Periphery version now matches Core version"
          )
        } else {
          format!(
            "Server {name}{region} is running Periphery {server_version}, Core is {core_version}"
          )
        },
        badge: None,
        data,
      }
    }
    AlertData::ContainerStateChange {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      from,
      to,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "from", from.to_string());
      insert(&mut data, "to", to.to_string());
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Deployment, id),
      );
      Rendered {
        title: Some(String::from("Deployment state change")),
        message: format!(
          "Deployment {name} is now {to}{}, previously {from}",
          host_suffix(server_name, swarm_name)
        ),
        badge: Some(("State", to.to_string())),
        data,
      }
    }
    AlertData::DeploymentImageUpdateAvailable {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      image,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "image", image);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Deployment, id),
      );
      Rendered {
        title: Some(String::from("Deployment update available")),
        message: format!(
          "Deployment {name}{} has an image update available: {image}",
          host_suffix(server_name, swarm_name)
        ),
        badge: None,
        data,
      }
    }
    AlertData::DeploymentAutoUpdated {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      image,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "image", image);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Deployment, id),
      );
      Rendered {
        title: Some(String::from("Deployment auto-updated")),
        message: format!(
          "Deployment {name}{} was updated automatically to {image}",
          host_suffix(server_name, swarm_name)
        ),
        badge: None,
        data,
      }
    }
    AlertData::StackStateChange {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      from,
      to,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "from", from.to_string());
      insert(&mut data, "to", to.to_string());
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Stack, id),
      );
      Rendered {
        title: Some(String::from("Stack state change")),
        message: format!(
          "Stack {name} is now {to}{}, previously {from}",
          host_suffix(server_name, swarm_name)
        ),
        badge: Some(("State", to.to_string())),
        data,
      }
    }
    AlertData::StackImageUpdateAvailable {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      service,
      image,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "service", service);
      insert(&mut data, "image", image);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Stack, id),
      );
      Rendered {
        title: Some(String::from("Stack update available")),
        message: format!(
          "Stack {name} service {service}{} has an image update available: {image}",
          host_suffix(server_name, swarm_name)
        ),
        badge: None,
        data,
      }
    }
    AlertData::StackAutoUpdated {
      id,
      name,
      server_id: _,
      server_name,
      swarm_id: _,
      swarm_name,
      images,
    } => {
      insert(&mut data, "name", name);
      insert_opt(&mut data, "server", server_name);
      insert_opt(&mut data, "swarm", swarm_name);
      insert(&mut data, "images", images.join(", "));
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Stack, id),
      );
      let images_label =
        if images.len() > 1 { "images" } else { "image" };
      Rendered {
        title: Some(String::from("Stack auto-updated")),
        message: format!(
          "Stack {name}{} was updated automatically, {images_label}: {}",
          host_suffix(server_name, swarm_name),
          images.join(", ")
        ),
        badge: None,
        data,
      }
    }
    AlertData::AwsBuilderTerminationFailed {
      instance_id,
      message,
    } => {
      insert(&mut data, "instance_id", instance_id);
      Rendered {
        title: Some(String::from("AWS builder termination failed")),
        message: format!(
          "Failed to terminate AWS builder instance {instance_id}: {message}"
        ),
        badge: None,
        data,
      }
    }
    AlertData::ResourceSyncPendingUpdates { id, name } => {
      insert(&mut data, "name", name);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::ResourceSync, id),
      );
      Rendered {
        title: Some(String::from("Pending resource sync updates")),
        message: format!("Resource sync {name} has pending updates"),
        badge: None,
        data,
      }
    }
    AlertData::BuildFailed { id, name, version } => {
      insert(&mut data, "name", name);
      insert(&mut data, "version", format!("v{version}"));
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Build, id),
      );
      Rendered {
        title: Some(String::from("Build failed")),
        message: format!("Build {name} failed at v{version}"),
        badge: None,
        data,
      }
    }
    AlertData::RepoBuildFailed { id, name } => {
      insert(&mut data, "name", name);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Repo, id),
      );
      Rendered {
        title: Some(String::from("Repo build failed")),
        message: format!("Repo build for {name} failed"),
        badge: None,
        data,
      }
    }
    AlertData::ProcedureFailed { id, name } => {
      insert(&mut data, "name", name);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Procedure, id),
      );
      Rendered {
        title: Some(String::from("Procedure failed")),
        message: format!("Procedure {name} failed"),
        badge: None,
        data,
      }
    }
    AlertData::ActionFailed { id, name } => {
      insert(&mut data, "name", name);
      insert(
        &mut data,
        "link",
        resource_link(ResourceTargetVariant::Action, id),
      );
      Rendered {
        title: Some(String::from("Action failed")),
        message: format!("Action {name} failed"),
        badge: None,
        data,
      }
    }
    AlertData::ScheduleRun {
      resource_type,
      id,
      name,
    } => {
      insert(&mut data, "name", name);
      insert(&mut data, "link", resource_link(*resource_type, id));
      Rendered {
        title: Some(String::from("Scheduled run")),
        message: format!(
          "{name} ({resource_type}) scheduled run started"
        ),
        badge: None,
        data,
      }
    }
    // Custom alerts already carry a headline / body, so pass
    // them through instead of a generic title. Pinglet rejects
    // an empty message, so an all-empty custom alert falls back
    // to a placeholder body.
    AlertData::Custom { message, details } => {
      let (title, message) = match (
        message.trim().is_empty(),
        details.trim().is_empty(),
      ) {
        (false, false) => (Some(message.clone()), details.clone()),
        (false, true) => (None, message.clone()),
        (true, false) => (None, details.clone()),
        (true, true) => (None, String::from("Custom alert")),
      };
      Rendered {
        title,
        message,
        badge: None,
        data,
      }
    }
  };
  Some(rendered)
}

/// Assemble the full detail-sheet metadata: a `source` marker
/// and the alert target type, then the variant specific
/// entries collected in [`render`].
fn data_bag(
  alert: &Alert,
  mut data: Map<String, Value>,
) -> Option<Value> {
  data.insert(String::from("source"), json!("komodo"));
  data.insert(
    String::from("resource_type"),
    json!(alert.target.extract_variant().to_string()),
  );
  Some(Value::Object(data))
}

/// ` on server {server}` / ` on swarm {swarm}`, or empty when
/// the alert isn't attached to either — for inlining into a
/// sentence.
fn host_suffix(
  server_name: &Option<String>,
  swarm_name: &Option<String>,
) -> String {
  if let Some(swarm) = swarm_name {
    format!(" on swarm {swarm}")
  } else if let Some(server) = server_name {
    format!(" on server {server}")
  } else {
    String::new()
  }
}

fn insert(
  data: &mut Map<String, Value>,
  key: &str,
  value: impl AsRef<str>,
) {
  data.insert(
    key.to_string(),
    json!(truncate(value.as_ref(), MAX_DATA_VALUE_LEN)),
  );
}

fn insert_opt(
  data: &mut Map<String, Value>,
  key: &str,
  value: &Option<String>,
) {
  if let Some(value) = value {
    insert(data, key, value);
  }
}

/// Pinglet rejects (rather than truncates) over-length badge /
/// data values, so trim them before sending.
fn truncate(s: &str, max_chars: usize) -> String {
  if s.chars().count() <= max_chars {
    s.to_string()
  } else {
    s.chars().take(max_chars).collect()
  }
}

fn http_client() -> &'static reqwest::Client {
  static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
  CLIENT.get_or_init(reqwest::Client::new)
}
