use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::builder::{
  Builder, BuilderListItem, BuilderQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetBuilder",
  description = "Get a specific builder by id or name.",
  request_body(content = GetBuilder),
  responses(
    (status = 200, description = "The builder", body = crate::entities::builder::BuilderSchema),
  ),
)]
pub fn get_builder() {}

/// Get a specific builder by id or name. Response: [Builder].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuilderResponse)]
#[error(mogh_error::Error)]
pub struct GetBuilder {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub builder: String,
}

#[typeshare]
pub type GetBuilderResponse = Builder;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListBuilders",
  description = "List builders matching structured query.",
  request_body(content = ListBuilders),
  responses(
    (status = 200, description = "The list of builders", body = ListBuildersResponse),
  ),
)]
pub fn list_builders() {}

/// List builders matching structured query. Response: [ListBuildersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildersResponse)]
#[error(mogh_error::Error)]
pub struct ListBuilders {
  #[serde(default)]
  pub query: BuilderQuery,
}

#[typeshare]
pub type ListBuildersResponse = Vec<BuilderListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListFullBuilders",
  description = "List builders matching structured query.",
  request_body(content = ListFullBuilders),
  responses(
    (status = 200, description = "The list of builders", body = ListFullBuildersResponse),
  ),
)]
pub fn list_full_builders() {}

/// List builders matching structured query. Response: [ListFullBuildersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullBuildersResponse)]
#[error(mogh_error::Error)]
pub struct ListFullBuilders {
  #[serde(default)]
  pub query: BuilderQuery,
}

#[typeshare]
pub type ListFullBuildersResponse = Vec<Builder>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetBuildersSummary",
  description = "Gets a summary of data relating to all builders.",
  request_body(content = GetBuildersSummary),
  responses(
    (status = 200, description = "The builders summary", body = GetBuildersSummaryResponse),
  ),
)]
pub fn get_builders_summary() {}

/// Gets a summary of data relating to all builders.
/// Response: [GetBuildersSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildersSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetBuildersSummary {}

/// Response for [GetBuildersSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetBuildersSummaryResponse {
  /// The total number of builders.
  pub total: u32,
}
