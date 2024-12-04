use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

/// Runs the target Action. Response: [Update]
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
pub struct RunAction {
  /// Id or name
  pub action: String,
}

/// Runs multiple Actions in parallel that match pattern. Response: [BatchExecutionResponse]
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
#[response(BatchExecutionResponse)]
#[error(serror::Error)]
pub struct BatchRunAction {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```
  /// # match all foo-* actions
  /// foo-*
  /// # add some more
  /// extra-action-1, extra-action-2
  /// ```
  pub pattern: String,
}
