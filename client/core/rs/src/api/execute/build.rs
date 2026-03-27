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
  path = "/RunBuild",
  description = "Runs the target build.",
  request_body(content = RunBuild),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn run_build() {}

/// Runs the target build. Response: [Update].
///
/// 1. Get a handle to the builder. If using AWS builder, this means starting a builder ec2 instance.
///
/// 2. Clone the repo on the builder. If an `on_clone` commmand is given, it will be executed.
///
/// 3. Execute `docker build {...params}`, where params are determined using the builds configuration.
///
/// 4. If a docker registry is configured, the build will be pushed to the registry.
///
/// 5. If using AWS builder, destroy the builder ec2 instance.
///
/// 6. Deploy any Deployments with *Redeploy on Build* enabled.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RunBuild {
  /// Can be build id or name
  pub build: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchRunBuild",
  description = "Runs multiple builds in parallel that match pattern.",
  request_body(content = BatchRunBuild),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_run_build() {}

/// Runs multiple builds in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
pub struct BatchRunBuild {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* builds
  /// foo-*
  /// # add some more
  /// extra-build-1, extra-build-2
  /// ```
  pub pattern: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CancelBuild",
  description = "Cancels the target build.",
  request_body(content = CancelBuild),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn cancel_build() {}

/// Cancels the target build.
/// Only does anything if the build is `building` when called.
/// Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CancelBuild {
  /// Can be id or name
  pub build: String,
}
