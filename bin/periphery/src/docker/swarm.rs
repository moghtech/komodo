use anyhow::Context;
use komodo_client::entities::docker::swarm::{
  JoinTokens, SwarmInspectInfo, SwarmSpec, SwarmSpecCaConfig,
  SwarmSpecCaConfigExternalCas,
  SwarmSpecCaConfigExternalCasProtocolEnum, SwarmSpecDispatcher,
  SwarmSpecEncryptionConfig, SwarmSpecOrchestration, SwarmSpecRaft,
  SwarmSpecTaskDefaults, SwarmSpecTaskDefaultsLogDriver,
};

use super::DockerClient;

impl DockerClient {
  /// Inspect swarm info
  pub async fn inspect_swarm(
    &self,
  ) -> anyhow::Result<SwarmInspectInfo> {
    self
      .docker
      .inspect_swarm()
      .await
      .map(convert_swarm_info)
      .context("Failed to query for swarm info")
  }
}

fn convert_swarm_info(
  swarm: bollard::models::Swarm,
) -> SwarmInspectInfo {
  SwarmInspectInfo {
    id: swarm.id,
    version: swarm.version.map(super::convert_object_version),
    created_at: swarm.created_at,
    updated_at: swarm.updated_at,
    spec: swarm.spec.map(|spec| SwarmSpec {
      name: spec.name,
      labels: spec.labels,
      orchestration: spec.orchestration.map(|orchestration| {
        SwarmSpecOrchestration {
          task_history_retention_limit: orchestration
            .task_history_retention_limit,
        }
      }),
      raft: spec.raft.map(|raft| SwarmSpecRaft {
        snapshot_interval: raft.snapshot_interval,
        keep_old_snapshots: raft.keep_old_snapshots,
        log_entries_for_slow_followers: raft
          .log_entries_for_slow_followers,
        election_tick: raft.election_tick,
        heartbeat_tick: raft.heartbeat_tick,
      }),
      dispatcher: spec.dispatcher.map(|dispatcher| {
        SwarmSpecDispatcher {
          heartbeat_period: dispatcher.heartbeat_period,
        }
      }),
      ca_config: spec.ca_config.map(|config| SwarmSpecCaConfig {
        node_cert_expiry: config.node_cert_expiry,
        external_cas: config.external_cas.map(|cas| {
          cas
            .into_iter()
            .map(|cas| SwarmSpecCaConfigExternalCas {
              protocol: cas.protocol.map(|protocol| match protocol {
                bollard::config::SwarmSpecCaConfigExternalCasProtocolEnum::EMPTY => SwarmSpecCaConfigExternalCasProtocolEnum::EMPTY,
                bollard::config::SwarmSpecCaConfigExternalCasProtocolEnum::CFSSL => SwarmSpecCaConfigExternalCasProtocolEnum::CFSSL,
              }),
              url: cas.url,
              options: cas.options,
              ca_cert: cas.ca_cert,
            })
            .collect()
        }),
        signing_ca_cert: config.signing_ca_cert,
        signing_ca_key: config.signing_ca_key,
        force_rotate: config.force_rotate,
      }),
      encryption_config: spec.encryption_config.map(|config| SwarmSpecEncryptionConfig {
        auto_lock_managers: config.auto_lock_managers,
    }),
      task_defaults: spec.task_defaults.map(|defaults| SwarmSpecTaskDefaults {
        log_driver: defaults.log_driver.map(|driver| SwarmSpecTaskDefaultsLogDriver {
          name: driver.name,
          options: driver.options,
        }),
    }),
    }),
    tls_info: swarm.tls_info.map(super::convert_tls_info),
    root_rotation_in_progress: swarm.root_rotation_in_progress,
    data_path_port: swarm.data_path_port,
    default_addr_pool: swarm.default_addr_pool,
    subnet_size: swarm.subnet_size,
    join_tokens: swarm.join_tokens.map(|tokens| JoinTokens {
      worker: tokens.worker,
      manager: tokens.manager,
    }),
  }
}
