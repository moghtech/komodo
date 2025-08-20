use std::sync::OnceLock;

use super::*;

#[instrument(level = "debug")]
pub async fn send_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let level = fmt_level(alert.level);
  let content = match &alert.data {
    AlertData::Test { id, name } => {
      let link = resource_link(ResourceTargetVariant::Alerter, id);
      format!(
        "{level} | If you see this message, then Alerter {name} is working\n{link}",
      )
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
    AlertData::None {} => Default::default(),
  };

  if !content.is_empty() {
    send_message(url, content).await?;
  }
  Ok(())
}

async fn send_message(
  url: &str,
  content: String,
) -> anyhow::Result<()> {
  // pushover needs all information to be encoded in the URL. At minimum they need
  // the user key, the application token, and the message (url encoded).
  // other optional params here: https://pushover.net/api (just add them to the
  // webhook url along with the application token and the user key).
  let content = [("message", content)];

  let response = http_client()
    .post(url)
    .form(&content)
    .send()
    .await
    .context("Failed to send message")?;

  let status = response.status();
  if status.is_success() {
    debug!("pushover alert sent successfully: {}", status);
    Ok(())
  } else {
    let text = response.text().await.with_context(|| {
      format!(
        "Failed to send message to pushover | {status} | failed to get response text"
      )
    })?;
    Err(anyhow!(
      "Failed to send message to pushover | {} | {}",
      status,
      text
    ))
  }
}

fn http_client() -> &'static reqwest::Client {
  static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
  CLIENT.get_or_init(reqwest::Client::new)
}
