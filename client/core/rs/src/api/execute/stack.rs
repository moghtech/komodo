use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

/// Deploys the target stack. `docker compose up`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct DeployStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to "compose up"
  pub service: Option<String>,
  /// Override the default termination max time.
  /// Only used if the stack needs to be taken down first.
  pub stop_time: Option<i32>,
}

//

/// Deploys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchDeployStack {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}

//

/// Checks deployed contents vs latest contents,
/// and only if any changes found
/// will `docker compose up`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct DeployStackIfChanged {
  /// Id or name
  pub stack: String,
  /// Override the default termination max time.
  /// Only used if the stack needs to be taken down first.
  pub stop_time: Option<i32>,
}

//

/// Deploys multiple Stacks if changed in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchDeployStackIfChanged {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}

//

/// Pulls images for the target stack. `docker compose pull`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct PullStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to start
  pub service: Option<String>,
}

//

/// Starts the target stack. `docker compose start`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct StartStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to start
  pub service: Option<String>,
}

//

/// Restarts the target stack. `docker compose restart`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RestartStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to restart
  pub service: Option<String>,
}

//

/// Pauses the target stack. `docker compose pause`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct PauseStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to pause
  pub service: Option<String>,
}

//

/// Unpauses the target stack. `docker compose unpause`. Response: [Update].
///
/// Note. This is the only way to restart a paused container.
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct UnpauseStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to unpause
  pub service: Option<String>,
}

//

/// Stops the target stack. `docker compose stop`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct StopStack {
  /// Id or name
  pub stack: String,
  /// Override the default termination max time.
  pub stop_time: Option<i32>,
  /// Optionally specify a specific service to stop
  pub service: Option<String>,
}

//

/// Destoys the target stack. `docker compose down`. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct DestroyStack {
  /// Id or name
  pub stack: String,
  /// Optionally specify a specific service to destroy
  pub service: Option<String>,
  /// Pass `--remove-orphans`
  #[serde(default)]
  pub remove_orphans: bool,
  /// Override the default termination max time.
  pub stop_time: Option<i32>,
}

//

/// Destroys multiple Stacks in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchDestroyStack {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///d
  /// Example:
  /// ```
  /// # match all foo-* stacks
  /// foo-*
  /// # add some more
  /// extra-stack-1, extra-stack-2
  /// ```
  pub pattern: String,
}
