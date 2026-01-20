use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{MongoDocument, tag::Tag};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetTag",
  description = "Get data for a specific tag.",
  request_body(content = GetTag),
  responses(
    (status = 200, description = "The tag", body = GetTagResponse),
  ),
)]
pub fn get_tag() {}

/// Get data for a specific tag. Response [Tag].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetTagResponse)]
#[error(mogh_error::Error)]
pub struct GetTag {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub tag: String,
}

#[typeshare]
pub type GetTagResponse = Tag;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListTags",
  description = "List data for tags matching optional mongo query.",
  request_body(content = ListTags),
  responses(
    (status = 200, description = "The list of tags", body = ListTagsResponse),
  ),
)]
pub fn list_tags() {}

/// List data for tags matching optional mongo query.
/// Response: [ListTagsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListTagsResponse)]
#[error(mogh_error::Error)]
pub struct ListTags {
  #[cfg_attr(feature = "utoipa", schema(value_type = Option<serde_json::Value>))]
  pub query: Option<MongoDocument>,
}

#[typeshare]
pub type ListTagsResponse = Vec<Tag>;
