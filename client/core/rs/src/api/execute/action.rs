use anyhow::Context;
use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{JsonObject, update::Update};

use super::{BatchExecutionResponse, KomodoExecuteRequest};

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RunAction",
  description = "Run an action.",
  request_body(content = RunAction),
  responses(
    (status = 200, description = "In progress update", body = Update),
  ),
)]
pub fn run_action() {}

/// Runs the target Action. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RunAction {
  /// Id or name
  pub action: String,

  /// Custom arguments which are merged on top of the default arguments.
  /// CLI Format: `"VAR1=val1&VAR2=val2"`
  ///
  /// Webhook-triggered actions use this to pass WEBHOOK_BRANCH and WEBHOOK_BODY.
  #[clap(value_parser = args_parser)]
  #[cfg_attr(feature = "utoipa", schema(value_type = Option<std::collections::HashMap<String, serde_json::Value>>))]
  pub args: Option<JsonObject>,
}

fn args_parser(args: &str) -> anyhow::Result<JsonObject> {
  serde_qs::from_str(args).context("Failed to parse args")
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchRunAction",
  description = "Runs multiple Actions in parallel that match pattern.",
  request_body(content = BatchRunAction),
  responses(
    (status = 200, description = "The batch execution updates", body = BatchExecutionResponse),
  ),
)]
pub fn batch_run_action() {}

/// Runs multiple Actions in parallel that match pattern. Response: [BatchExecutionResponse]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchRunAction {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* actions
  /// foo-*
  /// # add some more
  /// extra-action-1, extra-action-2
  /// ```
  pub pattern: String,
}
