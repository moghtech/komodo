use std::{collections::HashMap, sync::OnceLock};

use anyhow::{Context, anyhow};
use bollard::{
  Docker,
  container::{InspectContainerOptions, ListContainersOptions},
  network::InspectNetworkOptions,
};
use command::run_komodo_command;
use komodo_client::entities::{
  TerminationSignal,
  docker::{
    ContainerConfig, GraphDriverData, HealthConfig, PortBinding,
    container::*, image::*, network::*, volume::*,
  },
  to_docker_compatible_name,
  update::Log,
};
use run_command::async_run_command;

pub fn docker_client() -> &'static DockerClient {
  static DOCKER_CLIENT: OnceLock<DockerClient> = OnceLock::new();
  DOCKER_CLIENT.get_or_init(Default::default)
}

pub struct DockerClient {
  docker: Docker,
}

impl Default for DockerClient {
  fn default() -> DockerClient {
    DockerClient {
      docker: Docker::connect_with_defaults()
        .expect("failed to connect to docker daemon"),
    }
  }
}

impl DockerClient {
  pub async fn list_containers(
    &self,
  ) -> anyhow::Result<Vec<ContainerListItem>> {
    let mut containers = self
      .docker
      .list_containers(Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
      }))
      .await?
      .into_iter()
      .flat_map(|container| {
        anyhow::Ok(ContainerListItem {
          server_id: None,
          name: container
            .names
            .context("no names on container")?
            .pop()
            .context("no names on container (empty vec)")?
            .replace('/', ""),
          id: container.id,
          image: container.image,
          image_id: container.image_id,
          created: container.created,
          size_rw: container.size_rw,
          size_root_fs: container.size_root_fs,
          state: container
            .state
            .context("no container state")?
            .parse()
            .context("failed to parse container state")?,
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
      args: container
        .args
        .unwrap_or_default(),
      state: container.state.map(|state| ContainerState {
        status: state
          .status
          .map(|status| match status {
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
            bollard::secret::ContainerStateStatusEnum::REMOVING => {
              ContainerStateStatusEnum::Removing
            }
            bollard::secret::ContainerStateStatusEnum::EXITED => {
              ContainerStateStatusEnum::Exited
            }
            bollard::secret::ContainerStateStatusEnum::DEAD => {
              ContainerStateStatusEnum::Dead
            }
          })
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
            .map(|status| match status {
              bollard::secret::HealthStatusEnum::EMPTY => {
                HealthStatusEnum::Empty
              }
              bollard::secret::HealthStatusEnum::NONE => {
                HealthStatusEnum::None
              }
              bollard::secret::HealthStatusEnum::STARTING => {
                HealthStatusEnum::Starting
              }
              bollard::secret::HealthStatusEnum::HEALTHY => {
                HealthStatusEnum::Healthy
              }
              bollard::secret::HealthStatusEnum::UNHEALTHY => {
                HealthStatusEnum::Unhealthy
              }
            })
            .unwrap_or_default(),
          failing_streak: health.failing_streak,
          log: health
            .log
            .map(|log| {
              log
                .into_iter()
                .map(|log| HealthcheckResult {
                  start: log.start,
                  end: log.end,
                  exit_code: log.exit_code,
                  output: log.output,
                })
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
          .map(|ulimit| ResourcesUlimits {
            name: ulimit.name,
            soft: ulimit.soft,
            hard: ulimit.hard,
        })
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
          .map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|v| PortBinding {
            host_ip: v.host_ip,
            host_port: v.host_port,
        }).collect()))
          .collect(),
        restart_policy: config.restart_policy.map(|policy| {
          RestartPolicy {
            name: policy.name.map(|policy| match policy {
              bollard::secret::RestartPolicyNameEnum::EMPTY => RestartPolicyNameEnum::Empty,
              bollard::secret::RestartPolicyNameEnum::NO => RestartPolicyNameEnum::No,
              bollard::secret::RestartPolicyNameEnum::ALWAYS => RestartPolicyNameEnum::Always,
              bollard::secret::RestartPolicyNameEnum::UNLESS_STOPPED => RestartPolicyNameEnum::UnlessStopped,
              bollard::secret::RestartPolicyNameEnum::ON_FAILURE => RestartPolicyNameEnum::OnFailure,
            }).unwrap_or_default(),
            maximum_retry_count: policy.maximum_retry_count,
          }
        }),
        auto_remove: config.auto_remove,
        volume_driver: config.volume_driver,
        volumes_from: config.volumes_from.unwrap_or_default(),
        mounts: config.mounts
        .unwrap_or_default().into_iter()
        .map(|mount| ContainerMount {
            target: mount.target,
            source: mount.source,
            typ: mount.typ.map(|typ| match typ {
                bollard::secret::MountTypeEnum::EMPTY => MountTypeEnum::Empty,
                bollard::secret::MountTypeEnum::BIND => MountTypeEnum::Bind,
                bollard::secret::MountTypeEnum::VOLUME => MountTypeEnum::Volume,
                bollard::secret::MountTypeEnum::TMPFS => MountTypeEnum::Tmpfs,
                bollard::secret::MountTypeEnum::NPIPE => MountTypeEnum::Npipe,
                bollard::secret::MountTypeEnum::CLUSTER => MountTypeEnum::Cluster,
            }).unwrap_or_default(),
            read_only: mount.read_only,
            consistency: mount.consistency,
            bind_options: mount.bind_options.map(|options| MountBindOptions {
              propagation: options.propagation.map(|propogation| match propogation {
                bollard::secret::MountBindOptionsPropagationEnum::EMPTY => MountBindOptionsPropagationEnum::Empty,
                bollard::secret::MountBindOptionsPropagationEnum::PRIVATE => MountBindOptionsPropagationEnum::Private,
                bollard::secret::MountBindOptionsPropagationEnum::RPRIVATE => MountBindOptionsPropagationEnum::Rprivate,
                bollard::secret::MountBindOptionsPropagationEnum::SHARED => MountBindOptionsPropagationEnum::Shared,
                bollard::secret::MountBindOptionsPropagationEnum::RSHARED => MountBindOptionsPropagationEnum::Rshared,
                bollard::secret::MountBindOptionsPropagationEnum::SLAVE => MountBindOptionsPropagationEnum::Slave,
                bollard::secret::MountBindOptionsPropagationEnum::RSLAVE => MountBindOptionsPropagationEnum::Rslave,
              }).unwrap_or_default(),
              non_recursive: options.non_recursive,
              create_mountpoint: options.create_mountpoint,
              read_only_non_recursive: options.read_only_non_recursive,
              read_only_force_recursive: options.read_only_force_recursive,
            }),
            volume_options: mount.volume_options.map(|options| MountVolumeOptions {
                no_copy: options.no_copy,
                labels: options.labels.unwrap_or_default(),
                driver_config: options.driver_config.map(|config| MountVolumeOptionsDriverConfig {
                    name: config.name,
                    options: config.options.unwrap_or_default(),
                }),
                subpath: options.subpath,
            }),
            tmpfs_options: mount.tmpfs_options.map(|options| MountTmpfsOptions {
                size_bytes: options.size_bytes,
                mode: options.mode
            }),
        }).collect(),
        console_size: config.console_size.unwrap_or_default(),
        annotations: config.annotations.unwrap_or_default(),
        cap_add: config.cap_add.unwrap_or_default(),
        cap_drop: config.cap_drop.unwrap_or_default(),
        cgroupns_mode: config.cgroupns_mode.map(|mode| match mode {
          bollard::secret::HostConfigCgroupnsModeEnum::EMPTY => HostConfigCgroupnsModeEnum::Empty,
          bollard::secret::HostConfigCgroupnsModeEnum::PRIVATE => HostConfigCgroupnsModeEnum::Private,
          bollard::secret::HostConfigCgroupnsModeEnum::HOST => HostConfigCgroupnsModeEnum::Host,
        }),
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
        isolation: config.isolation.map(|isolation| match isolation {
          bollard::secret::HostConfigIsolationEnum::EMPTY => HostConfigIsolationEnum::Empty,
          bollard::secret::HostConfigIsolationEnum::DEFAULT => HostConfigIsolationEnum::Default,
          bollard::secret::HostConfigIsolationEnum::PROCESS => HostConfigIsolationEnum::Process,
          bollard::secret::HostConfigIsolationEnum::HYPERV => HostConfigIsolationEnum::Hyperv,
        }).unwrap_or_default(),
        masked_paths: config.masked_paths.unwrap_or_default(),
        readonly_paths: config.readonly_paths.unwrap_or_default(),
      }),
      graph_driver: container.graph_driver.map(|driver| GraphDriverData {
        name: driver.name,
        data: driver.data,
      }),
      size_rw: container.size_rw,
      size_root_fs: container.size_root_fs,
      mounts: container.mounts.unwrap_or_default().into_iter().map(|mount| MountPoint {
        typ: mount.typ.map(|typ| match typ {
          bollard::secret::MountPointTypeEnum::EMPTY => MountTypeEnum::Empty,
          bollard::secret::MountPointTypeEnum::BIND => MountTypeEnum::Bind,
          bollard::secret::MountPointTypeEnum::VOLUME => MountTypeEnum::Volume,
          bollard::secret::MountPointTypeEnum::TMPFS => MountTypeEnum::Tmpfs,
          bollard::secret::MountPointTypeEnum::NPIPE => MountTypeEnum::Npipe,
          bollard::secret::MountPointTypeEnum::CLUSTER => MountTypeEnum::Cluster,
        }).unwrap_or_default(),
        name: mount.name,
        source: mount.source,
        destination: mount.destination,
        driver: mount.driver,
        mode: mount.mode,
        rw: mount.rw,
        propagation: mount.propagation,
      }).collect(),
      config: container.config.map(|config| ContainerConfig {
        hostname: config.hostname,
        domainname: config.domainname,
        user: config.user,
        attach_stdin: config.attach_stdin,
        attach_stdout: config.attach_stdout,
        attach_stderr: config.attach_stderr,
        exposed_ports: config.exposed_ports.unwrap_or_default().into_keys().map(|k| (k, Default::default())).collect(),
        tty: config.tty,
        open_stdin: config.open_stdin,
        stdin_once: config.stdin_once,
        env: config.env.unwrap_or_default(),
        cmd: config.cmd.unwrap_or_default(),
        healthcheck: config.healthcheck.map(|health| HealthConfig {
          test: health.test.unwrap_or_default(),
          interval: health.interval,
          timeout: health.timeout,
          retries: health.retries,
          start_period: health.start_period,
          start_interval: health.start_interval,
        }),
        args_escaped: config.args_escaped,
        image: config.image,
        volumes: config.volumes.unwrap_or_default().into_keys().map(|k| (k, Default::default())).collect(),
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
      network_settings: container.network_settings.map(|settings| NetworkSettings {
        bridge: settings.bridge,
        sandbox_id: settings.sandbox_id,
        ports: settings
          .ports
          .unwrap_or_default()
          .into_iter()
          .map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|v| PortBinding { host_ip: v.host_ip, host_port: v.host_port }).collect())).collect(),
        sandbox_key: settings.sandbox_key,
        networks: settings.networks
        .unwrap_or_default().into_iter()
        .map(|(k, v)| (k, EndpointSettings {
          ipam_config: v.ipam_config.map(|ipam| EndpointIpamConfig {
            ipv4_address: ipam.ipv4_address,
            ipv6_address: ipam.ipv6_address,
            link_local_ips: ipam.link_local_ips.unwrap_or_default(),
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
          dns_names: v.dns_names.unwrap_or_default()
        })).collect(),
    }),
    })
  }

  pub async fn list_networks(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<NetworkListItem>> {
    let networks = self
      .docker
      .list_networks::<String>(None)
      .await?
      .into_iter()
      .map(|network| {
        let (ipam_driver, ipam_subnet, ipam_gateway) =
          if let Some(ipam) = network.ipam {
            let (subnet, gateway) = if let Some(config) = ipam
              .config
              .and_then(|configs| configs.into_iter().next())
            {
              (config.subnet, config.gateway)
            } else {
              (None, None)
            };
            (ipam.driver, subnet, gateway)
          } else {
            (None, None, None)
          };
        let in_use = match &network.name {
          Some(name) => containers.iter().any(|container| {
            container.networks.iter().any(|_name| name == _name)
          }),
          None => false,
        };
        NetworkListItem {
          name: network.name,
          id: network.id,
          created: network.created,
          scope: network.scope,
          driver: network.driver,
          enable_ipv6: network.enable_ipv6,
          ipam_driver,
          ipam_subnet,
          ipam_gateway,
          internal: network.internal,
          attachable: network.attachable,
          ingress: network.ingress,
          in_use,
        }
      })
      .collect();
    Ok(networks)
  }

  pub async fn inspect_network(
    &self,
    network_name: &str,
  ) -> anyhow::Result<Network> {
    let network = self
      .docker
      .inspect_network::<String>(
        network_name,
        InspectNetworkOptions {
          verbose: true,
          ..Default::default()
        }
        .into(),
      )
      .await?;
    Ok(Network {
      name: network.name,
      id: network.id,
      created: network.created,
      scope: network.scope,
      driver: network.driver,
      enable_ipv6: network.enable_ipv6,
      ipam: network.ipam.map(|ipam| Ipam {
        driver: ipam.driver,
        config: ipam
          .config
          .unwrap_or_default()
          .into_iter()
          .map(|config| IpamConfig {
            subnet: config.subnet,
            ip_range: config.ip_range,
            gateway: config.gateway,
            auxiliary_addresses: config
              .auxiliary_addresses
              .unwrap_or_default(),
          })
          .collect(),
        options: ipam.options.unwrap_or_default(),
      }),
      internal: network.internal,
      attachable: network.attachable,
      ingress: network.ingress,
      containers: network
        .containers
        .unwrap_or_default()
        .into_iter()
        .map(|(container_id, container)| NetworkContainer {
          container_id,
          name: container.name,
          endpoint_id: container.endpoint_id,
          mac_address: container.mac_address,
          ipv4_address: container.ipv4_address,
          ipv6_address: container.ipv6_address,
        })
        .collect(),
      options: network.options.unwrap_or_default(),
      labels: network.labels.unwrap_or_default(),
    })
  }

  pub async fn list_images(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<ImageListItem>> {
    let images = self
      .docker
      .list_images::<String>(None)
      .await?
      .into_iter()
      .map(|image| {
        let in_use = containers.iter().any(|container| {
          container
            .image_id
            .as_ref()
            .map(|id| id == &image.id)
            .unwrap_or_default()
        });
        ImageListItem {
          name: image
            .repo_tags
            .into_iter()
            .next()
            .unwrap_or_else(|| image.id.clone()),
          id: image.id,
          parent_id: image.parent_id,
          created: image.created,
          size: image.size,
          in_use,
        }
      })
      .collect();
    Ok(images)
  }

  pub async fn inspect_image(
    &self,
    image_name: &str,
  ) -> anyhow::Result<Image> {
    let image = self.docker.inspect_image(image_name).await?;
    Ok(Image {
      id: image.id,
      repo_tags: image.repo_tags.unwrap_or_default(),
      repo_digests: image.repo_digests.unwrap_or_default(),
      parent: image.parent,
      comment: image.comment,
      created: image.created,
      docker_version: image.docker_version,
      author: image.author,
      architecture: image.architecture,
      variant: image.variant,
      os: image.os,
      os_version: image.os_version,
      size: image.size,
      graph_driver: image.graph_driver.map(|driver| {
        GraphDriverData {
          name: driver.name,
          data: driver.data,
        }
      }),
      root_fs: image.root_fs.map(|fs| ImageInspectRootFs {
        typ: fs.typ,
        layers: fs.layers.unwrap_or_default(),
      }),
      metadata: image.metadata.map(|metadata| ImageInspectMetadata {
        last_tag_time: metadata.last_tag_time,
      }),
      config: image.config.map(|config| ContainerConfig {
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
        healthcheck: config.healthcheck.map(|health| HealthConfig {
          test: health.test.unwrap_or_default(),
          interval: health.interval,
          timeout: health.timeout,
          retries: health.retries,
          start_period: health.start_period,
          start_interval: health.start_interval,
        }),
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
    })
  }

  pub async fn image_history(
    &self,
    image_name: &str,
  ) -> anyhow::Result<Vec<ImageHistoryResponseItem>> {
    let res = self
      .docker
      .image_history(image_name)
      .await?
      .into_iter()
      .map(|image| ImageHistoryResponseItem {
        id: image.id,
        created: image.created,
        created_by: image.created_by,
        tags: image.tags,
        size: image.size,
        comment: image.comment,
      })
      .collect();
    Ok(res)
  }

  pub async fn list_volumes(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<VolumeListItem>> {
    let volumes = self
      .docker
      .list_volumes::<String>(None)
      .await?
      .volumes
      .unwrap_or_default()
      .into_iter()
      .map(|volume| {
        let scope = volume
          .scope
          .map(|scope| match scope {
            bollard::secret::VolumeScopeEnum::EMPTY => {
              VolumeScopeEnum::Empty
            }
            bollard::secret::VolumeScopeEnum::LOCAL => {
              VolumeScopeEnum::Local
            }
            bollard::secret::VolumeScopeEnum::GLOBAL => {
              VolumeScopeEnum::Global
            }
          })
          .unwrap_or(VolumeScopeEnum::Empty);
        let in_use = containers.iter().any(|container| {
          container.volumes.iter().any(|name| &volume.name == name)
        });
        VolumeListItem {
          name: volume.name,
          driver: volume.driver,
          mountpoint: volume.mountpoint,
          created: volume.created_at,
          size: volume.usage_data.map(|data| data.size),
          scope,
          in_use,
        }
      })
      .collect();
    Ok(volumes)
  }

  pub async fn inspect_volume(
    &self,
    volume_name: &str,
  ) -> anyhow::Result<Volume> {
    let volume = self.docker.inspect_volume(volume_name).await?;
    Ok(Volume {
      name: volume.name,
      driver: volume.driver,
      mountpoint: volume.mountpoint,
      created_at: volume.created_at,
      status: volume.status.unwrap_or_default().into_keys().map(|k| (k, Default::default())).collect(),
      labels: volume.labels,
      scope: volume
        .scope
        .map(|scope| match scope {
          bollard::secret::VolumeScopeEnum::EMPTY => {
            VolumeScopeEnum::Empty
          }
          bollard::secret::VolumeScopeEnum::LOCAL => {
            VolumeScopeEnum::Local
          }
          bollard::secret::VolumeScopeEnum::GLOBAL => {
            VolumeScopeEnum::Global
          }
        })
        .unwrap_or_default(),
      cluster_volume: volume.cluster_volume.map(|volume| {
        ClusterVolume {
          id: volume.id,
          version: volume.version.map(|version| ObjectVersion {
            index: version.index,
          }),
          created_at: volume.created_at,
          updated_at: volume.updated_at,
          spec: volume.spec.map(|spec| ClusterVolumeSpec {
            group: spec.group,
            access_mode: spec.access_mode.map(|mode| {
              ClusterVolumeSpecAccessMode {
                scope: mode.scope.map(|scope| match scope {
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::EMPTY => ClusterVolumeSpecAccessModeScopeEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::SINGLE => ClusterVolumeSpecAccessModeScopeEnum::Single,
                  bollard::secret::ClusterVolumeSpecAccessModeScopeEnum::MULTI => ClusterVolumeSpecAccessModeScopeEnum::Multi,
                }).unwrap_or_default(),
                sharing: mode.sharing.map(|sharing| match sharing {
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::EMPTY => ClusterVolumeSpecAccessModeSharingEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::NONE => ClusterVolumeSpecAccessModeSharingEnum::None,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::READONLY => ClusterVolumeSpecAccessModeSharingEnum::Readonly,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::ONEWRITER => ClusterVolumeSpecAccessModeSharingEnum::Onewriter,
                  bollard::secret::ClusterVolumeSpecAccessModeSharingEnum::ALL => ClusterVolumeSpecAccessModeSharingEnum::All,
                }).unwrap_or_default(),
                secrets: mode.secrets.unwrap_or_default().into_iter().map(|secret| ClusterVolumeSpecAccessModeSecrets {
                    key: secret.key,
                    secret: secret.secret,
                }).collect(),
                accessibility_requirements: mode
                  .accessibility_requirements.map(|req| ClusterVolumeSpecAccessModeAccessibilityRequirements {
                    requisite: req.requisite.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
                    preferred: req.preferred.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
                }),
                capacity_range: mode.capacity_range.map(|range| ClusterVolumeSpecAccessModeCapacityRange {
                  required_bytes: range.required_bytes,
                  limit_bytes: range.limit_bytes,
                }),
                availability: mode.availability.map(|availability| match availability {
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::EMPTY => ClusterVolumeSpecAccessModeAvailabilityEnum::Empty,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::ACTIVE => ClusterVolumeSpecAccessModeAvailabilityEnum::Active,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::PAUSE => ClusterVolumeSpecAccessModeAvailabilityEnum::Pause,
                  bollard::secret::ClusterVolumeSpecAccessModeAvailabilityEnum::DRAIN => ClusterVolumeSpecAccessModeAvailabilityEnum::Drain,
                }).unwrap_or_default(),
              }
            }),
          }),
          info: volume.info.map(|info| ClusterVolumeInfo {
            capacity_bytes: info.capacity_bytes,
            volume_context: info.volume_context.unwrap_or_default(),
            volume_id: info.volume_id,
            accessible_topology: info.accessible_topology.unwrap_or_default().into_iter().map(|map| map.into_iter().map(|(k, v)| (k, v.unwrap_or_default().into_iter().map(|p| PortBinding { host_ip: p.host_ip, host_port: p.host_port }).collect())).collect()).collect(),
          }),
          publish_status: volume
            .publish_status
            .unwrap_or_default()
            .into_iter()
            .map(|status| ClusterVolumePublishStatus {
              node_id: status.node_id,
              state: status.state.map(|state| match state {
                bollard::secret::ClusterVolumePublishStatusStateEnum::EMPTY => ClusterVolumePublishStatusStateEnum::Empty,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_PUBLISH => ClusterVolumePublishStatusStateEnum::PendingPublish,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PUBLISHED => ClusterVolumePublishStatusStateEnum::Published,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_NODE_UNPUBLISH => ClusterVolumePublishStatusStateEnum::PendingNodeUnpublish,
                bollard::secret::ClusterVolumePublishStatusStateEnum::PENDING_CONTROLLER_UNPUBLISH => ClusterVolumePublishStatusStateEnum::PendingControllerUnpublish,
              }).unwrap_or_default(),
              publish_context: status.publish_context.unwrap_or_default(),
            })
            .collect(),
        }
      }),
      options: volume.options,
      usage_data: volume.usage_data.map(|data| VolumeUsageData {
        size: data.size,
        ref_count: data.ref_count,
      }),
    })
  }
}

/// Returns whether build result should be pushed after build
#[instrument(skip(registry_token))]
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
  let log = async_run_command(&format!(
    "echo {registry_token} | docker login {domain} --username '{account}' --password-stdin",
  ))
  .await;
  if log.success() {
    Ok(true)
  } else {
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
}

#[instrument]
pub async fn pull_image(image: &str) -> Log {
  let command = format!("docker pull {image}");
  run_komodo_command("Docker Pull", None, command).await
}

pub fn stop_container_command(
  container_name: &str,
  signal: Option<TerminationSignal>,
  time: Option<i32>,
) -> String {
  let container_name = to_docker_compatible_name(container_name);
  let signal = signal
    .map(|signal| format!(" --signal {signal}"))
    .unwrap_or_default();
  let time = time
    .map(|time| format!(" --time {time}"))
    .unwrap_or_default();
  format!("docker stop{signal}{time} {container_name}")
}

pub async fn container_stats(
  container_name: Option<String>,
) -> anyhow::Result<Vec<ContainerStats>> {
  let format = "--format \"{{ json . }}\"";
  let container_name = match container_name {
    Some(name) => format!(" {name}"),
    None => "".to_string(),
  };
  let command =
    format!("docker stats{container_name} --no-stream {format}");
  let output = async_run_command(&command).await;
  if output.success() {
    output
      .stdout
      .split('\n')
      .filter(|e| !e.is_empty())
      .map(|e| {
        let parsed = serde_json::from_str(e)
          .context(format!("failed at parsing entry {e}"))?;
        Ok(parsed)
      })
      .collect()
  } else {
    Err(anyhow!("{}", output.stderr.replace('\n', "")))
  }
}
