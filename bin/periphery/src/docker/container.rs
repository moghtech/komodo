use std::collections::HashMap;

use anyhow::Context;
use bollard::query_parameters::{
  InspectContainerOptions, ListContainersOptions,
};
use komodo_client::entities::docker::{container::*, *};

use crate::state::container_stats;

use super::{
  DockerClient, convert_health_config, convert_mount,
  convert_mount_point_type, convert_resources_ulimits,
};

impl DockerClient {
  pub async fn list_containers(
    &self,
  ) -> anyhow::Result<Vec<ContainerListItem>> {
    let containers = self
      .docker
      .list_containers(Some(ListContainersOptions {
        all: true,
        ..Default::default()
      }))
      .await?;
    let stats = container_stats().load();
    let mut containers = containers
      .into_iter()
      .flat_map(|container| {
        let name = container
          .names
          .context("no names on container")?
          .pop()
          .context("no names on container (empty vec)")?
          .replace('/', "");
        let stats = stats.get(&name).cloned();
        anyhow::Ok(ContainerListItem {
          server_id: None,
          name,
          stats,
          id: container.id,
          image: container.image,
          image_id: container.image_id,
          created: container.created,
          size_rw: container.size_rw,
          size_root_fs: container.size_root_fs,
          state: convert_summary_container_state(
            container.state.context("no container state")?,
          ),
          status: container.status,
          network_mode: container
            .host_config
            .and_then(|config| config.network_mode),
          networks: container
            .network_settings
            .and_then(|settings| {
              settings.networks.map(|networks| {
                let mut keys =
                  networks.into_keys().collect::<Vec<_>>();
                keys.sort();
                keys
              })
            })
            .unwrap_or_default(),
          ports: container
            .ports
            .map(|ports| {
              ports.into_iter().map(convert_port).collect()
            })
            .unwrap_or_default(),
          volumes: container
            .mounts
            .map(|settings| {
              settings
                .into_iter()
                .filter_map(|mount| mount.name)
                .collect()
            })
            .unwrap_or_default(),
          labels: container.labels.unwrap_or_default(),
        })
      })
      .collect::<Vec<_>>();
    let container_id_to_network = containers
      .iter()
      .filter_map(|c| Some((c.id.clone()?, c.network_mode.clone()?)))
      .collect::<HashMap<_, _>>();
    // Fix containers which use `container:container_id` network_mode,
    // by replacing with the referenced network mode.
    containers.iter_mut().for_each(|container| {
      let Some(network_name) = &container.network_mode else {
        return;
      };
      let Some(container_id) =
        network_name.strip_prefix("container:")
      else {
        return;
      };
      container.network_mode =
        container_id_to_network.get(container_id).cloned();
    });
    containers.sort_by(|a, b| {
      a.state.cmp(&b.state).then_with(|| a.name.cmp(&b.name))
    });
    Ok(containers)
  }

  pub async fn inspect_container(
    &self,
    container_name: &str,
  ) -> anyhow::Result<Container> {
    let container = self
      .docker
      .inspect_container(
        container_name,
        InspectContainerOptions { size: true }.into(),
      )
      .await?;
    Ok(Container {
      id: container.id,
      created: container.created,
      path: container.path,
      args: container.args.unwrap_or_default(),
      state: container.state.map(|state| ContainerState {
        status: state
          .status
          .map(convert_container_state_status)
          .unwrap_or_default(),
        running: state.running,
        paused: state.paused,
        restarting: state.restarting,
        oom_killed: state.oom_killed,
        dead: state.dead,
        pid: state.pid,
        exit_code: state.exit_code,
        error: state.error,
        started_at: state.started_at,
        finished_at: state.finished_at,
        health: state.health.map(|health| ContainerHealth {
          status: health
            .status
            .map(convert_health_status)
            .unwrap_or_default(),
          failing_streak: health.failing_streak,
          log: health
            .log
            .map(|log| {
              log
                .into_iter()
                .map(convert_health_check_result)
                .collect()
            })
            .unwrap_or_default(),
        }),
      }),
      image: container.image,
      resolv_conf_path: container.resolv_conf_path,
      hostname_path: container.hostname_path,
      hosts_path: container.hosts_path,
      log_path: container.log_path,
      name: container.name,
      restart_count: container.restart_count,
      driver: container.driver,
      platform: container.platform,
      mount_label: container.mount_label,
      process_label: container.process_label,
      app_armor_profile: container.app_armor_profile,
      exec_ids: container.exec_ids.unwrap_or_default(),
      host_config: container.host_config.map(|config| HostConfig {
        cpu_shares: config.cpu_shares,
        memory: config.memory,
        cgroup_parent: config.cgroup_parent,
        blkio_weight: config.blkio_weight,
        blkio_weight_device: config
          .blkio_weight_device
          .unwrap_or_default()
          .into_iter()
          .map(|device| ResourcesBlkioWeightDevice {
            path: device.path,
            weight: device.weight,
          })
          .collect(),
        blkio_device_read_bps: config
          .blkio_device_read_bps
          .unwrap_or_default()
          .into_iter()
          .map(|bp| ThrottleDevice {
            path: bp.path,
            rate: bp.rate,
          })
          .collect(),
        blkio_device_write_bps: config
          .blkio_device_write_bps
          .unwrap_or_default()
          .into_iter()
          .map(|bp| ThrottleDevice {
            path: bp.path,
            rate: bp.rate,
          })
          .collect(),
        blkio_device_read_iops: config
          .blkio_device_read_iops
          .unwrap_or_default()
          .into_iter()
          .map(|iops| ThrottleDevice {
            path: iops.path,
            rate: iops.rate,
          })
          .collect(),
        blkio_device_write_iops: config
          .blkio_device_write_iops
          .unwrap_or_default()
          .into_iter()
          .map(|iops| ThrottleDevice {
            path: iops.path,
            rate: iops.rate,
          })
          .collect(),
        cpu_period: config.cpu_period,
        cpu_quota: config.cpu_quota,
        cpu_realtime_period: config.cpu_realtime_period,
        cpu_realtime_runtime: config.cpu_realtime_runtime,
        cpuset_cpus: config.cpuset_cpus,
        cpuset_mems: config.cpuset_mems,
        devices: config
          .devices
          .unwrap_or_default()
          .into_iter()
          .map(|device| DeviceMapping {
            path_on_host: device.path_on_host,
            path_in_container: device.path_in_container,
            cgroup_permissions: device.cgroup_permissions,
          })
          .collect(),
        device_cgroup_rules: config
          .device_cgroup_rules
          .unwrap_or_default(),
        device_requests: config
          .device_requests
          .unwrap_or_default()
          .into_iter()
          .map(|request| DeviceRequest {
            driver: request.driver,
            count: request.count,
            device_ids: request.device_ids.unwrap_or_default(),
            capabilities: request.capabilities.unwrap_or_default(),
            options: request.options.unwrap_or_default(),
          })
          .collect(),
        kernel_memory_tcp: config.kernel_memory_tcp,
        memory_reservation: config.memory_reservation,
        memory_swap: config.memory_swap,
        memory_swappiness: config.memory_swappiness,
        nano_cpus: config.nano_cpus,
        oom_kill_disable: config.oom_kill_disable,
        init: config.init,
        pids_limit: config.pids_limit,
        ulimits: config
          .ulimits
          .unwrap_or_default()
          .into_iter()
          .map(convert_resources_ulimits)
          .collect(),
        cpu_count: config.cpu_count,
        cpu_percent: config.cpu_percent,
        io_maximum_iops: config.io_maximum_iops,
        io_maximum_bandwidth: config.io_maximum_bandwidth,
        binds: config.binds.unwrap_or_default(),
        container_id_file: config.container_id_file,
        log_config: config.log_config.map(|config| {
          HostConfigLogConfig {
            typ: config.typ,
            config: config.config.unwrap_or_default(),
          }
        }),
        network_mode: config.network_mode,
        port_bindings: config
          .port_bindings
          .unwrap_or_default()
          .into_iter()
          .map(|(k, v)| {
            (
              k,
              v.unwrap_or_default()
                .into_iter()
                .map(|v| PortBinding {
                  host_ip: v.host_ip,
                  host_port: v.host_port,
                })
                .collect(),
            )
          })
          .collect(),
        restart_policy: config.restart_policy.map(|policy| {
          RestartPolicy {
            name: policy
              .name
              .map(convert_restart_policy)
              .unwrap_or_default(),
            maximum_retry_count: policy.maximum_retry_count,
          }
        }),
        auto_remove: config.auto_remove,
        volume_driver: config.volume_driver,
        volumes_from: config.volumes_from.unwrap_or_default(),
        mounts: config
          .mounts
          .unwrap_or_default()
          .into_iter()
          .map(convert_mount)
          .collect(),
        console_size: config
          .console_size
          .map(|v| v.into_iter().map(|s| s as i32).collect())
          .unwrap_or_default(),
        annotations: config.annotations.unwrap_or_default(),
        cap_add: config.cap_add.unwrap_or_default(),
        cap_drop: config.cap_drop.unwrap_or_default(),
        cgroupns_mode: config
          .cgroupns_mode
          .map(convert_cgroupns_mode),
        dns: config.dns.unwrap_or_default(),
        dns_options: config.dns_options.unwrap_or_default(),
        dns_search: config.dns_search.unwrap_or_default(),
        extra_hosts: config.extra_hosts.unwrap_or_default(),
        group_add: config.group_add.unwrap_or_default(),
        ipc_mode: config.ipc_mode,
        cgroup: config.cgroup,
        links: config.links.unwrap_or_default(),
        oom_score_adj: config.oom_score_adj,
        pid_mode: config.pid_mode,
        privileged: config.privileged,
        publish_all_ports: config.publish_all_ports,
        readonly_rootfs: config.readonly_rootfs,
        security_opt: config.security_opt.unwrap_or_default(),
        storage_opt: config.storage_opt.unwrap_or_default(),
        tmpfs: config.tmpfs.unwrap_or_default(),
        uts_mode: config.uts_mode,
        userns_mode: config.userns_mode,
        shm_size: config.shm_size,
        sysctls: config.sysctls.unwrap_or_default(),
        runtime: config.runtime,
        isolation: config
          .isolation
          .map(convert_isolation_mode)
          .unwrap_or_default(),
        masked_paths: config.masked_paths.unwrap_or_default(),
        readonly_paths: config.readonly_paths.unwrap_or_default(),
      }),
      graph_driver: container.graph_driver.map(|driver| {
        GraphDriverData {
          name: driver.name,
          data: driver.data,
        }
      }),
      size_rw: container.size_rw,
      size_root_fs: container.size_root_fs,
      mounts: container
        .mounts
        .unwrap_or_default()
        .into_iter()
        .map(|mount| MountPoint {
          typ: mount
            .typ
            .map(convert_mount_point_type)
            .unwrap_or_default(),
          name: mount.name,
          source: mount.source,
          destination: mount.destination,
          driver: mount.driver,
          mode: mount.mode,
          rw: mount.rw,
          propagation: mount.propagation,
        })
        .collect(),
      config: container.config.map(|config| ContainerConfig {
        hostname: config.hostname,
        domainname: config.domainname,
        user: config.user,
        attach_stdin: config.attach_stdin,
        attach_stdout: config.attach_stdout,
        attach_stderr: config.attach_stderr,
        exposed_ports: config
          .exposed_ports
          .unwrap_or_default()
          .into_keys()
          .map(|k| (k, Default::default()))
          .collect(),
        tty: config.tty,
        open_stdin: config.open_stdin,
        stdin_once: config.stdin_once,
        env: config.env.unwrap_or_default(),
        cmd: config.cmd.unwrap_or_default(),
        healthcheck: config.healthcheck.map(convert_health_config),
        args_escaped: config.args_escaped,
        image: config.image,
        volumes: config
          .volumes
          .unwrap_or_default()
          .into_keys()
          .map(|k| (k, Default::default()))
          .collect(),
        working_dir: config.working_dir,
        entrypoint: config.entrypoint.unwrap_or_default(),
        network_disabled: config.network_disabled,
        mac_address: config.mac_address,
        on_build: config.on_build.unwrap_or_default(),
        labels: config.labels.unwrap_or_default(),
        stop_signal: config.stop_signal,
        stop_timeout: config.stop_timeout,
        shell: config.shell.unwrap_or_default(),
      }),
      network_settings: container.network_settings.map(|settings| {
        NetworkSettings {
          bridge: settings.bridge,
          sandbox_id: settings.sandbox_id,
          ports: settings
            .ports
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
              (
                k,
                v.unwrap_or_default()
                  .into_iter()
                  .map(|v| PortBinding {
                    host_ip: v.host_ip,
                    host_port: v.host_port,
                  })
                  .collect(),
              )
            })
            .collect(),
          sandbox_key: settings.sandbox_key,
          networks: settings
            .networks
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
              (
                k,
                EndpointSettings {
                  ipam_config: v.ipam_config.map(|ipam| {
                    EndpointIpamConfig {
                      ipv4_address: ipam.ipv4_address,
                      ipv6_address: ipam.ipv6_address,
                      link_local_ips: ipam
                        .link_local_ips
                        .unwrap_or_default(),
                    }
                  }),
                  links: v.links.unwrap_or_default(),
                  mac_address: v.mac_address,
                  aliases: v.aliases.unwrap_or_default(),
                  network_id: v.network_id,
                  endpoint_id: v.endpoint_id,
                  gateway: v.gateway,
                  ip_address: v.ip_address,
                  ip_prefix_len: v.ip_prefix_len,
                  ipv6_gateway: v.ipv6_gateway,
                  global_ipv6_address: v.global_ipv6_address,
                  global_ipv6_prefix_len: v.global_ipv6_prefix_len,
                  driver_opts: v.driver_opts.unwrap_or_default(),
                  dns_names: v.dns_names.unwrap_or_default(),
                },
              )
            })
            .collect(),
        }
      }),
    })
  }
}

fn convert_summary_container_state(
  state: bollard::secret::ContainerSummaryStateEnum,
) -> ContainerStateStatusEnum {
  match state {
    bollard::secret::ContainerSummaryStateEnum::EMPTY => {
      ContainerStateStatusEnum::Empty
    }
    bollard::secret::ContainerSummaryStateEnum::CREATED => {
      ContainerStateStatusEnum::Created
    }
    bollard::secret::ContainerSummaryStateEnum::RUNNING => {
      ContainerStateStatusEnum::Running
    }
    bollard::secret::ContainerSummaryStateEnum::PAUSED => {
      ContainerStateStatusEnum::Paused
    }
    bollard::secret::ContainerSummaryStateEnum::RESTARTING => {
      ContainerStateStatusEnum::Restarting
    }
    bollard::secret::ContainerSummaryStateEnum::EXITED => {
      ContainerStateStatusEnum::Exited
    }
    bollard::secret::ContainerSummaryStateEnum::REMOVING => {
      ContainerStateStatusEnum::Removing
    }
    bollard::secret::ContainerSummaryStateEnum::DEAD => {
      ContainerStateStatusEnum::Dead
    }
  }
}

fn convert_container_state_status(
  state: bollard::secret::ContainerStateStatusEnum,
) -> ContainerStateStatusEnum {
  match state {
    bollard::secret::ContainerStateStatusEnum::EMPTY => {
      ContainerStateStatusEnum::Empty
    }
    bollard::secret::ContainerStateStatusEnum::CREATED => {
      ContainerStateStatusEnum::Created
    }
    bollard::secret::ContainerStateStatusEnum::RUNNING => {
      ContainerStateStatusEnum::Running
    }
    bollard::secret::ContainerStateStatusEnum::PAUSED => {
      ContainerStateStatusEnum::Paused
    }
    bollard::secret::ContainerStateStatusEnum::RESTARTING => {
      ContainerStateStatusEnum::Restarting
    }
    bollard::secret::ContainerStateStatusEnum::EXITED => {
      ContainerStateStatusEnum::Exited
    }
    bollard::secret::ContainerStateStatusEnum::REMOVING => {
      ContainerStateStatusEnum::Removing
    }
    bollard::secret::ContainerStateStatusEnum::DEAD => {
      ContainerStateStatusEnum::Dead
    }
  }
}

fn convert_port_type(
  typ: bollard::secret::PortTypeEnum,
) -> PortTypeEnum {
  match typ {
    bollard::secret::PortTypeEnum::EMPTY => PortTypeEnum::EMPTY,
    bollard::secret::PortTypeEnum::TCP => PortTypeEnum::TCP,
    bollard::secret::PortTypeEnum::UDP => PortTypeEnum::UDP,
    bollard::secret::PortTypeEnum::SCTP => PortTypeEnum::SCTP,
  }
}
fn convert_port(port: bollard::secret::Port) -> Port {
  Port {
    ip: port.ip,
    private_port: port.private_port,
    public_port: port.public_port,
    typ: port.typ.map(convert_port_type).unwrap_or_default(),
  }
}

fn convert_health_status(
  status: bollard::secret::HealthStatusEnum,
) -> HealthStatusEnum {
  match status {
    bollard::secret::HealthStatusEnum::EMPTY => {
      HealthStatusEnum::Empty
    }
    bollard::secret::HealthStatusEnum::NONE => HealthStatusEnum::None,
    bollard::secret::HealthStatusEnum::STARTING => {
      HealthStatusEnum::Starting
    }
    bollard::secret::HealthStatusEnum::HEALTHY => {
      HealthStatusEnum::Healthy
    }
    bollard::secret::HealthStatusEnum::UNHEALTHY => {
      HealthStatusEnum::Unhealthy
    }
  }
}

fn convert_health_check_result(
  check: bollard::secret::HealthcheckResult,
) -> HealthcheckResult {
  HealthcheckResult {
    start: check.start,
    end: check.end,
    exit_code: check.exit_code,
    output: check.output,
  }
}

fn convert_restart_policy(
  policy: bollard::secret::RestartPolicyNameEnum,
) -> RestartPolicyNameEnum {
  match policy {
    bollard::secret::RestartPolicyNameEnum::EMPTY => {
      RestartPolicyNameEnum::Empty
    }
    bollard::secret::RestartPolicyNameEnum::NO => {
      RestartPolicyNameEnum::No
    }
    bollard::secret::RestartPolicyNameEnum::ALWAYS => {
      RestartPolicyNameEnum::Always
    }
    bollard::secret::RestartPolicyNameEnum::UNLESS_STOPPED => {
      RestartPolicyNameEnum::UnlessStopped
    }
    bollard::secret::RestartPolicyNameEnum::ON_FAILURE => {
      RestartPolicyNameEnum::OnFailure
    }
  }
}

fn convert_cgroupns_mode(
  mode: bollard::secret::HostConfigCgroupnsModeEnum,
) -> HostConfigCgroupnsModeEnum {
  match mode {
    bollard::secret::HostConfigCgroupnsModeEnum::EMPTY => {
      HostConfigCgroupnsModeEnum::Empty
    }
    bollard::secret::HostConfigCgroupnsModeEnum::PRIVATE => {
      HostConfigCgroupnsModeEnum::Private
    }
    bollard::secret::HostConfigCgroupnsModeEnum::HOST => {
      HostConfigCgroupnsModeEnum::Host
    }
  }
}

fn convert_isolation_mode(
  isolation: bollard::secret::HostConfigIsolationEnum,
) -> HostConfigIsolationEnum {
  match isolation {
    bollard::secret::HostConfigIsolationEnum::EMPTY => {
      HostConfigIsolationEnum::Empty
    }
    bollard::secret::HostConfigIsolationEnum::DEFAULT => {
      HostConfigIsolationEnum::Default
    }
    bollard::secret::HostConfigIsolationEnum::PROCESS => {
      HostConfigIsolationEnum::Process
    }
    bollard::secret::HostConfigIsolationEnum::HYPERV => {
      HostConfigIsolationEnum::Hyperv
    }
  }
}
