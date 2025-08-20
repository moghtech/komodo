use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

//

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
pub struct RunBuild {
  /// Can be build id or name
  pub build: String,
}

//

/// Runs multiple builds in parallel that match pattern. Response: [BatchExecutionResponse].
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

/// Cancels the target build.
/// Only does anything if the build is `building` when called.
/// Response: [Update]
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
#[response(Update)]
#[error(serror::Error)]
pub struct CancelBuild {
  /// Can be id or name
  pub build: String,
}
