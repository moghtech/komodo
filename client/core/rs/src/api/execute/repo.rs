use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::{BatchExecutionResponse, KomodoExecuteRequest};

//

/// Clones the target repo. Response: [Update].
///
/// Note. Repo must have server attached at `server_id`.
///
/// 1. Clones the repo on the target server using `git clone https://{$token?}@github.com/${repo} -b ${branch}`.
/// The token will only be used if a github account is specified,
/// and must be declared in the periphery configuration on the target server.
/// 2. If `on_clone` and `on_pull` are specified, they will be executed.
/// `on_clone` will be executed before `on_pull`.
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
pub struct CloneRepo {
  /// Id or name
  pub repo: String,
}

//

/// Clones multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
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
pub struct BatchCloneRepo {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* repos
  /// foo-*
  /// # add some more
  /// extra-repo-1, extra-repo-2
  /// ```
  pub pattern: String,
}

//

/// Pulls the target repo. Response: [Update].
///
/// Note. Repo must have server attached at `server_id`.
///
/// 1. Pulls the repo on the target server using `git pull`.
/// 2. If `on_pull` is specified, it will be executed after the pull is complete.
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
pub struct PullRepo {
  /// Id or name
  pub repo: String,
}

//

/// Pulls multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
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
pub struct BatchPullRepo {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* repos
  /// foo-*
  /// # add some more
  /// extra-repo-1, extra-repo-2
  /// ```
  pub pattern: String,
}

//

/// Builds the target repo, using the attached builder. Response: [Update].
///
/// Note. Repo must have builder attached at `builder_id`.
///
/// 1. Spawns the target builder instance (For AWS type. For Server type, just use CloneRepo).
/// 2. Clones the repo on the builder using `git clone https://{$token?}@github.com/${repo} -b ${branch}`.
/// The token will only be used if a github account is specified,
/// and must be declared in the periphery configuration on the builder instance.
/// 3. If `on_clone` and `on_pull` are specified, they will be executed.
/// `on_clone` will be executed before `on_pull`.
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
pub struct BuildRepo {
  /// Id or name
  pub repo: String,
}

//

/// Builds multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
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
pub struct BatchBuildRepo {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* repos
  /// foo-*
  /// # add some more
  /// extra-repo-1, extra-repo-2
  /// ```
  pub pattern: String,
}

//

/// Cancels the target repo build.
/// Only does anything if the repo build is `building` when called.
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
pub struct CancelRepoBuild {
  /// Can be id or name
  pub repo: String,
}
