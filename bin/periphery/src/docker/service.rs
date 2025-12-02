use anyhow::Context;
use bollard::query_parameters::{
  InspectServiceOptions, ListServicesOptions,
};
use komodo_client::entities::docker::{
  NetworkAttachmentConfig, service::*,
};

use super::*;

impl DockerClient {
  /// List swarm services
  pub async fn list_swarm_services(
    &self,
  ) -> anyhow::Result<Vec<SwarmServiceListItem>> {
    let mut services = self
      .docker
      .list_services(Option::<ListServicesOptions>::None)
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
  let (name, (image, restart, runtime), replicas) = service
    .spec
    .map(|spec| {
      (
        spec.name,
        spec
          .task_template
          .map(|template| {
            (
              template.container_spec.and_then(|spec| spec.image),
              template.restart_policy.and_then(|policy| {
                policy
                  .condition
                  .map(convert_task_spec_restart_policy_condition)
              }),
              template.runtime,
            )
          })
          .unwrap_or_default(),
        spec.mode.and_then(|mode| {
          mode.replicated.and_then(|replicated| replicated.replicas)
        }),
      )
    })
    .unwrap_or_default();
  SwarmServiceListItem {
    id: service.id,
    name,
    replicas,
    image,
    restart,
    runtime,
    created_at: service.created_at,
    updated_at: service.updated_at,
  }
}

fn convert_service(
  service: bollard::models::Service,
) -> SwarmService {
  SwarmService {
    id: service.id,
    version: service.version.map(convert_object_version),
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
        // global: mode.global,
        replicated_job: mode.replicated_job.map(|job| ServiceSpecModeReplicatedJob {
          max_concurrent: job.max_concurrent,
          total_completions: job.total_completions,
        }),
        // global_job: mode.global_job,
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
