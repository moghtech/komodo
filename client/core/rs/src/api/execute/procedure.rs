use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RunProcedure",
  description = "Runs the target Procedure.",
  request_body(content = RunProcedure),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn run_procedure() {}

/// Runs the target Procedure. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RunProcedure {
  /// Id or name
  pub procedure: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchRunProcedure",
  description = "Runs multiple Procedures in parallel that match pattern.",
  request_body(content = BatchRunProcedure),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_run_procedure() {}

/// Runs multiple Procedures in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchRunProcedure {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* procedures
  /// foo-*
  /// # add some more
  /// extra-procedure-1, extra-procedure-2
  /// ```
  pub pattern: String,
}
