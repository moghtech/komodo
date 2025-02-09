use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::repo::{
  Repo, RepoActionState, RepoListItem, RepoQuery,
};

use super::KomodoReadRequest;

//

/// Get a specific repo. Response: [Repo].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(Repo)]
#[error(serror::Error)]
pub struct GetRepo {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

#[typeshare]
pub type GetRepoResponse = Repo;

//

/// List repos matching optional query. Response: [ListReposResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListReposResponse)]
#[error(serror::Error)]
pub struct ListRepos {
  /// optional structured query to filter repos.
  #[serde(default)]
  pub query: RepoQuery,
}

#[typeshare]
pub type ListReposResponse = Vec<RepoListItem>;

//

/// List repos matching optional query. Response: [ListFullReposResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullReposResponse)]
#[error(serror::Error)]
pub struct ListFullRepos {
  /// optional structured query to filter repos.
  #[serde(default)]
  pub query: RepoQuery,
}

#[typeshare]
pub type ListFullReposResponse = Vec<Repo>;

//

/// Get current action state for the repo. Response: [RepoActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetRepoActionStateResponse)]
#[error(serror::Error)]
pub struct GetRepoActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

#[typeshare]
pub type GetRepoActionStateResponse = RepoActionState;

//

/// Gets a summary of data relating to all repos.
/// Response: [GetReposSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetReposSummaryResponse)]
#[error(serror::Error)]
pub struct GetReposSummary {}

/// Response for [GetReposSummary]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

//

/// Get a target Repo's configured webhooks. Response: [GetRepoWebhooksEnabledResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetRepoWebhooksEnabledResponse)]
#[error(serror::Error)]
pub struct GetRepoWebhooksEnabled {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub repo: String,
}

/// Response for [GetRepoWebhooksEnabled]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRepoWebhooksEnabledResponse {
  /// Whether the repo webhooks can even be managed.
  /// The repo owner must be in `github_webhook_app.owners` list to be managed.
  pub managed: bool,
  /// Whether pushes to branch trigger clone. Will always be false if managed is false.
  pub clone_enabled: bool,
  /// Whether pushes to branch trigger pull. Will always be false if managed is false.
  pub pull_enabled: bool,
  /// Whether pushes to branch trigger build. Will always be false if managed is false.
  pub build_enabled: bool,
}
