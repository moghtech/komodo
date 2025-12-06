use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  I64, U64, docker::task::TaskSpecRestartPolicyConditionEnum,
  swarm::SwarmState,
};

use super::{
  EndpointPortConfig, NetworkAttachmentConfig, ObjectVersion,
  task::TaskSpec,
};

/// Swarm service list item.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmServiceListItem {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Name of the service.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// The image associated with service
  #[serde(rename = "Image")]
  pub image: Option<String>,

  /// Runtime is the type of runtime specified for the task executor.
  #[serde(rename = "Runtime")]
  pub runtime: Option<String>,

  /// Condition for restart.
  #[serde(rename = "Restart")]
  pub restart: Option<TaskSpecRestartPolicyConditionEnum>,

  /// Number of replicas
  #[serde(rename = "Replicas")]
  pub replicas: Option<I64>,

  /// Swarm service state.
  /// - Healthy if all associated tasks match their desired state (or report no desired state)
  /// - Unhealthy otherwise
  ///
  /// Not included in docker cli return, computed by Komodo
  #[serde(rename = "State")]
  pub state: SwarmState,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,
}

/// Swarm service details.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SwarmService {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<ServiceSpec>,

  #[serde(rename = "Endpoint")]
  pub endpoint: Option<ServiceEndpoint>,

  #[serde(rename = "UpdateStatus")]
  pub update_status: Option<ServiceUpdateStatus>,

  #[serde(rename = "ServiceStatus")]
  pub service_status: Option<ServiceServiceStatus>,

  #[serde(rename = "JobStatus")]
  pub job_status: Option<ServiceJobStatus>,
}

/// User modifiable configuration for a service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpec {
  /// Name of the service.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  #[serde(rename = "TaskTemplate")]
  pub task_template: Option<TaskSpec>,

  #[serde(rename = "Mode")]
  pub mode: Option<ServiceSpecMode>,

  #[serde(rename = "UpdateConfig")]
  pub update_config: Option<ServiceSpecUpdateConfig>,

  #[serde(rename = "RollbackConfig")]
  pub rollback_config: Option<ServiceSpecRollbackConfig>,

  /// Specifies which networks the service should attach to.  Deprecated: This field is deprecated since v1.44. The Networks field in TaskSpec should be used instead.
  #[serde(rename = "Networks")]
  pub networks: Option<Vec<NetworkAttachmentConfig>>,

  #[serde(rename = "EndpointSpec")]
  pub endpoint_spec: Option<EndpointSpec>,
}

/// Scheduling mode for the service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpecMode {
  #[serde(rename = "Replicated")]
  pub replicated: Option<ServiceSpecModeReplicated>,

  // #[serde(rename = "Global")]
  // pub global: Option<HashMap<(), ()>>,
  #[serde(rename = "ReplicatedJob")]
  pub replicated_job: Option<ServiceSpecModeReplicatedJob>,
  // /// The mode used for services which run a task to the completed state on each valid node.
  // #[serde(rename = "GlobalJob")]
  // pub global_job: Option<HashMap<(), ()>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpecModeReplicated {
  #[serde(rename = "Replicas")]
  pub replicas: Option<I64>,
}

/// The mode used for services with a finite number of tasks that run to a completed state.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpecModeReplicatedJob {
  /// The maximum number of replicas to run simultaneously.
  #[serde(rename = "MaxConcurrent")]
  pub max_concurrent: Option<I64>,

  /// The total number of replicas desired to reach the Completed state. If unset, will default to the value of `MaxConcurrent`
  #[serde(rename = "TotalCompletions")]
  pub total_completions: Option<I64>,
}

/// Specification for the update strategy of the service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpecUpdateConfig {
  /// Maximum number of tasks to be updated in one iteration (0 means unlimited parallelism).
  #[serde(rename = "Parallelism")]
  pub parallelism: Option<I64>,

  /// Amount of time between updates, in nanoseconds.
  #[serde(rename = "Delay")]
  pub delay: Option<I64>,

  /// Action to take if an updated task fails to run, or stops running during the update.
  #[serde(rename = "FailureAction")]
  pub failure_action:
    Option<ServiceSpecUpdateConfigFailureActionEnum>,

  /// Amount of time to monitor each updated task for failures, in nanoseconds.
  #[serde(rename = "Monitor")]
  pub monitor: Option<I64>,

  /// The fraction of tasks that may fail during an update before the failure action is invoked, specified as a floating point number between 0 and 1.
  #[serde(rename = "MaxFailureRatio")]
  pub max_failure_ratio: Option<f64>,

  /// The order of operations when rolling out an updated task. Either the old task is shut down before the new task is started, or the new task is started before the old task is shut down.
  #[serde(rename = "Order")]
  pub order: Option<ServiceSpecUpdateConfigOrderEnum>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum ServiceSpecUpdateConfigFailureActionEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "continue")]
  CONTINUE,
  #[serde(rename = "pause")]
  PAUSE,
  #[serde(rename = "rollback")]
  ROLLBACK,
}

#[allow(non_camel_case_types)]
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum ServiceSpecUpdateConfigOrderEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "stop-first")]
  STOP_FIRST,
  #[serde(rename = "start-first")]
  START_FIRST,
}

/// Specification for the rollback strategy of the service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceSpecRollbackConfig {
  /// Maximum number of tasks to be rolled back in one iteration (0 means unlimited parallelism).
  #[serde(rename = "Parallelism")]
  pub parallelism: Option<I64>,

  /// Amount of time between rollback iterations, in nanoseconds.
  #[serde(rename = "Delay")]
  pub delay: Option<I64>,

  /// Action to take if an rolled back task fails to run, or stops running during the rollback.
  #[serde(rename = "FailureAction")]
  pub failure_action:
    Option<ServiceSpecRollbackConfigFailureActionEnum>,

  /// Amount of time to monitor each rolled back task for failures, in nanoseconds.
  #[serde(rename = "Monitor")]
  pub monitor: Option<I64>,

  /// The fraction of tasks that may fail during a rollback before the failure action is invoked, specified as a floating point number between 0 and 1.
  #[serde(rename = "MaxFailureRatio")]
  pub max_failure_ratio: Option<f64>,

  /// The order of operations when rolling back a task. Either the old task is shut down before the new task is started, or the new task is started before the old task is shut down.
  #[serde(rename = "Order")]
  pub order: Option<ServiceSpecRollbackConfigOrderEnum>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum ServiceSpecRollbackConfigFailureActionEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "continue")]
  CONTINUE,
  #[serde(rename = "pause")]
  PAUSE,
}

#[allow(non_camel_case_types)]
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum ServiceSpecRollbackConfigOrderEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "stop-first")]
  STOP_FIRST,
  #[serde(rename = "start-first")]
  START_FIRST,
}

/// Properties that can be configured to access and load balance a service.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct EndpointSpec {
  /// The mode of resolution to use for internal load balancing between tasks.
  #[serde(rename = "Mode")]
  pub mode: Option<EndpointSpecModeEnum>,

  /// List of exposed ports that this service is accessible on from the outside. Ports can only be provided if `vip` resolution mode is used.
  #[serde(rename = "Ports")]
  pub ports: Option<Vec<EndpointPortConfig>>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum EndpointSpecModeEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "vip")]
  VIP,
  #[serde(rename = "dnsrr")]
  DNSRR,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceEndpoint {
  #[serde(rename = "Spec")]
  pub spec: Option<EndpointSpec>,

  #[serde(rename = "Ports")]
  pub ports: Option<Vec<EndpointPortConfig>>,

  #[serde(rename = "VirtualIPs")]
  pub virtual_ips: Option<Vec<ServiceEndpointVirtualIps>>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceEndpointVirtualIps {
  #[serde(rename = "NetworkID")]
  pub network_id: Option<String>,

  #[serde(rename = "Addr")]
  pub addr: Option<String>,
}

/// The status of a service update.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceUpdateStatus {
  #[serde(rename = "State")]
  pub state: Option<ServiceUpdateStatusStateEnum>,

  #[serde(rename = "StartedAt")]
  pub started_at: Option<String>,

  #[serde(rename = "CompletedAt")]
  pub completed_at: Option<String>,

  #[serde(rename = "Message")]
  pub message: Option<String>,
}

#[allow(non_camel_case_types)]
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Default,
  Serialize,
  Deserialize,
)]
pub enum ServiceUpdateStatusStateEnum {
  #[default]
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "updating")]
  UPDATING,
  #[serde(rename = "paused")]
  PAUSED,
  #[serde(rename = "completed")]
  COMPLETED,
  #[serde(rename = "rollback_started")]
  ROLLBACK_STARTED,
  #[serde(rename = "rollback_paused")]
  ROLLBACK_PAUSED,
  #[serde(rename = "rollback_completed")]
  ROLLBACK_COMPLETED,
}

/// The status of the service's tasks. Provided only when requested as part of a ServiceList operation.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceServiceStatus {
  /// The number of tasks for the service currently in the Running state.
  #[serde(rename = "RunningTasks")]
  pub running_tasks: Option<U64>,

  /// The number of tasks for the service desired to be running.
  /// For replicated services, this is the replica count from the service spec.
  /// For global services, this is computed by taking count of all tasks for the service with a Desired State other than Shutdown.
  #[serde(rename = "DesiredTasks")]
  pub desired_tasks: Option<U64>,

  /// The number of tasks for a job that are in the Completed state.
  /// This field must be cross-referenced with the service type, as the value of 0 may mean the service is not in a job mode,
  /// or it may mean the job-mode service has no tasks yet Completed.
  #[serde(rename = "CompletedTasks")]
  pub completed_tasks: Option<U64>,
}

/// The status of the service when it is in one of ReplicatedJob or GlobalJob modes. Absent on Replicated and Global mode services. The JobIteration is an ObjectVersion, but unlike the Service's version, does not need to be sent with an update request.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ServiceJobStatus {
  /// JobIteration is a value increased each time a Job is executed, successfully or otherwise. \"Executed\", in this case, means the job as a whole has been started, not that an individual Task has been launched. A job is \"Executed\" when its ServiceSpec is updated. JobIteration can be used to disambiguate Tasks belonging to different executions of a job.  Though JobIteration will increase with each subsequent execution, it may not necessarily increase by 1, and so JobIteration should not be used to
  #[serde(rename = "JobIteration")]
  pub job_iteration: Option<ObjectVersion>,

  /// The last time, as observed by the server, that this job was started.
  #[serde(rename = "LastExecution")]
  pub last_execution: Option<String>,
}
