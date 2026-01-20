use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::repo::{
  Repo, RepoActionState, RepoListItem, RepoQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetRepo",
  description = "Get a specific repo.",
  request_body(content = GetRepo),
  responses(
    (status = 200, description = "The repo", body = GetRepoResponse),
  ),
)]
pub fn get_repo() {}

/// Get a specific repo. Response: [Repo].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(Repo)]
#[error(mogh_error::Error)]
pub struct GetRepo {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

#[typeshare]
pub type GetRepoResponse = Repo;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListRepos",
  description = "List repos matching optional query.",
  request_body(content = ListRepos),
  responses(
    (status = 200, description = "The list of repos", body = ListReposResponse),
  ),
)]
pub fn list_repos() {}

/// List repos matching optional query. Response: [ListReposResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListReposResponse)]
#[error(mogh_error::Error)]
pub struct ListRepos {
  /// optional structured query to filter repos.
  #[serde(default)]
  pub query: RepoQuery,
}

#[typeshare]
pub type ListReposResponse = Vec<RepoListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListFullRepos",
  description = "List repos matching optional query.",
  request_body(content = ListFullRepos),
  responses(
    (status = 200, description = "The list of repos", body = ListFullReposResponse),
  ),
)]
pub fn list_full_repos() {}

/// List repos matching optional query. Response: [ListFullReposResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullReposResponse)]
#[error(mogh_error::Error)]
pub struct ListFullRepos {
  /// optional structured query to filter repos.
  #[serde(default)]
  pub query: RepoQuery,
}

#[typeshare]
pub type ListFullReposResponse = Vec<Repo>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetRepoActionState",
  description = "Get current action state for the repo.",
  request_body(content = GetRepoActionState),
  responses(
    (status = 200, description = "The repo action state", body = GetRepoActionStateResponse),
  ),
)]
pub fn get_repo_action_state() {}

/// Get current action state for the repo. Response: [RepoActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetRepoActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetRepoActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

#[typeshare]
pub type GetRepoActionStateResponse = RepoActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetReposSummary",
  description = "Gets a summary of data relating to all repos.",
  request_body(content = GetReposSummary),
  responses(
    (status = 200, description = "The repos summary", body = GetReposSummaryResponse),
  ),
)]
pub fn get_repos_summary() {}

/// Gets a summary of data relating to all repos.
/// Response: [GetReposSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetReposSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetReposSummary {}

/// Response for [GetReposSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetReposSummaryResponse {
  /// The total number of repos
  pub total: u32,
  /// The number of repos with Ok state.
  pub ok: u32,
  /// The number of repos currently cloning.
  pub cloning: u32,
  /// The number of repos currently pulling.
  pub pulling: u32,
  /// The number of repos currently building.
  pub building: u32,
  /// The number of repos with failed state.
  pub failed: u32,
  /// The number of repos with unknown state.
  pub unknown: u32,
}
