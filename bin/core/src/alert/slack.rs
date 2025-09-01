use super::*;

#[instrument(level = "debug")]
pub async fn send_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let level = fmt_level(alert.level);
  let (text, blocks): (_, Option<_>) = match &alert.data {
    AlertData::Test { id, name } => {
      let text = format!(
        "{level} | If you see this message, then Alerter *{name}* is *working*"
      );
      let blocks = vec![
        Block::header(level),
        Block::section(format!(
          "If you see this message, then Alerter *{name}* is *working*"
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Alerter,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ServerVersionMismatch {
      id,
      name,
      region,
      server_version,
      core_version,
    } => {
      let region = fmt_region(region);
      let text = match alert.level {
        SeverityLevel::Ok => {
          format!(
            "{level} | *{name}*{region} | Periphery version now matches Core version ✅"
          )
        }
        _ => {
          format!(
            "{level} | *{name}*{region} | Version mismatch detected ⚠️\nPeriphery: {server_version} | Core: {core_version}"
          )
        }
      };
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(resource_link(
          ResourceTargetVariant::Server,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ServerUnreachable {
      id,
      name,
      region,
      err,
    } => {
      let region = fmt_region(region);
      match alert.level {
        SeverityLevel::Ok => {
          let text =
            format!("{level} | *{name}*{region} is now *reachable*");
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} is now *reachable*"
            )),
          ];
          (text, blocks.into())
        }
        SeverityLevel::Critical => {
          let text =
            format!("{level} | *{name}*{region} is *unreachable* ❌");
          let err = err
            .as_ref()
            .map(|e| format!("\nerror: {e:#?}"))
            .unwrap_or_default();
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} is *unreachable* ❌{err}"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
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
      match alert.level {
        SeverityLevel::Ok => {
          let text = format!(
            "{level} | *{name}*{region} cpu usage at *{percentage:.1}%*"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} cpu usage at *{percentage:.1}%*"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
        _ => {
          let text = format!(
            "{level} | *{name}*{region} cpu usage at *{percentage:.1}%* 📈"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} cpu usage at *{percentage:.1}%* 📈"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
      }
    }
    AlertData::ServerMem {
      id,
      name,
      region,
      used_gb,
      total_gb,
    } => {
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      match alert.level {
        SeverityLevel::Ok => {
          let text = format!(
            "{level} | *{name}*{region} memory usage at *{percentage:.1}%* 💾"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} memory usage at *{percentage:.1}%* 💾"
            )),
            Block::section(format!(
              "using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
        _ => {
          let text = format!(
            "{level} | *{name}*{region} memory usage at *{percentage:.1}%* 💾"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} memory usage at *{percentage:.1}%* 💾"
            )),
            Block::section(format!(
              "using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
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
      let region = fmt_region(region);
      let percentage = 100.0 * used_gb / total_gb;
      match alert.level {
        SeverityLevel::Ok => {
          let text = format!(
            "{level} | *{name}*{region} disk usage at *{percentage:.1}%* | mount point: *{path:?}* 💿"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} disk usage at *{percentage:.1}%* 💿"
            )),
            Block::section(format!(
              "mount point: {path:?} | using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
        _ => {
          let text = format!(
            "{level} | *{name}*{region} disk usage at *{percentage:.1}%* | mount point: *{path:?}* 💿"
          );
          let blocks = vec![
            Block::header(level),
            Block::section(format!(
              "*{name}*{region} disk usage at *{percentage:.1}%* 💿"
            )),
            Block::section(format!(
              "mount point: {path:?} | using *{used_gb:.1} GiB* / *{total_gb:.1} GiB*"
            )),
            Block::section(resource_link(
              ResourceTargetVariant::Server,
              id,
            )),
          ];
          (text, blocks.into())
        }
      }
    }
    AlertData::ContainerStateChange {
      name,
      server_name,
      from,
      to,
      id,
      ..
    } => {
      let to = fmt_docker_container_state(to);
      let text = format!("📦 Container *{name}* is now *{to}*");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: {server_name}\nprevious: {from}",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Deployment,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::DeploymentImageUpdateAvailable {
      id,
      name,
      server_name,
      server_id: _server_id,
      image,
    } => {
      let text =
        format!("⬆ Deployment *{name}* has an update available");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: *{server_name}*\nimage: *{image}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Deployment,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::DeploymentAutoUpdated {
      id,
      name,
      server_name,
      server_id: _server_id,
      image,
    } => {
      let text =
        format!("⬆ Deployment *{name}* was updated automatically ⏫");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: *{server_name}*\nimage: *{image}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Deployment,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::StackStateChange {
      name,
      server_name,
      from,
      to,
      id,
      ..
    } => {
      let to = fmt_stack_state(to);
      let text = format!("🥞 Stack *{name}* is now *{to}*");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: *{server_name}*\nprevious: *{from}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Stack,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::StackImageUpdateAvailable {
      id,
      name,
      server_name,
      server_id: _server_id,
      service,
      image,
    } => {
      let text = format!("⬆ Stack *{name}* has an update available");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: *{server_name}*\nservice: *{service}*\nimage: *{image}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Stack,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::StackAutoUpdated {
      id,
      name,
      server_name,
      server_id: _server_id,
      images,
    } => {
      let text =
        format!("⬆ Stack *{name}* was updated automatically ⏫");
      let images_label =
        if images.len() > 1 { "images" } else { "image" };
      let images = images.join(", ");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "server: *{server_name}*\n{images_label}: *{images}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::Stack,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::AwsBuilderTerminationFailed {
      instance_id,
      message,
    } => {
      let text = format!(
        "{level} | Failed to terminated AWS builder instance "
      );
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "instance id: *{instance_id}*\n{message}"
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ResourceSyncPendingUpdates { id, name } => {
      let text = format!(
        "{level} | Pending resource sync updates on *{name}*"
      );
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!(
          "sync id: *{id}*\nsync name: *{name}*",
        )),
        Block::section(resource_link(
          ResourceTargetVariant::ResourceSync,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::BuildFailed { id, name, version } => {
      let text = format!("{level} | Build {name} has failed");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(format!("version: *v{version}*",)),
        Block::section(resource_link(
          ResourceTargetVariant::Build,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::RepoBuildFailed { id, name } => {
      let text =
        format!("{level} | Repo build for *{name}* has *failed*");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(resource_link(
          ResourceTargetVariant::Repo,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ProcedureFailed { id, name } => {
      let text = format!("{level} | Procedure *{name}* has *failed*");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(resource_link(
          ResourceTargetVariant::Procedure,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ActionFailed { id, name } => {
      let text = format!("{level} | Action *{name}* has *failed*");
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(resource_link(
          ResourceTargetVariant::Action,
          id,
        )),
      ];
      (text, blocks.into())
    }
    AlertData::ScheduleRun {
      resource_type,
      id,
      name,
    } => {
      let text = format!(
        "{level} | *{name}* ({resource_type}) | Scheduled run started 🕝"
      );
      let blocks = vec![
        Block::header(text.clone()),
        Block::section(resource_link(*resource_type, id)),
      ];
      (text, blocks.into())
    }
    AlertData::Custom { message, details } => {
      let text = format!("{level} | {message}");
      let blocks =
        vec![Block::header(text.clone()), Block::section(details)];
      (text, blocks.into())
    }
    AlertData::None {} => Default::default(),
  };
  if !text.is_empty() {
    let VariablesAndSecrets { variables, secrets } =
      get_variables_and_secrets().await?;
    let mut url_interpolated = url.to_string();

    let mut interpolator =
      Interpolator::new(Some(&variables), &secrets);

    interpolator.interpolate_string(&mut url_interpolated)?;

    let slack = ::slack::Client::new(url_interpolated);
    slack.send_message(text, blocks).await.map_err(|e| {
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
