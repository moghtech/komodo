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
  path = "/CloneRepo",
  description = "Clones the target repo.",
  request_body(content = CloneRepo),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn clone_repo() {}

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
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CloneRepo {
  /// Id or name
  pub repo: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchCloneRepo",
  description = "Clones multiple Repos in parallel that match pattern.",
  request_body(content = BatchCloneRepo),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_clone_repo() {}

/// Clones multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/PullRepo",
  description = "Pulls the target repo.",
  request_body(content = PullRepo),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn pull_repo() {}

/// Pulls the target repo. Response: [Update].
///
/// Note. Repo must have server attached at `server_id`.
///
/// 1. Pulls the repo on the target server using `git pull`.
/// 2. If `on_pull` is specified, it will be executed after the pull is complete.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct PullRepo {
  /// Id or name
  pub repo: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchPullRepo",
  description = "Pulls multiple Repos in parallel that match pattern.",
  request_body(content = BatchPullRepo),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_pull_repo() {}

/// Pulls multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BuildRepo",
  description = "Builds the target repo, using the attached builder.",
  request_body(content = BuildRepo),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn build_repo() {}

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
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct BuildRepo {
  /// Id or name
  pub repo: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchBuildRepo",
  description = "Builds multiple Repos in parallel that match pattern.",
  request_body(content = BatchBuildRepo),
  responses(
    (status = 200, description = "The batch execution response", body = BatchExecutionResponse),
  ),
)]
pub fn batch_build_repo() {}

/// Builds multiple Repos in parallel that match pattern. Response: [BatchExecutionResponse].
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(BatchExecutionResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CancelRepoBuild",
  description = "Cancels the target repo build.",
  request_body(content = CancelRepoBuild),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn cancel_repo_build() {}

/// Cancels the target repo build.
/// Only does anything if the repo build is `building` when called.
/// Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CancelRepoBuild {
  /// Can be id or name
  pub repo: String,
}
