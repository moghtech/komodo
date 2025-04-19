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
        "{level} | If you see this message, then Alerter {} is working\n{link}",
        name,
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
          format!(
            "{level} | {}{} is now reachable\n{link}",
            name, region
          )
        }
        SeverityLevel::Critical => {
          let err = err
            .as_ref()
            .map(|e| format!("\nerror: {:#?}", e))
            .unwrap_or_default();
          format!(
            "{level} | {}{} is unreachable ❌\n{link}{err}",
            name, region
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
        "{level} | {}{} cpu usage at {percentage:.1}%\n{link}",
        name, region,
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
        "{level} | {}{} memory usage at {percentage:.1}%💾\n\nUsing {used_gb:.1} GiB / {total_gb:.1} GiB\n{link}",
        name, region,
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
        "{level} | {}{} disk usage at {percentage:.1}%💿\nmount point: {:?}\nusing {used_gb:.1} GiB / {total_gb:.1} GiB\n{link}",
        name, region, path,
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
        "📦Deployment {} is now {}\nserver: {}\nprevious: {}\n{link}",
        name, to_state, server_name, from,
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
        "⬆ Deployment {} has an update available\nserver: {}\nimage: {}\n{link}",
        name, server_name, image,
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
        "⬆ Deployment {} was updated automatically\nserver: {}\nimage: {}\n{link}",
        name, server_name, image,
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
        "🥞 Stack {} is now {}\nserver: {}\nprevious: {}\n{link}",
        name, to_state, server_name, from,
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
        "⬆ Stack {} has an update available\nserver: {}\nservice: {}\nimage: {}\n{link}",
        name, server_name, service, image,
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
        "⬆ Stack {} was updated automatically ⏫\nserver: {}\n{}: {}\n{link}",
        name, server_name, images_label, images_str,
      )
    }
    AlertData::AwsBuilderTerminationFailed {
      instance_id,
      message,
    } => {
      format!(
        "{level} | Failed to terminate AWS builder instance\ninstance id: {}\n{}",
        instance_id, message,
      )
    }
    AlertData::ResourceSyncPendingUpdates { id, name } => {
      let link =
        resource_link(ResourceTargetVariant::ResourceSync, id);
      format!(
        "{level} | Pending resource sync updates on {}\n{link}",
        name,
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
      format!("{level} | Repo build for {} failed\n{link}", name,)
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
        "Failed to send message to pushover | {} | failed to get response text",
        status
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
