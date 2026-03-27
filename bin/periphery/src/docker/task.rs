use anyhow::Context;
use bollard::query_parameters::ListTasksOptions;
use komodo_client::entities::docker::task::*;

use super::*;

impl DockerClient {
  pub async fn list_swarm_tasks(
    &self,
  ) -> anyhow::Result<Vec<SwarmTaskListItem>> {
    let mut tasks = self
      .docker
      .list_tasks(Option::<ListTasksOptions>::None)
      .await
      .context("Failed to query for swarm tasks list")?
      .into_iter()
      .map(convert_task_list_item)
      .collect::<Vec<_>>();

    tasks.sort_by(|a, b| {
      a.state.cmp(&b.state).then_with(|| a.name.cmp(&b.name))
    });

    Ok(tasks)
  }

  pub async fn inspect_swarm_task(
    &self,
    task_id: &str,
  ) -> anyhow::Result<SwarmTask> {
    self
      .docker
      .inspect_task(task_id)
      .await
      .map(convert_task)
      .with_context(|| {
        format!("Failed to query for swarm task with id {task_id}")
      })
  }
}

fn convert_task_list_item(
  task: bollard::models::Task,
) -> SwarmTaskListItem {
  let (container_id, state) = task
    .status
    .map(|status| {
      (
        status
          .container_status
          .and_then(|status| status.container_id),
        status.state.map(convert_task_state),
      )
    })
    .unwrap_or_default();
  let (configs, secrets) = task
    .spec
    .and_then(|spec| {
      spec.container_spec.map(|spec| {
        (
          spec
            .configs
            .map(|config| {
              config
                .into_iter()
                .filter_map(|config| config.config_name)
                .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
          spec
            .secrets
            .map(|secret| {
              secret
                .into_iter()
                .filter_map(|secret| secret.secret_name)
                .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        )
      })
    })
    .unwrap_or_default();
  SwarmTaskListItem {
    id: task.id,
    name: task.name,
    node_id: task.node_id,
    service_id: task.service_id,
    container_id,
    state,
    desired_state: task.desired_state.map(convert_task_state),
    configs,
    secrets,
    created_at: task.created_at,
    updated_at: task.updated_at,
  }
}

fn convert_task(task: bollard::models::Task) -> SwarmTask {
  SwarmTask {
    id: task.id,
    version: task.version.map(convert_object_version),
    created_at: task.created_at,
    updated_at: task.updated_at,
    name: task.name,
    labels: task.labels,
    spec: task.spec.map(convert_task_spec),
    service_id: task.service_id,
    slot: task.slot,
    node_id: task.node_id,
    assigned_generic_resources: task
      .assigned_generic_resources
      .map(convert_generic_resources),
    status: task.status.map(|status| TaskStatus {
      timestamp: status.timestamp,
      state: status.state.map(convert_task_state),
      message: status.message,
      err: status.err,
      container_status: status.container_status.map(|status| {
        ContainerStatus {
          container_id: status.container_id,
          pid: status.pid,
          exit_code: status.exit_code,
        }
      }),
      port_status: status.port_status.map(|status| PortStatus {
        ports: status.ports.map(convert_endpoint_spec_ports),
      }),
    }),
    desired_state: task.desired_state.map(convert_task_state),
    job_iteration: task.job_iteration.map(convert_object_version),
  }
}

fn convert_task_state(
  state: bollard::models::TaskState,
) -> TaskState {
  match state {
    bollard::config::TaskState::NEW => TaskState::NEW,
    bollard::config::TaskState::ALLOCATED => TaskState::ALLOCATED,
    bollard::config::TaskState::PENDING => TaskState::PENDING,
    bollard::config::TaskState::ASSIGNED => TaskState::ASSIGNED,
    bollard::config::TaskState::ACCEPTED => TaskState::ACCEPTED,
    bollard::config::TaskState::PREPARING => TaskState::PREPARING,
    bollard::config::TaskState::READY => TaskState::READY,
    bollard::config::TaskState::STARTING => TaskState::STARTING,
    bollard::config::TaskState::RUNNING => TaskState::RUNNING,
    bollard::config::TaskState::COMPLETE => TaskState::COMPLETE,
    bollard::config::TaskState::SHUTDOWN => TaskState::SHUTDOWN,
    bollard::config::TaskState::FAILED => TaskState::FAILED,
    bollard::config::TaskState::REJECTED => TaskState::REJECTED,
    bollard::config::TaskState::REMOVE => TaskState::REMOVE,
    bollard::config::TaskState::ORPHANED => TaskState::ORPHANED,
  }
}
