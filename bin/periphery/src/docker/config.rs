use std::fmt::Write;

use anyhow::{Context, anyhow};
use command::{
  run_komodo_shell_command, run_komodo_standard_command,
};
use data_encoding::BASE64URL;
use futures_util::{TryStreamExt as _, stream::FuturesUnordered};
use komodo_client::entities::{
  all_logs_success,
  docker::{
    config::{SwarmConfig, SwarmConfigListItem},
    service::SwarmService,
  },
  random_string,
};
use periphery_client::api::swarm::CreateSwarmConfig;

use super::*;

pub async fn list_swarm_configs()
-> anyhow::Result<Vec<SwarmConfigListItem>> {
  let res = run_komodo_standard_command(
    "List Swarm Configs",
    None,
    "docker config ls --format json",
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(
      "Failed to list swarm configs using 'docker config ls'",
    ));
  }

  // The output is in JSONL, need to convert to standard JSON vec.
  let mut res = serde_json::from_str::<Vec<SwarmConfigListItem>>(
    &format!("[{}]", res.stdout.trim().replace('\n', ",")),
  )
  .context("Failed to parse 'docker config ls' response from json")?;

  res.sort_by(|a, b| {
    a.name
      .cmp(&b.name)
      .then_with(|| b.updated_at.cmp(&a.updated_at))
  });

  Ok(res)
}

pub async fn inspect_swarm_config(
  config: &str,
) -> anyhow::Result<SwarmConfig> {
  let res = run_komodo_standard_command(
    "Inspect Swarm Config",
    None,
    format!(r#"docker config inspect "{config}""#),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(format!(
      "Failed to inspect swarm config using 'docker config inspect {config}'"
    )));
  }

  let mut res = serde_json::from_str::<Vec<SwarmConfig>>(&res.stdout)
    .context(
      "Failed to parse 'docker config inspect' response from json",
    )?
    .pop()
    .with_context(|| {
      format!("Did not find any config matching {config}")
    })?;

  // Convert data back to readable / editable format
  res.spec.iter_mut().next().map(|spec| {
    spec.data.iter_mut().next().map(|data| {
      if let Ok(res) = BASE64URL
        .decode(data.as_bytes())
        .map_err(anyhow::Error::new)
        .and_then(|data| {
          String::from_utf8(data).map_err(anyhow::Error::new)
        })
      {
        *data = res;
      }
    })
  });

  Ok(res)
}

pub async fn create_swarm_config(
  CreateSwarmConfig {
    name,
    data,
    labels,
    template_driver,
  }: &CreateSwarmConfig,
) -> anyhow::Result<Log> {
  let mut command = String::from("docker config create");

  for label in labels {
    write!(&mut command, " --label {label}")?;
  }

  if let Some(driver) = template_driver {
    write!(&mut command, " --template-driver {driver}")?;
  }

  write!(
    &mut command,
    r#" {name} - <<'EOF'
{}
EOF"#,
    data.trim()
  )?;

  let log =
    run_komodo_shell_command("Create Config", None, command).await;

  Ok(log)
}

pub async fn remove_swarm_configs(
  configs: impl Iterator<Item = &str>,
) -> Log {
  let mut command = String::from("docker config rm");
  for config in configs {
    command += " ";
    command += config;
  }
  run_komodo_standard_command("Remove Swarm Configs", None, command)
    .await
}

pub async fn recreate_swarm_config(
  config: &CreateSwarmConfig,
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let remove =
    remove_swarm_configs([config.name.as_str()].into_iter()).await;
  let success = remove.success;
  logs.push(remove);
  if !success {
    return Ok(());
  }
  let log = create_swarm_config(config).await?;
  logs.push(log);
  Ok(())
}

struct ServiceConfigFile {
  /// Service name
  service: String,
  /// Config file spec
  file: TaskSpecContainerSpecFile,
}

impl DockerClient {
  pub async fn rotate_swarm_config(
    &self,
    config: &str,
    data: String,
    logs: &mut Vec<Log>,
  ) -> anyhow::Result<()> {
    let config = inspect_swarm_config(config).await?;
    let config_id = config.id.context("Failed to get config id")?;
    let spec = config.spec.context("Failed to get config spec")?;
    let name = spec.name.context("Failed to get config name")?;
    let labels = spec
      .labels
      .map(|labels| {
        labels
          .into_iter()
          .map(|(variable, value)| format!("{variable}={value}"))
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();
    let template_driver =
      spec.templating.map(|templating| templating.name);

    let create_config = CreateSwarmConfig {
      name,
      data,
      labels,
      template_driver,
    };

    let services = self
      .filter_map_swarm_services(|service| {
        extract_from_service(service, &config_id)
      })
      .await?;
    if services.is_empty() {
      return recreate_swarm_config(&create_config, logs).await;
    }

    // Create a tmp config for rotation
    let tmp_create_config = CreateSwarmConfig {
      name: format!(
        "{}-tmp-{}",
        create_config.name,
        random_string(10)
      ),
      data: create_config.data.clone(),
      labels: create_config.labels.clone(),
      template_driver: create_config.template_driver.clone(),
    };
    let log = create_swarm_config(&tmp_create_config).await?;
    logs.push(log);
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update services to tmp
    switch_services_config(
      &services,
      &create_config.name,
      &tmp_create_config.name,
      logs,
    )
    .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Recreate actual config
    recreate_swarm_config(&create_config, logs).await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update back to original
    switch_services_config(
      &services,
      &tmp_create_config.name,
      &create_config.name,
      logs,
    )
    .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Remove tmp config
    let log = remove_swarm_configs(
      [tmp_create_config.name.as_str()].into_iter(),
    )
    .await;
    logs.push(log);

    Ok(())
  }
}

async fn switch_services_config(
  services: &[ServiceConfigFile],
  from: &str,
  to: &str,
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let res = services
    .iter()
    .map(|service| async move {
      switch_service_config(&service.service, from, to, &service.file)
        .await
    })
    .collect::<FuturesUnordered<_>>()
    .try_collect::<Vec<_>>()
    .await?;
  logs.extend(res);
  Ok(())
}

async fn switch_service_config(
  service: &str,
  from: &str,
  to: &str,
  TaskSpecContainerSpecFile {
    name: path,
    uid,
    gid,
    mode,
  }: &TaskSpecContainerSpecFile,
) -> anyhow::Result<Log> {
  let mut command = format!(
    "docker service update --config-rm {from} --config-add source={to}"
  );

  // Add same file mount options
  if let Some(container_path) = path {
    write!(&mut command, ",target={container_path}")?;
  }
  if let Some(uid) = uid {
    write!(&mut command, ",uid={uid}")?;
  }
  if let Some(gid) = gid {
    write!(&mut command, ",gid={gid}")?;
  }
  if let Some(mode) = mode {
    write!(&mut command, ",mode={mode}")?;
  }

  write!(&mut command, " {service}")?;

  let log = run_komodo_standard_command(
    "Switch Service Config",
    None,
    command,
  )
  .await;

  Ok(log)
}

fn extract_from_service(
  service: SwarmService,
  config_id: &str,
) -> Option<ServiceConfigFile> {
  let spec = service.spec?;
  let configs = spec.task_template?.container_spec?.configs?;
  let config = configs.into_iter().find(|cfg| {
    cfg
      .config_id
      .as_ref()
      .map(|id| id == config_id)
      .unwrap_or_default()
  })?;
  Some(ServiceConfigFile {
    service: spec.name?,
    file: config.file?,
  })
}
