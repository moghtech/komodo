use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{MongoDocument, tag::Tag};

use super::KomodoReadRequest;

//

/// Get data for a specific tag. Response [Tag].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetTagResponse)]
#[error(serror::Error)]
pub struct GetTag {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub tag: String,
}

#[typeshare]
pub type GetTagResponse = Tag;

//

/// List data for tags matching optional mongo query.
/// Response: [ListTagsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListTagsResponse)]
#[error(serror::Error)]
pub struct ListTags {
  #[cfg_attr(feature = "utoipa", schema(value_type = Option<serde_json::Value>))]
  pub query: Option<MongoDocument>,
}

#[typeshare]
pub type ListTagsResponse = Vec<Tag>;
