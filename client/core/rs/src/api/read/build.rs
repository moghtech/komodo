use std::cmp::Ordering;

use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  build::{Build, BuildActionState, BuildListItem, BuildQuery},
  Version, I64,
};

use super::KomodoReadRequest;

//

/// Get a specific build. Response: [Build].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildResponse)]
#[error(serror::Error)]
pub struct GetBuild {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type GetBuildResponse = Build;

//

/// List builds matching optional query. Response: [ListBuildsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildsResponse)]
#[error(serror::Error)]
pub struct ListBuilds {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListBuildsResponse = Vec<BuildListItem>;

//

/// List builds matching optional query. Response: [ListFullBuildsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullBuildsResponse)]
#[error(serror::Error)]
pub struct ListFullBuilds {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListFullBuildsResponse = Vec<Build>;

//

/// Get current action state for the build. Response: [BuildActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildActionStateResponse)]
#[error(serror::Error)]
pub struct GetBuildActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type GetBuildActionStateResponse = BuildActionState;

//

/// Gets a summary of data relating to all builds.
/// Response: [GetBuildsSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildsSummaryResponse)]
#[error(serror::Error)]
pub struct GetBuildsSummary {}

/// Response for [GetBuildsSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetBuildsSummaryResponse {
  /// The total number of builds in Komodo.
  pub total: u32,
  /// The number of builds with Ok state.
  pub ok: u32,
  /// The number of builds with Failed state.
  pub failed: u32,
  /// The number of builds currently building.
  pub building: u32,
  /// The number of builds with unknown state.
  pub unknown: u32,
}

//

/// Gets summary and timeseries breakdown of the last months build count / time for charting.
/// Response: [GetBuildMonthlyStatsResponse].
///
/// Note. This method is paginated. One page is 30 days of data.
/// Query for older pages by incrementing the page, starting at 0.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildMonthlyStatsResponse)]
#[error(serror::Error)]
pub struct GetBuildMonthlyStats {
  /// Query for older data by incrementing the page.
  /// `page: 0` is the default, and will return the most recent data.
  #[serde(default)]
  pub page: u32,
}

/// Response for [GetBuildMonthlyStats].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetBuildMonthlyStatsResponse {
  pub total_time: f64,  // in hours
  pub total_count: f64, // number of builds
  pub days: Vec<BuildStatsDay>,
}

/// Item in [GetBuildMonthlyStatsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BuildStatsDay {
  pub time: f64,
  pub count: f64,
  pub ts: f64,
}

impl GetBuildMonthlyStatsResponse {
  pub fn new(
    mut days: Vec<BuildStatsDay>,
  ) -> GetBuildMonthlyStatsResponse {
    days.sort_by(|a, b| {
      if a.ts < b.ts {
        Ordering::Less
      } else {
        Ordering::Greater
      }
    });
    let mut total_time = 0.0;
    let mut total_count = 0.0;
    for day in &days {
      total_time += day.time;
      total_count += day.count;
    }
    GetBuildMonthlyStatsResponse {
      total_time,
      total_count,
      days,
    }
  }
}

//

/// Retrieve versions of the build that were built in the past and available for deployment,
/// sorted by most recent first.
/// Response: [ListBuildVersionsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildVersionsResponse)]
#[error(serror::Error)]
pub struct ListBuildVersions {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
  /// Filter to only include versions matching this major version.
  pub major: Option<i32>,
  /// Filter to only include versions matching this minor version.
  pub minor: Option<i32>,
  /// Filter to only include versions matching this patch version.
  pub patch: Option<i32>,
  /// Limit the number of included results. Default is no limit.
  pub limit: Option<I64>,
}

#[typeshare]
pub type ListBuildVersionsResponse = Vec<BuildVersionResponseItem>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BuildVersionResponseItem {
  pub version: Version,
  pub ts: I64,
}

//

/// Gets a list of existing values used as extra args across other builds.
/// Useful to offer suggestions. Response: [ListCommonBuildExtraArgsResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonBuildExtraArgsResponse)]
#[error(serror::Error)]
pub struct ListCommonBuildExtraArgs {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListCommonBuildExtraArgsResponse = Vec<String>;

//

/// Get whether a Build's target repo has a webhook for the build configured. Response: [GetBuildWebhookEnabledResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildWebhookEnabledResponse)]
#[error(serror::Error)]
pub struct GetBuildWebhookEnabled {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

/// Response for [GetBuildWebhookEnabled]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetBuildWebhookEnabledResponse {
  /// Whether the repo webhooks can even be managed.
  /// The repo owner must be in `github_webhook_app.owners` list to be managed.
  pub managed: bool,
  /// Whether pushes to branch trigger build. Will always be false if managed is false.
  pub enabled: bool,
}
