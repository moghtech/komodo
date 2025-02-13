use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::builder::{
  Builder, BuilderListItem, BuilderQuery,
};

use super::KomodoReadRequest;

//

/// Get a specific builder by id or name. Response: [Builder].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuilderResponse)]
#[error(serror::Error)]
pub struct GetBuilder {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub builder: String,
}

#[typeshare]
pub type GetBuilderResponse = Builder;

//

/// List builders matching structured query. Response: [ListBuildersResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListBuildersResponse)]
#[error(serror::Error)]
pub struct ListBuilders {
  #[serde(default)]
  pub query: BuilderQuery,
}

#[typeshare]
pub type ListBuildersResponse = Vec<BuilderListItem>;

//

/// List builders matching structured query. Response: [ListFullBuildersResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullBuildersResponse)]
#[error(serror::Error)]
pub struct ListFullBuilders {
  #[serde(default)]
  pub query: BuilderQuery,
}

#[typeshare]
pub type ListFullBuildersResponse = Vec<Builder>;

//

/// Gets a summary of data relating to all builders.
/// Response: [GetBuildersSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetBuildersSummaryResponse)]
#[error(serror::Error)]
pub struct GetBuildersSummary {}

/// Response for [GetBuildersSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetBuildersSummaryResponse {
  /// The total number of builders.
  pub total: u32,
}
