use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{tag::Tag, MongoDocument};

use super::KomodoReadRequest;

//

/// Get data for a specific tag. Response [Tag].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
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
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListTagsResponse)]
#[error(serror::Error)]
pub struct ListTags {
  pub query: Option<MongoDocument>,
}

#[typeshare]
pub type ListTagsResponse = Vec<Tag>;
