use std::cmp::Ordering;

use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  I64, Version,
  build::{Build, BuildActionState, BuildListItem, BuildQuery},
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetBuild",
  description = "Get a specific build.",
  request_body(content = GetBuild),
  responses(
    (status = 200, description = "The build", body = crate::entities::build::BuildSchema),
  ),
)]
pub fn get_build() {}

/// Get a specific build. Response: [Build].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildResponse)]
#[error(mogh_error::Error)]
pub struct GetBuild {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type GetBuildResponse = Build;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListBuilds",
  description = "List builds matching optional query.",
  request_body(content = ListBuilds),
  responses(
    (status = 200, description = "The list of builds", body = ListBuildsResponse),
  ),
)]
pub fn list_builds() {}

/// List builds matching optional query. Response: [ListBuildsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildsResponse)]
#[error(mogh_error::Error)]
pub struct ListBuilds {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListBuildsResponse = Vec<BuildListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullBuilds",
  description = "List builds matching optional query.",
  request_body(content = ListFullBuilds),
  responses(
    (status = 200, description = "The list of builds", body = ListFullBuildsResponse),
  ),
)]
pub fn list_full_builds() {}

/// List builds matching optional query. Response: [ListFullBuildsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullBuildsResponse)]
#[error(mogh_error::Error)]
pub struct ListFullBuilds {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListFullBuildsResponse = Vec<Build>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetBuildActionState",
  description = "Get current action state for the build.",
  request_body(content = GetBuildActionState),
  responses(
    (status = 200, description = "The build action state", body = GetBuildActionStateResponse),
  ),
)]
pub fn get_build_action_state() {}

/// Get current action state for the build. Response: [BuildActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetBuildActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub build: String,
}

#[typeshare]
pub type GetBuildActionStateResponse = BuildActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetBuildsSummary",
  description = "Gets a summary of data relating to all builds.",
  request_body(content = GetBuildsSummary),
  responses(
    (status = 200, description = "The builds summary", body = GetBuildsSummaryResponse),
  ),
)]
pub fn get_builds_summary() {}

/// Gets a summary of data relating to all builds.
/// Response: [GetBuildsSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildsSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetBuildsSummary {}

/// Response for [GetBuildsSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetBuildMonthlyStats",
  description = "Gets summary and timeseries breakdown of the last months build count / time for charting.",
  request_body(content = GetBuildMonthlyStats),
  responses(
    (status = 200, description = "The build monthly stats", body = GetBuildMonthlyStatsResponse),
  ),
)]
pub fn get_build_monthly_stats() {}

/// Gets summary and timeseries breakdown of the last months build count / time for charting.
/// Response: [GetBuildMonthlyStatsResponse].
///
/// Note. This method is paginated. One page is 30 days of data.
/// Query for older pages by incrementing the page, starting at 0.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildMonthlyStatsResponse)]
#[error(mogh_error::Error)]
pub struct GetBuildMonthlyStats {
  /// Query for older data by incrementing the page.
  /// `page: 0` is the default, and will return the most recent data.
  #[serde(default)]
  pub page: u32,
}

/// Response for [GetBuildMonthlyStats].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetBuildMonthlyStatsResponse {
  pub total_time: f64,  // in hours
  pub total_count: f64, // number of builds
  pub days: Vec<BuildStatsDay>,
}

/// Item in [GetBuildMonthlyStatsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListBuildVersions",
  description = "Retrieve versions of the build that were built in the past and available for deployment, sorted by most recent first.",
  request_body(content = ListBuildVersions),
  responses(
    (status = 200, description = "The list of build versions", body = ListBuildVersionsResponse),
  ),
)]
pub fn list_build_versions() {}

/// Retrieve versions of the build that were built in the past and available for deployment,
/// sorted by most recent first.
/// Response: [ListBuildVersionsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildVersionsResponse)]
#[error(mogh_error::Error)]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BuildVersionResponseItem {
  pub version: Version,
  pub ts: I64,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListCommonBuildExtraArgs",
  description = "Gets a list of existing values used as extra args across other builds.",
  request_body(content = ListCommonBuildExtraArgs),
  responses(
    (status = 200, description = "The common extra args", body = ListCommonBuildExtraArgsResponse),
  ),
)]
pub fn list_common_build_extra_args() {}

/// Gets a list of existing values used as extra args across other builds.
/// Useful to offer suggestions. Response: [ListCommonBuildExtraArgsResponse]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListCommonBuildExtraArgsResponse)]
#[error(mogh_error::Error)]
pub struct ListCommonBuildExtraArgs {
  /// optional structured query to filter builds.
  #[serde(default)]
  pub query: BuildQuery,
}

#[typeshare]
pub type ListCommonBuildExtraArgsResponse = Vec<String>;
