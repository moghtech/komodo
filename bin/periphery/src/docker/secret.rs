use std::fmt::Write;

use anyhow::Context;
use bollard::query_parameters::ListSecretsOptions;
use command::{
  run_komodo_shell_command, run_komodo_standard_command,
};
use futures_util::{TryStreamExt as _, stream::FuturesUnordered};
use komodo_client::entities::{
  all_logs_success,
  docker::{
    secret::{SecretSpec, SwarmSecret, SwarmSecretListItem},
    service::SwarmService,
    task::TaskSpecContainerSpecFile,
  },
  random_string,
  update::Log,
};
use periphery_client::api::swarm::CreateSwarmSecret;

use super::DockerClient;

impl DockerClient {
  pub async fn list_swarm_secrets(
    &self,
  ) -> anyhow::Result<Vec<SwarmSecretListItem>> {
    let mut secrets = self
      .docker
      .list_secrets(Option::<ListSecretsOptions>::None)
      .await
      .context("Failed to query for swarm secret list")?
      .into_iter()
      .map(convert_secret_list_item)
      .collect::<Vec<_>>();

    secrets.sort_by(|a, b| {
      a.name
        .cmp(&b.name)
        .then_with(|| b.updated_at.cmp(&a.updated_at))
    });

    Ok(secrets)
  }

  pub async fn inspect_swarm_secret(
    &self,
    secret_id: &str,
  ) -> anyhow::Result<SwarmSecret> {
    self
      .docker
      .inspect_secret(secret_id)
      .await
      .map(convert_secret)
      .with_context(|| {
        format!(
          "Failed to query for swarm secret with id {secret_id}"
        )
      })
  }
}

fn convert_secret_list_item(
  secret: bollard::models::Secret,
) -> SwarmSecretListItem {
  let (name, driver, templating) = secret
    .spec
    .map(|spec| {
      (
        spec.name,
        spec.driver.map(|driver| driver.name),
        spec.templating.map(|driver| driver.name),
      )
    })
    .unwrap_or_default();
  SwarmSecretListItem {
    id: secret.id,
    name,
    driver,
    templating,
    created_at: secret.created_at,
    updated_at: secret.updated_at,
  }
}

fn convert_secret(secret: bollard::models::Secret) -> SwarmSecret {
  SwarmSecret {
    id: secret.id,
    version: secret.version.map(super::convert_object_version),
    created_at: secret.created_at,
    updated_at: secret.updated_at,
    spec: secret.spec.map(|spec| SecretSpec {
      name: spec.name,
      labels: spec.labels,
      data: spec.data,
      driver: spec.driver.map(super::convert_driver),
      templating: spec.templating.map(super::convert_driver),
    }),
  }
}

pub async fn create_swarm_secret(
  CreateSwarmSecret {
    name,
    data,
    driver,
    labels,
    template_driver,
  }: &CreateSwarmSecret,
) -> anyhow::Result<Log> {
  let mut command = String::from("docker secret create");

  if let Some(driver) = driver {
    write!(&mut command, " --driver {driver}")?;
  }

  for label in labels {
    write!(&mut command, " --label {label}")?;
  }

  if let Some(driver) = template_driver {
    write!(&mut command, " --template-driver {driver}")?;
  }

  let mut sanitized_command = command.clone();

  write!(
    &mut command,
    r#" {name} - <<'EOF'
{}
EOF"#,
    data.trim()
  )?;

  write!(
    &mut sanitized_command,
    r#" {name} - <<'EOF'
<secret-data>
EOF"#
  )?;

  let mut log =
    run_komodo_shell_command("Create Secret", None, command).await;

  log.command = sanitized_command;

  Ok(log)
}

pub async fn remove_swarm_secrets(
  secrets: impl Iterator<Item = &str>,
) -> Log {
  let mut command = String::from("docker secret rm");
  for secret in secrets {
    command += " ";
    command += secret;
  }
  run_komodo_standard_command("Remove Swarm Secrets", None, command)
    .await
}

pub async fn recreate_swarm_secret(
  secret: &CreateSwarmSecret,
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let remove =
    remove_swarm_secrets([secret.name.as_str()].into_iter()).await;
  let success = remove.success;
  logs.push(remove);
  if !success {
    return Ok(());
  }
  let log = create_swarm_secret(secret).await?;
  logs.push(log);
  Ok(())
}

struct ServiceSecretFile {
  /// Service name
  service: String,
  /// Secret file spec
  file: TaskSpecContainerSpecFile,
}

impl DockerClient {
  pub async fn rotate_swarm_secret(
    &self,
    secret: &str,
    data: String,
    logs: &mut Vec<Log>,
  ) -> anyhow::Result<()> {
    let secret = self.inspect_swarm_secret(secret).await?;
    let secret_id = secret.id.context("Failed to get secret id")?;
    let spec = secret.spec.context("Failed to get secret spec")?;
    let name = spec.name.context("Failed to get secret name")?;
    let driver = spec.driver.map(|driver| driver.name);
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

    let create_secret = CreateSwarmSecret {
      name,
      data,
      driver,
      labels,
      template_driver,
    };

    let services = self
      .filter_map_swarm_services(|service| {
        extract_from_service(service, &secret_id)
      })
      .await?;
    if services.is_empty() {
      return recreate_swarm_secret(&create_secret, logs).await;
    }

    // Create a tmp secret for rotation
    let tmp_create_config = CreateSwarmSecret {
      name: format!(
        "{}-tmp-{}",
        create_secret.name,
        random_string(10)
      ),
      data: create_secret.data.clone(),
      driver: create_secret.driver.clone(),
      labels: create_secret.labels.clone(),
      template_driver: create_secret.template_driver.clone(),
    };
    let log = create_swarm_secret(&tmp_create_config).await?;
    logs.push(log);
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update services to tmp
    switch_services_secret(
      &services,
      &create_secret.name,
      &tmp_create_config.name,
      logs,
    )
    .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Recreate actual secret
    recreate_swarm_secret(&create_secret, logs).await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Update back to original
    switch_services_secret(
      &services,
      &tmp_create_config.name,
      &create_secret.name,
      logs,
    )
    .await?;
    if !all_logs_success(logs) {
      return Ok(());
    }

    // Remove tmp secret
    let log = remove_swarm_secrets(
      [tmp_create_config.name.as_str()].into_iter(),
    )
    .await;
    logs.push(log);

    Ok(())
  }
}

async fn switch_services_secret(
  services: &[ServiceSecretFile],
  from: &str,
  to: &str,
  logs: &mut Vec<Log>,
) -> anyhow::Result<()> {
  let res = services
    .iter()
    .map(|service| async move {
      switch_service_secret(&service.service, from, to, &service.file)
        .await
    })
    .collect::<FuturesUnordered<_>>()
    .try_collect::<Vec<_>>()
    .await?;
  logs.extend(res);
  Ok(())
}

async fn switch_service_secret(
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
    "docker service update --secret-rm {from} --secret-add source={to}"
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
    "Switch Service Secret",
    None,
    command,
  )
  .await;

  Ok(log)
}

fn extract_from_service(
  service: SwarmService,
  secret_id: &str,
) -> Option<ServiceSecretFile> {
  let spec = service.spec?;
  let secrets = spec.task_template?.container_spec?.secrets?;
  let secret = secrets.into_iter().find(|cfg| {
    cfg
      .secret_id
      .as_ref()
      .map(|id| id == secret_id)
      .unwrap_or_default()
  })?;
  Some(ServiceSecretFile {
    service: spec.name?,
    file: secret.file?,
  })
}
