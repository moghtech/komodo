use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  MongoDocument,
  update::{Update, UpdateListItem},
};

use super::KomodoReadRequest;

/// Get all data for the target update.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetUpdateResponse)]
#[error(serror::Error)]
pub struct GetUpdate {
  /// The update id.
  pub id: String,
}

#[typeshare]
pub type GetUpdateResponse = Update;

//

/// Paginated endpoint for updates matching optional query.
/// More recent updates will be returned first.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListUpdatesResponse)]
#[error(serror::Error)]
pub struct ListUpdates {
  /// An optional mongo query to filter the updates.
  #[cfg_attr(feature = "utoipa", schema(value_type = Option<serde_json::Value>))]
  pub query: Option<MongoDocument>,
  /// Page of updates. Default is 0, which is the most recent data.
  /// Use with the `next_page` field of the response.
  #[serde(default)]
  pub page: u32,
}

/// Response for [ListUpdates].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListUpdatesResponse {
  /// The page of updates, sorted by timestamp descending.
  pub updates: Vec<UpdateListItem>,
  /// If there is a next page of data, pass this to `page` to get it.
  pub next_page: Option<u32>,
}
