use std::fmt::Write;

use anyhow::{Context, anyhow};
use command::{
  run_komodo_shell_command, run_komodo_standard_command,
};
use futures_util::{TryStreamExt as _, stream::FuturesUnordered};
use komodo_client::entities::{
  EnvironmentVar, all_logs_success,
  docker::{
    config::{SwarmConfig, SwarmConfigListItem},
    service::SwarmService,
  },
  random_string,
};

use crate::helpers::push_labels;

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
) -> anyhow::Result<Vec<SwarmConfig>> {
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

  serde_json::from_str(&res.stdout).context(
    "Failed to parse 'docker config inspect' response from json",
  )
}

pub async fn create_swarm_config(
  name: &str,
  data: &str,
  labels: &[EnvironmentVar],
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let mut command = String::from("docker config create");

  push_labels(&mut command, labels)?;

  write!(
    &mut command,
    r#" {name} - <<'EOF'
{data}
EOF
"#
  )?;

  let log =
    run_komodo_shell_command("Create Config", None, command).await;

  logs.push(log);

  Ok(())
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
  name: &str,
  data: &str,
  labels: &[EnvironmentVar],
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let remove = remove_swarm_configs([name].into_iter()).await;
  let success = remove.success;
  logs.push(remove);
  if !success {
    return Ok(());
  }
  create_swarm_config(name, data, labels, logs).await
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
    data: &str,
    logs: &mut Vec<Log>,
  ) -> anyhow::Result<()> {
    let labels = inspect_swarm_config(config)
      .await?
      .pop()
      .context("Did not find any matching config")?
      .spec
      .and_then(|spec| spec.labels)
      .map(|labels| {
        labels
          .into_iter()
          .map(|(variable, value)| EnvironmentVar { variable, value })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let services = self
      .filter_map_swarm_services(|service| {
        extract_from_service(service, config)
      })
      .await?;
    if services.is_empty() {
      return recreate_swarm_config(config, data, &labels, logs)
        .await;
    }

    // Create a tmp config
    let tmp_name = format!("{config}-tmp-{}", random_string(10));
    create_swarm_config(&tmp_name, data, &labels, logs).await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update services to tmp
    switch_services_config(&services, config, &tmp_name, logs)
      .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Recreate actual config
    recreate_swarm_config(config, data, &labels, logs).await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update back to original
    switch_services_config(&services, &tmp_name, config, logs)
      .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Remove tmp config
    let log =
      remove_swarm_configs([tmp_name.as_str()].into_iter()).await;
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
  config: &str,
) -> Option<ServiceConfigFile> {
  let spec = service.spec?;
  let configs = spec.task_template?.container_spec?.configs?;
  let config = configs.into_iter().find(|cfg| {
    cfg
      .config_id
      .as_ref()
      // Supports passing short id
      .map(|id| id.starts_with(config))
      .unwrap_or_default()
      || cfg
        .config_name
        .as_ref()
        // Has to match by name exactly
        .map(|name| name == config)
        .unwrap_or_default()
  })?;
  Some(ServiceConfigFile {
    service: spec.name?,
    file: config.file?,
  })
}
