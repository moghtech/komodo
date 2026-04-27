use std::process::Stdio;

use anyhow::{Context, anyhow};
use bollard::Docker;
use command::{CommandOutput, run_komodo_standard_command};
use komodo_client::entities::{
  TerminationSignal,
  docker::{task::*, *},
  update::Log,
};
use tokio::{io::AsyncWriteExt, process::Command};

pub mod compose;
pub mod config;
pub mod image;
pub mod secret;
pub mod stack;
pub mod stats;

mod container;
mod network;
mod node;
mod service;
mod swarm;
mod task;
mod volume;

pub struct DockerClient {
  docker: Docker,
}

impl DockerClient {
  pub fn connect() -> anyhow::Result<DockerClient> {
    let docker = Docker::connect_with_defaults()
      .context("Failed to connect to docker api. Docker monitoring won't work and will return empty results.")?;
    Ok(DockerClient { docker })
  }
}

/// Returns whether login was actually performed.
#[instrument("DockerLogin", skip(registry_token))]
pub async fn docker_login(
  domain: &str,
  account: &str,
  // For local token override from core.
  registry_token: Option<&str>,
) -> anyhow::Result<bool> {
  if domain.is_empty() || account.is_empty() {
    return Ok(false);
  }

  let registry_token = match registry_token {
    Some(token) => token,
    None => crate::helpers::registry_token(domain, account)?,
  };

  let log =
    docker_login_command(domain, account, registry_token).await?;

  if log.success() {
    return Ok(true);
  }

  let mut e = anyhow!("End of trace");
  for line in
    log.stderr.split('\n').filter(|line| !line.is_empty()).rev()
  {
    e = e.context(line.to_string());
  }
  for line in
    log.stdout.split('\n').filter(|line| !line.is_empty()).rev()
  {
    e = e.context(line.to_string());
  }
  Err(e.context(format!("Registry {domain} login error")))
}

async fn docker_login_command(
  domain: &str,
  account: &str,
  registry_token: &str,
) -> anyhow::Result<CommandOutput> {
  let mut child = Command::new("docker")
    .args([
      "login",
      domain,
      "--username",
      account,
      "--password-stdin",
    ])
    .kill_on_drop(true)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .context("Failed to spawn docker login")?;

  let mut stdin = child
    .stdin
    .take()
    .context("Failed to open docker login stdin")?;
  stdin.write_all(registry_token.as_bytes()).await.context(
    "Failed to write registry token to docker login stdin",
  )?;
  drop(stdin);

  Ok(CommandOutput::from(child.wait_with_output().await))
}

#[instrument("PullImage")]
pub async fn pull_image(image: &str) -> Log {
  let command = format!("docker pull {image}");
  run_komodo_standard_command("Docker Pull", None, command).await
}

pub fn stop_container_command(
  container_name: &str,
  signal: Option<TerminationSignal>,
  time: Option<i32>,
) -> String {
  let signal = signal
    .map(|signal| format!(" --signal {signal}"))
    .unwrap_or_default();
  let time = time
    .map(|time| format!(" --time {time}"))
    .unwrap_or_default();
  format!("docker stop{signal}{time} {container_name}")
}

fn convert_object_version(
  version: bollard::models::ObjectVersion,
) -> ObjectVersion {
  ObjectVersion {
    index: version.index,
  }
}

fn convert_driver(driver: bollard::models::Driver) -> Driver {
  Driver {
    name: driver.name,
    options: driver.options,
  }
}

fn convert_mount(mount: bollard::models::Mount) -> Mount {
  Mount {
    target: mount.target,
    source: mount.source,
    typ: mount.typ.map(convert_mount_type).unwrap_or_default(),
    read_only: mount.read_only,
    consistency: mount.consistency,
    bind_options: mount.bind_options.map(|options| {
      MountBindOptions {
        propagation: options
          .propagation
          .map(convert_mount_propogation)
          .unwrap_or_default(),
        non_recursive: options.non_recursive,
        create_mountpoint: options.create_mountpoint,
        read_only_non_recursive: options.read_only_non_recursive,
        read_only_force_recursive: options.read_only_force_recursive,
      }
    }),
    volume_options: mount.volume_options.map(|options| {
      MountVolumeOptions {
        no_copy: options.no_copy,
        labels: options.labels.unwrap_or_default(),
        driver_config: options.driver_config.map(|config| {
          MountVolumeOptionsDriverConfig {
            name: config.name,
            options: config.options.unwrap_or_default(),
          }
        }),
        subpath: options.subpath,
      }
    }),
    tmpfs_options: mount.tmpfs_options.map(|options| {
      MountTmpfsOptions {
        size_bytes: options.size_bytes,
        mode: options.mode,
      }
    }),
  }
}

fn convert_mount_type(
  typ: bollard::config::MountTypeEnum,
) -> MountTypeEnum {
  match typ {
    bollard::config::MountTypeEnum::EMPTY => MountTypeEnum::Empty,
    bollard::config::MountTypeEnum::BIND => MountTypeEnum::Bind,
    bollard::config::MountTypeEnum::VOLUME => MountTypeEnum::Volume,
    bollard::config::MountTypeEnum::IMAGE => MountTypeEnum::Image,
    bollard::config::MountTypeEnum::TMPFS => MountTypeEnum::Tmpfs,
    bollard::config::MountTypeEnum::NPIPE => MountTypeEnum::Npipe,
    bollard::config::MountTypeEnum::CLUSTER => MountTypeEnum::Cluster,
  }
}

fn convert_mount_propogation(
  propogation: bollard::config::MountBindOptionsPropagationEnum,
) -> MountBindOptionsPropagationEnum {
  match propogation {
    bollard::config::MountBindOptionsPropagationEnum::EMPTY => {
      MountBindOptionsPropagationEnum::Empty
    }
    bollard::config::MountBindOptionsPropagationEnum::PRIVATE => {
      MountBindOptionsPropagationEnum::Private
    }
    bollard::config::MountBindOptionsPropagationEnum::RPRIVATE => {
      MountBindOptionsPropagationEnum::Rprivate
    }
    bollard::config::MountBindOptionsPropagationEnum::SHARED => {
      MountBindOptionsPropagationEnum::Shared
    }
    bollard::config::MountBindOptionsPropagationEnum::RSHARED => {
      MountBindOptionsPropagationEnum::Rshared
    }
    bollard::config::MountBindOptionsPropagationEnum::SLAVE => {
      MountBindOptionsPropagationEnum::Slave
    }
    bollard::config::MountBindOptionsPropagationEnum::RSLAVE => {
      MountBindOptionsPropagationEnum::Rslave
    }
  }
}

fn convert_mount_point_type(
  typ: bollard::config::MountPointTypeEnum,
) -> MountTypeEnum {
  match typ {
    bollard::config::MountPointTypeEnum::EMPTY => {
      MountTypeEnum::Empty
    }
    bollard::config::MountPointTypeEnum::BIND => MountTypeEnum::Bind,
    bollard::config::MountPointTypeEnum::VOLUME => {
      MountTypeEnum::Volume
    }
    bollard::config::MountPointTypeEnum::IMAGE => {
      MountTypeEnum::Image
    }
    bollard::config::MountPointTypeEnum::TMPFS => {
      MountTypeEnum::Tmpfs
    }
    bollard::config::MountPointTypeEnum::NPIPE => {
      MountTypeEnum::Npipe
    }
    bollard::config::MountPointTypeEnum::CLUSTER => {
      MountTypeEnum::Cluster
    }
  }
}

fn convert_health_config(
  config: bollard::models::HealthConfig,
) -> HealthConfig {
  HealthConfig {
    test: config.test.unwrap_or_default(),
    interval: config.interval,
    timeout: config.timeout,
    retries: config.retries,
    start_period: config.start_period,
    start_interval: config.start_interval,
  }
}

fn convert_resources_ulimits(
  ulimit: bollard::models::ResourcesUlimits,
) -> ResourcesUlimits {
  ResourcesUlimits {
    name: ulimit.name,
    soft: ulimit.soft,
    hard: ulimit.hard,
  }
}

fn convert_resource_object(
  object: bollard::models::ResourceObject,
) -> ResourceObject {
  ResourceObject {
    nano_cpus: object.nano_cpus,
    memory_bytes: object.memory_bytes,
    generic_resources: object
      .generic_resources
      .map(convert_generic_resources),
  }
}

fn convert_generic_resources(
  resources: Vec<bollard::models::GenericResourcesInner>,
) -> Vec<GenericResourcesInner> {
  resources
    .into_iter()
    .map(|resource| GenericResourcesInner {
      named_resource_spec: resource.named_resource_spec.map(|spec| {
        GenericResourcesInnerNamedResourceSpec {
          kind: spec.kind,
          value: spec.value,
        }
      }),
      discrete_resource_spec: resource.discrete_resource_spec.map(
        |spec| GenericResourcesInnerDiscreteResourceSpec {
          kind: spec.kind,
          value: spec.value,
        },
      ),
    })
    .collect()
}

fn convert_platform(platform: bollard::models::Platform) -> Platform {
  Platform {
    architecture: platform.architecture,
    os: platform.os,
  }
}

fn convert_endpoint_spec_ports(
  ports: Vec<bollard::models::EndpointPortConfig>,
) -> Vec<EndpointPortConfig> {
  ports
    .into_iter()
    .map(|port| EndpointPortConfig {
      name: port.name,
      protocol: port.protocol.map(|protocol| match protocol {
        bollard::config::EndpointPortConfigProtocolEnum::EMPTY => EndpointPortConfigProtocolEnum::EMPTY,
        bollard::config::EndpointPortConfigProtocolEnum::TCP => EndpointPortConfigProtocolEnum::TCP,
        bollard::config::EndpointPortConfigProtocolEnum::UDP => EndpointPortConfigProtocolEnum::UDP,
        bollard::config::EndpointPortConfigProtocolEnum::SCTP => EndpointPortConfigProtocolEnum::SCTP,
      }),
      target_port: port.target_port,
      published_port: port.published_port,
      publish_mode: port.publish_mode.map(|protocol| match protocol {
        bollard::config::EndpointPortConfigPublishModeEnum::EMPTY => EndpointPortConfigPublishModeEnum::EMPTY,
        bollard::config::EndpointPortConfigPublishModeEnum::INGRESS => EndpointPortConfigPublishModeEnum::INGRESS,
        bollard::config::EndpointPortConfigPublishModeEnum::HOST => EndpointPortConfigPublishModeEnum::HOST,
      }),
    })
    .collect()
}

fn convert_task_spec(spec: bollard::models::TaskSpec) -> TaskSpec {
  TaskSpec {
    plugin_spec: spec.plugin_spec.map(|spec| TaskSpecPluginSpec {
      name: spec.name,
      remote: spec.remote,
      disabled: spec.disabled,
      plugin_privilege: spec.plugin_privilege.map(|privileges| {
        privileges
          .into_iter()
          .map(|privilege| PluginPrivilege {
            name: privilege.name,
            description: privilege.description,
            value: privilege.value,
          })
          .collect()
      }),
    }),
    container_spec: spec
      .container_spec
      .map(convert_task_spec_container_spec),
    network_attachment_spec: spec.network_attachment_spec.map(
      |spec| TaskSpecNetworkAttachmentSpec {
        container_id: spec.container_id,
      },
    ),
    resources: spec.resources.map(|resources| TaskSpecResources {
      limits: resources.limits.map(|limits| Limit {
        nano_cpus: limits.nano_cpus,
        memory_bytes: limits.memory_bytes,
        pids: limits.pids,
      }),
      reservations: resources
        .reservations
        .map(convert_resource_object),
    }),
    restart_policy: spec.restart_policy.map(|policy| {
      TaskSpecRestartPolicy {
        condition: policy
          .condition
          .map(convert_task_spec_restart_policy_condition),
        delay: policy.delay,
        max_attempts: policy.max_attempts,
        window: policy.window,
      }
    }),
    placement: spec.placement.map(|placement| TaskSpecPlacement {
      constraints: placement.constraints,
      preferences: placement.preferences.map(|preferences| {
        preferences
          .into_iter()
          .map(|preference| TaskSpecPlacementPreferences {
            spread: preference.spread.map(|spread| {
              TaskSpecPlacementSpread {
                spread_descriptor: spread.spread_descriptor,
              }
            }),
          })
          .collect()
      }),
      max_replicas: placement.max_replicas,
      platforms: placement.platforms.map(|platforms| {
        platforms.into_iter().map(convert_platform).collect()
      }),
    }),
    force_update: spec.force_update,
    runtime: spec.runtime,
    networks: spec.networks.map(|networks| {
      networks
        .into_iter()
        .map(|network| NetworkAttachmentConfig {
          target: network.target,
          aliases: network.aliases,
          driver_opts: network.driver_opts,
        })
        .collect()
    }),
    log_driver: spec.log_driver.map(|driver| TaskSpecLogDriver {
      name: driver.name,
      options: driver.options,
    }),
  }
}

fn convert_task_spec_container_spec(
  spec: bollard::models::TaskSpecContainerSpec,
) -> TaskSpecContainerSpec {
  TaskSpecContainerSpec {
    image: spec.image,
    labels: spec.labels,
    command: spec.command,
    args: spec.args,
    hostname: spec.hostname,
    env: spec.env,
    dir: spec.dir,
    user: spec.user,
    groups: spec.groups,
    privileges: spec.privileges.map(|privilege| {
      TaskSpecContainerSpecPrivileges {
        credential_spec: privilege.credential_spec.map(|spec| {
          TaskSpecContainerSpecPrivilegesCredentialSpec {
            config: spec.config,
            file: spec.file,
            registry: spec.registry,
          }
        }),
        se_linux_context: privilege.se_linux_context.map(|context| {
          TaskSpecContainerSpecPrivilegesSeLinuxContext {
            disable: context.disable,
            user: context.user,
            role: context.role,
            typ: context.typ,
            level: context.level,
          }
        }),
        seccomp: privilege.seccomp.map(|seccomp| {
          TaskSpecContainerSpecPrivilegesSeccomp {
            mode: seccomp.mode.map(|mode| match mode {
              bollard::config::TaskSpecContainerSpecPrivilegesSeccompModeEnum::EMPTY => TaskSpecContainerSpecPrivilegesSeccompModeEnum::EMPTY,
              bollard::config::TaskSpecContainerSpecPrivilegesSeccompModeEnum::DEFAULT => TaskSpecContainerSpecPrivilegesSeccompModeEnum::DEFAULT,
              bollard::config::TaskSpecContainerSpecPrivilegesSeccompModeEnum::UNCONFINED => TaskSpecContainerSpecPrivilegesSeccompModeEnum::UNCONFINED,
              bollard::config::TaskSpecContainerSpecPrivilegesSeccompModeEnum::CUSTOM => TaskSpecContainerSpecPrivilegesSeccompModeEnum::CUSTOM,
            }),
            profile: seccomp.profile,
          }
        }),
        app_armor: privilege.app_armor.map(|app_armor| {
          TaskSpecContainerSpecPrivilegesAppArmor {
            mode: app_armor.mode.map(|mode| match mode {
              bollard::config::TaskSpecContainerSpecPrivilegesAppArmorModeEnum::EMPTY => TaskSpecContainerSpecPrivilegesAppArmorModeEnum::EMPTY,
              bollard::config::TaskSpecContainerSpecPrivilegesAppArmorModeEnum::DEFAULT => TaskSpecContainerSpecPrivilegesAppArmorModeEnum::DEFAULT,
              bollard::config::TaskSpecContainerSpecPrivilegesAppArmorModeEnum::DISABLED => TaskSpecContainerSpecPrivilegesAppArmorModeEnum::DISABLED,
            }),
          }
        }),
        no_new_privileges: privilege.no_new_privileges,
      }
    }),
    tty: spec.tty,
    open_stdin: spec.open_stdin,
    read_only: spec.read_only,
    mounts: spec.mounts.map(|mounts| mounts.into_iter().map(convert_mount).collect()),
    stop_signal: spec.stop_signal,
    stop_grace_period: spec.stop_grace_period,
    health_check: spec.health_check.map(convert_health_config),
    hosts: spec.hosts,
    dns_config: spec.dns_config.map(|config| TaskSpecContainerSpecDnsConfig {
      nameservers: config.nameservers,
      search: config.search,
      options: config.options,
    }),
    secrets: spec.secrets.map(|secrets| secrets.into_iter().map(|secret| TaskSpecContainerSpecSecrets {
      file: secret.file.map(|file| TaskSpecContainerSpecFile {
        name: file.name,
        uid: file.uid,
        gid: file.gid,
        mode: file.mode,
      }),
      secret_id: secret.secret_id,
      secret_name: secret.secret_name,
    }).collect()),
    oom_score_adj: spec.oom_score_adj,
    configs: spec.configs.map(|configs| configs.into_iter().map(|config| TaskSpecContainerSpecConfigs {
      file: config.file.map(|file| TaskSpecContainerSpecFile {
        name: file.name,
        uid: file.uid,
        gid: file.gid,
        mode: file.mode,
      }),
      config_id: config.config_id,
      config_name: config.config_name,
    }).collect()),
    isolation: spec.isolation.map(|isolation| match isolation {
      bollard::config::TaskSpecContainerSpecIsolationEnum::DEFAULT => TaskSpecContainerSpecIsolationEnum::DEFAULT,
      bollard::config::TaskSpecContainerSpecIsolationEnum::PROCESS => TaskSpecContainerSpecIsolationEnum::PROCESS,
      bollard::config::TaskSpecContainerSpecIsolationEnum::HYPERV => TaskSpecContainerSpecIsolationEnum::HYPERV,
      bollard::config::TaskSpecContainerSpecIsolationEnum::EMPTY => TaskSpecContainerSpecIsolationEnum::EMPTY,
    }),
    init: spec.init,
    sysctls: spec.sysctls,
    capability_add: spec.capability_add,
    capability_drop: spec.capability_drop,
    ulimits: spec.ulimits.map(|ulimits| ulimits.into_iter().map(convert_resources_ulimits).collect()),
  }
}

fn convert_task_spec_restart_policy_condition(
  condition: bollard::config::TaskSpecRestartPolicyConditionEnum,
) -> TaskSpecRestartPolicyConditionEnum {
  match condition {
    bollard::config::TaskSpecRestartPolicyConditionEnum::EMPTY => TaskSpecRestartPolicyConditionEnum::EMPTY,
    bollard::config::TaskSpecRestartPolicyConditionEnum::NONE => TaskSpecRestartPolicyConditionEnum::NONE,
    bollard::config::TaskSpecRestartPolicyConditionEnum::ON_FAILURE => TaskSpecRestartPolicyConditionEnum::ON_FAILURE,
    bollard::config::TaskSpecRestartPolicyConditionEnum::ANY => TaskSpecRestartPolicyConditionEnum::ANY,
  }
}

fn convert_tls_info(tls_info: bollard::models::TlsInfo) -> TlsInfo {
  TlsInfo {
    trust_root: tls_info.trust_root,
    cert_issuer_subject: tls_info.cert_issuer_subject,
    cert_issuer_public_key: tls_info.cert_issuer_public_key,
  }
}
