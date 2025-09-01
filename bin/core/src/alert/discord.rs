use std::sync::OnceLock;

use serde::Serialize;

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
        "{level} | If you see this message, then Alerter **{name}** is **working**\n{link}"
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
            "{level} | **{name}**{region} | Periphery version now matches Core version ✅\n{link}"
          )
        }
        _ => {
          format!(
            "{level} | **{name}**{region} | Version mismatch detected ⚠️\nPeriphery: **{server_version}** | Core: **{core_version}**\n{link}"
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
          format!(
            "{level} | **{name}**{region} is now **reachable**\n{link}"
          )
        }
        SeverityLevel::Critical => {
          let err = err
            .as_ref()
            .map(|e| format!("\n**error**: {e:#?}"))
            .unwrap_or_default();
          format!(
            "{level} | **{name}**{region} is **unreachable** ❌\n{link}{err}"
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
        "{level} | **{name}**{region} cpu usage at **{percentage:.1}%**\n{link}"
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
        "{level} | **{name}**{region} memory usage at **{percentage:.1}%** 💾\n\nUsing **{used_gb:.1} GiB** / **{total_gb:.1} GiB**\n{link}"
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
        "{level} | **{name}**{region} disk usage at **{percentage:.1}%** 💿\nmount point: `{path:?}`\nusing **{used_gb:.1} GiB** / **{total_gb:.1} GiB**\n{link}"
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
      let to = fmt_docker_container_state(to);
      format!(
        "📦 Deployment **{name}** is now **{to}**\nserver: **{server_name}**\nprevious: **{from}**\n{link}"
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
        "⬆ Deployment **{name}** has an update available\nserver: **{server_name}**\nimage: **{image}**\n{link}"
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
        "⬆ Deployment **{name}** was updated automatically ⏫\nserver: **{server_name}**\nimage: **{image}**\n{link}"
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
      let to = fmt_stack_state(to);
      format!(
        "🥞 Stack **{name}** is now {to}\nserver: **{server_name}**\nprevious: **{from}**\n{link}"
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
        "⬆ Stack **{name}** has an update available\nserver: **{server_name}**\nservice: **{service}**\nimage: **{image}**\n{link}"
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
      let images = images.join(", ");
      format!(
        "⬆ Stack **{name}** was updated automatically ⏫\nserver: **{server_name}**\n{images_label}: **{images}**\n{link}"
      )
    }
    AlertData::AwsBuilderTerminationFailed {
      instance_id,
      message,
    } => {
      format!(
        "{level} | Failed to terminated AWS builder instance\ninstance id: **{instance_id}**\n{message}"
      )
    }
    AlertData::ResourceSyncPendingUpdates { id, name } => {
      let link =
        resource_link(ResourceTargetVariant::ResourceSync, id);
      format!(
        "{level} | Pending resource sync updates on **{name}**\n{link}"
      )
    }
    AlertData::BuildFailed { id, name, version } => {
      let link = resource_link(ResourceTargetVariant::Build, id);
      format!(
        "{level} | Build **{name}** failed\nversion: **v{version}**\n{link}"
      )
    }
    AlertData::RepoBuildFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Repo, id);
      format!("{level} | Repo build for **{name}** failed\n{link}")
    }
    AlertData::ProcedureFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Procedure, id);
      format!("{level} | Procedure **{name}** failed\n{link}")
    }
    AlertData::ActionFailed { id, name } => {
      let link = resource_link(ResourceTargetVariant::Action, id);
      format!("{level} | Action **{name}** failed\n{link}")
    }
    AlertData::ScheduleRun {
      resource_type,
      id,
      name,
    } => {
      let link = resource_link(*resource_type, id);
      format!(
        "{level} | **{name}** ({resource_type}) | Scheduled run started 🕝\n{link}"
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
  };
  if !content.is_empty() {
    let VariablesAndSecrets { variables, secrets } =
      get_variables_and_secrets().await?;
    let mut url_interpolated = url.to_string();

    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolator.interpolate_string(&mut url_interpolated)?;

    send_message(&url_interpolated, &content)
      .await
      .map_err(|e| {
        let replacers = interpolator
          .secret_replacers
          .into_iter()
          .collect::<Vec<_>>();
        let sanitized_error =
          svi::replace_in_string(&format!("{e:?}"), &replacers);
        anyhow::Error::msg(format!(
          "Error with slack request: {sanitized_error}"
        ))
      })?;
  }
  Ok(())
}

async fn send_message(
  url: &str,
  content: &str,
) -> anyhow::Result<()> {
  let body = DiscordMessageBody { content };

  let response = http_client()
    .post(url)
    .json(&body)
    .send()
    .await
    .context("Failed to send message")?;

  let status = response.status();

  if status.is_success() {
    Ok(())
  } else {
    let text = response.text().await.with_context(|| {
      format!("Failed to send message to Discord | {status} | failed to get response text")
    })?;
    Err(anyhow::anyhow!(
      "Failed to send message to Discord | {status} | {text}"
    ))
  }
}

fn http_client() -> &'static reqwest::Client {
  static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
  CLIENT.get_or_init(reqwest::Client::new)
}

#[derive(Serialize)]
struct DiscordMessageBody<'a> {
  content: &'a str,
}
