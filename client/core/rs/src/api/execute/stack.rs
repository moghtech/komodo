use crate::entities::update::Update;
use anyhow::Context;
use clap::ArgAction::SetTrue;
use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use typeshare::typeshare;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeployStack",
  description = "Deploys the target stack.",
  request_body(content = DeployStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn deploy_stack() {}

/// Deploys the target stack. `docker compose up`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DeployStack {
  /// Id or name
  pub stack: String,
  /// Filter to only deploy specific services.
  /// If empty, will deploy all services.
  ///
  /// Note. For Swarm mode Stacks, this field is not supported and will be ignored.
  #[serde(default)]
  pub services: Vec<String>,
  /// Override the default termination max time.
  /// Only used if the stack needs to be taken down first.
  pub stop_time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDeployStack",
  description = "Deploys multiple Stacks in parallel that match pattern.",
  request_body(content = BatchDeployStack),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_deploy_stack() {}

/// Deploys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchDeployStack {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeployStackIfChanged",
  description = "Checks deployed contents vs latest contents and deploys if changed.",
  request_body(content = DeployStackIfChanged),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn deploy_stack_if_changed() {}

/// Checks deployed contents vs latest contents,
/// and only if any changes found
/// will `docker compose up`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DeployStackIfChanged {
  /// Id or name
  pub stack: String,
  /// Override the default termination max time.
  /// Only used if the stack needs to be taken down first.
  pub stop_time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDeployStackIfChanged",
  description = "Deploys multiple Stacks if changed in parallel that match pattern.",
  request_body(content = BatchDeployStackIfChanged),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_deploy_stack_if_changed() {}

/// Deploys multiple Stacks if changed in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchDeployStackIfChanged {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PullStack",
  description = "Pulls images for the target stack.",
  request_body(content = PullStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pull_stack() {}

/// Pulls images for the target stack. `docker compose pull`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PullStack {
  /// Id or name
  pub stack: String,
  /// Filter to only pull specific services.
  /// If empty, will pull all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchPullStack",
  description = "Pulls multiple Stacks in parallel that match pattern.",
  request_body(content = BatchPullStack),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_pull_stack() {}

/// Pulls multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchPullStack {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StartStack",
  description = "Starts the target stack.",
  request_body(content = StartStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn start_stack() {}

/// Starts the target stack. `docker compose start`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StartStack {
  /// Id or name
  pub stack: String,
  /// Filter to only start specific services.
  /// If empty, will start all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RestartStack",
  description = "Restarts the target stack.",
  request_body(content = RestartStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn restart_stack() {}

/// Restarts the target stack. `docker compose restart`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RestartStack {
  /// Id or name
  pub stack: String,
  /// Filter to only restart specific services.
  /// If empty, will restart all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PauseStack",
  description = "Pauses the target stack.",
  request_body(content = PauseStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pause_stack() {}

/// Pauses the target stack. `docker compose pause`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PauseStack {
  /// Id or name
  pub stack: String,
  /// Filter to only pause specific services.
  /// If empty, will pause all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UnpauseStack",
  description = "Unpauses the target stack.",
  request_body(content = UnpauseStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn unpause_stack() {}

/// Unpauses the target stack. `docker compose unpause`. Response: [Update].
///
/// Note. This is the only way to restart a paused container.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UnpauseStack {
  /// Id or name
  pub stack: String,
  /// Filter to only unpause specific services.
  /// If empty, will unpause all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/StopStack",
  description = "Stops the target stack.",
  request_body(content = StopStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn stop_stack() {}

/// Stops the target stack. `docker compose stop`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct StopStack {
  /// Id or name
  pub stack: String,
  /// Override the default termination max time.
  pub stop_time: Option<i32>,
  /// Filter to only stop specific services.
  /// If empty, will stop all services.
  #[serde(default)]
  pub services: Vec<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DestroyStack",
  description = "Destroys the target stack.",
  request_body(content = DestroyStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn destroy_stack() {}

/// Destoys the target stack. `docker compose down`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct DestroyStack {
  /// Id or name
  pub stack: String,
  /// Filter to only destroy specific services.
  /// If empty, will destroy all services.
  #[serde(default)]
  pub services: Vec<String>,
  /// Pass `--remove-orphans`
  #[serde(default)]
  pub remove_orphans: bool,
  /// Override the default termination max time.
  pub stop_time: Option<i32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RunStackService",
  description = "Runs a one-time command against a service using docker compose run.",
  request_body(content = RunStackService),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn run_stack_service() {}

/// Runs a one-time command against a service using `docker compose run`. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RunStackService {
  /// Id or name
  pub stack: String,
  /// Service to run
  pub service: String,
  /// Command and args to pass to the service container
  #[arg(trailing_var_arg = true, num_args = 1.., allow_hyphen_values = true)]
  pub command: Option<Vec<String>>,
  /// Do not allocate TTY
  #[arg(long = "no-tty", action = SetTrue)]
  pub no_tty: Option<bool>,
  /// Do not start linked services
  #[arg(long = "no-deps", action = SetTrue)]
  pub no_deps: Option<bool>,
  /// Detach container on run
  #[arg(long = "detach", action = SetTrue)]
  pub detach: Option<bool>,
  /// Map service ports to the host
  #[arg(long = "service-ports", action = SetTrue)]
  pub service_ports: Option<bool>,
  /// Extra environment variables for the run
  #[arg(long = "env", short = 'e', value_parser = env_parser)]
  pub env: Option<HashMap<String, String>>,
  /// Working directory inside the container
  #[arg(long = "workdir")]
  pub workdir: Option<String>,
  /// User to run as inside the container
  #[arg(long = "user")]
  pub user: Option<String>,
  /// Override the default entrypoint
  #[arg(long = "entrypoint")]
  pub entrypoint: Option<String>,
  /// Pull the image before running
  #[arg(long = "pull", action = SetTrue)]
  pub pull: Option<bool>,
}

fn env_parser(args: &str) -> anyhow::Result<HashMap<String, String>> {
  serde_qs::from_str(args).context("Failed to parse env")
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDestroyStack",
  description = "Destroys multiple Stacks in parallel that match pattern.",
  request_body(content = BatchDestroyStack),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_destroy_stack() {}

/// Destroys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchDestroyStack {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}
