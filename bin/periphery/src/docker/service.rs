use anyhow::Context;
use bollard::query_parameters::{
  InspectServiceOptions, ListServicesOptionsBuilder,
};
use futures_util::{TryStreamExt as _, stream::FuturesUnordered};
use komodo_client::entities::{
  NoData,
  docker::{NetworkAttachmentConfig, service::*},
  swarm::SwarmState,
};

use super::*;

impl DockerClient {
  /// List swarm services
  pub async fn list_swarm_services(
    &self,
  ) -> anyhow::Result<Vec<SwarmServiceListItem>> {
    let mut services = self
      .docker
      .list_services(
        ListServicesOptionsBuilder::new()
          .status(true)
          .build()
          .into(),
      )
      .await
      .context("Failed to query for swarm service list")?
      .into_iter()
      .map(convert_service_list_item)
      .collect::<Vec<_>>();

    services.sort_by(|a, b| {
      a.name
        .cmp(&b.name)
        .then_with(|| b.updated_at.cmp(&a.updated_at))
    });

    Ok(services)
  }

  pub async fn filter_map_swarm_services<T>(
    &self,
    filter_map: impl Fn(SwarmService) -> Option<T>,
  ) -> anyhow::Result<Vec<T>> {
    let res = self
      .list_swarm_services()
      .await?
      .into_iter()
      .map(|service| async {
        let Some(name) = service.name else {
          return Ok(None);
        };
        let service = self.inspect_swarm_service(&name).await?;
        anyhow::Ok(filter_map(service))
      })
      .collect::<FuturesUnordered<_>>()
      .try_collect::<Vec<_>>()
      .await?
      .into_iter()
      .flatten()
      .collect::<Vec<_>>();
    Ok(res)
  }

  pub async fn inspect_swarm_service(
    &self,
    service_name: &str,
  ) -> anyhow::Result<SwarmService> {
    self
      .docker
      .inspect_service(
        service_name,
        Some(InspectServiceOptions {
          insert_defaults: true,
        }),
      )
      .await
      .map(convert_service)
      .with_context(|| {
        format!(
          "Failed to query for swarm service with name {service_name}"
        )
      })
  }
}

fn convert_service_list_item(
  service: bollard::models::Service,
) -> SwarmServiceListItem {
  let (
    name,
    ((image, configs, secrets), restart, runtime),
    (mode, replicas, max_concurrent),
  ) = if let Some(spec) = service.spec {
    let task_template = if let Some(template) = spec.task_template {
      let container_spec = if let Some(spec) = template.container_spec
      {
        let configs = if let Some(configs) = spec.configs {
          configs
            .into_iter()
            .filter_map(|config| config.config_name)
            .collect::<Vec<_>>()
        } else {
          Default::default()
        };
        let secrets = if let Some(secrets) = spec.secrets {
          secrets
            .into_iter()
            .filter_map(|secret| secret.secret_name)
            .collect::<Vec<_>>()
        } else {
          Default::default()
        };
        (spec.image, configs, secrets)
      } else {
        Default::default()
      };
      (
        container_spec,
        template.restart_policy.and_then(|policy| {
          policy
            .condition
            .map(convert_task_spec_restart_policy_condition)
        }),
        template.runtime,
      )
    } else {
      Default::default()
    };
    (
      spec.name,
      task_template,
      extract_mode_replicas_concurrent(spec.mode.as_ref()),
    )
  } else {
    Default::default()
  };

  let state = extract_state_from_service_status(
    mode.unwrap_or(SwarmServiceMode::Replicated),
    replicas,
    max_concurrent,
    service.service_status.as_ref(),
  );

  let (running_tasks, desired_tasks, completed_tasks) =
    if let Some(status) = service.service_status {
      (
        status.running_tasks,
        status.desired_tasks,
        status.completed_tasks,
      )
    } else {
      Default::default()
    };

  SwarmServiceListItem {
    id: service.id,
    name,
    image,
    restart,
    runtime,
    configs,
    secrets,
    mode,
    replicas,
    max_concurrent,
    running_tasks,
    desired_tasks,
    completed_tasks,
    state,
    created_at: service.created_at,
    updated_at: service.updated_at,
  }
}

fn extract_mode_replicas_concurrent(
  mode: Option<&bollard::models::ServiceSpecMode>,
) -> (Option<SwarmServiceMode>, Option<i64>, Option<i64>) {
  let Some(mode) = mode else {
    return Default::default();
  };
  if let Some(replicated) = &mode.replicated {
    (
      Some(SwarmServiceMode::Replicated),
      replicated.replicas,
      None,
    )
  } else if let Some(replicated_job) = &mode.replicated_job {
    (
      Some(SwarmServiceMode::ReplicatedJob),
      replicated_job.total_completions,
      replicated_job.max_concurrent,
    )
  } else if mode.global.is_some() {
    (Some(SwarmServiceMode::Global), None, None)
  } else if mode.global_job.is_some() {
    (Some(SwarmServiceMode::GlobalJob), None, None)
  } else {
    Default::default()
  }
}

fn extract_state_from_service_status(
  mode: SwarmServiceMode,
  replicas: Option<i64>,
  max_concurrent: Option<i64>,
  status: Option<&bollard::models::ServiceServiceStatus>,
) -> SwarmState {
  let Some(status) = status else {
    return Default::default();
  };
  let (running, desired, completed) = (
    status.running_tasks.unwrap_or_default(),
    status.desired_tasks.unwrap_or_default(),
    status.completed_tasks.unwrap_or_default(),
  );
  match mode {
    SwarmServiceMode::Global | SwarmServiceMode::Replicated => {
      if desired == 0 {
        if running == 0 {
          SwarmState::Down
        } else {
          SwarmState::Unhealthy
        }
      } else if desired > running {
        SwarmState::Unhealthy
      } else {
        SwarmState::Healthy
      }
    }
    // Job mode
    SwarmServiceMode::GlobalJob => {
      if desired == completed {
        if running == 0 {
          SwarmState::Down
        } else {
          SwarmState::Unhealthy
        }
      } else {
        if running > 0 {
          SwarmState::Healthy
        } else {
          SwarmState::Unhealthy
        }
      }
    }
    SwarmServiceMode::ReplicatedJob => {
      if let (Some(replicas), Some(max_concurrent)) =
        (replicas, max_concurrent.or(replicas))
      {
        if completed >= replicas as u64 {
          if running == 0 {
            SwarmState::Down
          } else {
            SwarmState::Unhealthy
          }
        } else {
          if running == max_concurrent as u64
            // It may be just finishing up the last ones
            || desired - completed == running
          {
            SwarmState::Healthy
          } else {
            SwarmState::Unhealthy
          }
        }
      } else {
        // Do the best we can using global method
        extract_state_from_service_status(
          SwarmServiceMode::GlobalJob,
          None,
          None,
          Some(status),
        )
      }
    }
  }
}

fn convert_service(
  service: bollard::models::Service,
) -> SwarmService {
  let (mode, replicas, max_concurrent) = service
    .spec
    .as_ref()
    .map(|spec| extract_mode_replicas_concurrent(spec.mode.as_ref()))
    .unwrap_or_default();

  let state = extract_state_from_service_status(
    mode.unwrap_or(SwarmServiceMode::Replicated),
    replicas,
    max_concurrent,
    service.service_status.as_ref(),
  );

  SwarmService {
    id: service.id,
    version: service.version.map(convert_object_version),
    mode,
    replicas,
    max_concurrent,
    state,
    created_at: service.created_at,
    updated_at: service.updated_at,
    spec: service.spec.map(|spec| ServiceSpec {
      name: spec.name,
      labels: spec.labels,
      task_template: spec.task_template.map(convert_task_spec),
      mode: spec.mode.map(|mode| ServiceSpecMode {
        replicated: mode.replicated.map(|replicated| ServiceSpecModeReplicated {
          replicas: replicated.replicas,
        }),
        global: mode.global.map(|_| NoData {}),
        replicated_job: mode.replicated_job.map(|job| ServiceSpecModeReplicatedJob {
          max_concurrent: job.max_concurrent,
          total_completions: job.total_completions,
        }),
        global_job: mode.global_job.map(|_| NoData {}),
      }),
      update_config: spec.update_config.map(|config| {
        ServiceSpecUpdateConfig {
          parallelism: config.parallelism,
          delay: config.delay,
          failure_action: config.failure_action.map(|action| match action {
            bollard::secret::ServiceSpecUpdateConfigFailureActionEnum::EMPTY => ServiceSpecUpdateConfigFailureActionEnum::EMPTY,
            bollard::secret::ServiceSpecUpdateConfigFailureActionEnum::CONTINUE => ServiceSpecUpdateConfigFailureActionEnum::CONTINUE,
            bollard::secret::ServiceSpecUpdateConfigFailureActionEnum::PAUSE => ServiceSpecUpdateConfigFailureActionEnum::PAUSE,
            bollard::secret::ServiceSpecUpdateConfigFailureActionEnum::ROLLBACK => ServiceSpecUpdateConfigFailureActionEnum::ROLLBACK,
          }),
          monitor: config.monitor,
          max_failure_ratio: config.max_failure_ratio,
          order: config.order.map(|order| match order {
            bollard::secret::ServiceSpecUpdateConfigOrderEnum::EMPTY => ServiceSpecUpdateConfigOrderEnum::EMPTY,
            bollard::secret::ServiceSpecUpdateConfigOrderEnum::STOP_FIRST => ServiceSpecUpdateConfigOrderEnum::STOP_FIRST,
            bollard::secret::ServiceSpecUpdateConfigOrderEnum::START_FIRST => ServiceSpecUpdateConfigOrderEnum::START_FIRST,
          }),
        }
      }),
      rollback_config: spec.rollback_config.map(|config| {
        ServiceSpecRollbackConfig {
          parallelism: config.parallelism,
          delay: config.delay,
          failure_action: config.failure_action.map(|action| match action {
            bollard::secret::ServiceSpecRollbackConfigFailureActionEnum::EMPTY => ServiceSpecRollbackConfigFailureActionEnum::EMPTY,
            bollard::secret::ServiceSpecRollbackConfigFailureActionEnum::CONTINUE => ServiceSpecRollbackConfigFailureActionEnum::CONTINUE,
            bollard::secret::ServiceSpecRollbackConfigFailureActionEnum::PAUSE => ServiceSpecRollbackConfigFailureActionEnum::PAUSE,
          }),
          monitor: config.monitor,
          max_failure_ratio: config.max_failure_ratio,
          order: config.order.map(|order| match order {
            bollard::secret::ServiceSpecRollbackConfigOrderEnum::EMPTY => ServiceSpecRollbackConfigOrderEnum::EMPTY,
            bollard::secret::ServiceSpecRollbackConfigOrderEnum::STOP_FIRST => ServiceSpecRollbackConfigOrderEnum::STOP_FIRST,
            bollard::secret::ServiceSpecRollbackConfigOrderEnum::START_FIRST => ServiceSpecRollbackConfigOrderEnum::START_FIRST,
          }),
        }
      }),
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
      endpoint_spec: spec.endpoint_spec.map(convert_endpoint_spec),
    }),
    endpoint: service.endpoint.map(|endpoint| ServiceEndpoint {
      spec: endpoint.spec.map(convert_endpoint_spec),
      ports: endpoint.ports.map(convert_endpoint_spec_ports),
      virtual_ips: endpoint.virtual_ips.map(|ips| {
        ips
          .into_iter()
          .map(|ip| ServiceEndpointVirtualIps {
            network_id: ip.network_id,
            addr: ip.addr,
          })
          .collect()
      }),
    }),
    update_status: service.update_status.map(|status| {
      ServiceUpdateStatus {
        state: status.state.map(convert_state),
        started_at: status.started_at,
        completed_at: status.completed_at,
        message: status.message,
      }
    }),
    service_status: service.service_status.map(|status| {
      ServiceServiceStatus {
        running_tasks: status.running_tasks,
        desired_tasks: status.desired_tasks,
        completed_tasks: status.completed_tasks,
      }
    }),
    job_status: service.job_status.map(|status| ServiceJobStatus {
      job_iteration: status.job_iteration.map(convert_object_version),
      last_execution: status.last_execution,
    }),
  }
}

fn convert_endpoint_spec(
  spec: bollard::models::EndpointSpec,
) -> EndpointSpec {
  EndpointSpec {
    mode: spec.mode.map(|mode| match mode {
      bollard::secret::EndpointSpecModeEnum::EMPTY => {
        EndpointSpecModeEnum::EMPTY
      }
      bollard::secret::EndpointSpecModeEnum::VIP => {
        EndpointSpecModeEnum::VIP
      }
      bollard::secret::EndpointSpecModeEnum::DNSRR => {
        EndpointSpecModeEnum::DNSRR
      }
    }),
    ports: spec.ports.map(convert_endpoint_spec_ports),
  }
}

fn convert_state(
  state: bollard::secret::ServiceUpdateStatusStateEnum,
) -> ServiceUpdateStatusStateEnum {
  match state {
    bollard::secret::ServiceUpdateStatusStateEnum::EMPTY => ServiceUpdateStatusStateEnum::EMPTY,
    bollard::secret::ServiceUpdateStatusStateEnum::UPDATING => ServiceUpdateStatusStateEnum::UPDATING,
    bollard::secret::ServiceUpdateStatusStateEnum::PAUSED => ServiceUpdateStatusStateEnum::PAUSED,
    bollard::secret::ServiceUpdateStatusStateEnum::COMPLETED => ServiceUpdateStatusStateEnum::COMPLETED,
    bollard::secret::ServiceUpdateStatusStateEnum::ROLLBACK_STARTED => ServiceUpdateStatusStateEnum::ROLLBACK_STARTED,
    bollard::secret::ServiceUpdateStatusStateEnum::ROLLBACK_PAUSED => ServiceUpdateStatusStateEnum::ROLLBACK_PAUSED,
    bollard::secret::ServiceUpdateStatusStateEnum::ROLLBACK_COMPLETED => ServiceUpdateStatusStateEnum::ROLLBACK_COMPLETED,
  }
}
