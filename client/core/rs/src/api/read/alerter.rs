use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::alerter::{
  Alerter, AlerterListItem, AlerterQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetAlerter",
  description = "Get a specific alerter.",
  request_body(content = GetAlerter),
  responses(
    (status = 200, description = "The alerter", body = crate::entities::alerter::AlerterSchema),
  ),
)]
pub fn get_alerter() {}

/// Get a specific alerter. Response: [Alerter].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetAlerterResponse)]
#[error(mogh_error::Error)]
pub struct GetAlerter {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub alerter: String,
}

#[typeshare]
pub type GetAlerterResponse = Alerter;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListAlerters",
  description = "List alerters matching optional query.",
  request_body(content = ListAlerters),
  responses(
    (status = 200, description = "The list of alerters", body = ListAlertersResponse),
  ),
)]
pub fn list_alerters() {}

/// List alerters matching optional query. Response: [ListAlertersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListAlertersResponse)]
#[error(mogh_error::Error)]
pub struct ListAlerters {
  /// Structured query to filter alerters.
  #[serde(default)]
  pub query: AlerterQuery,
}

#[typeshare]
pub type ListAlertersResponse = Vec<AlerterListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullAlerters",
  description = "List full alerters matching optional query.",
  request_body(content = ListFullAlerters),
  responses(
    (status = 200, description = "The list of alerters", body = ListFullAlertersResponse),
  ),
)]
pub fn list_full_alerters() {}

/// List full alerters matching optional query. Response: [ListFullAlertersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullAlertersResponse)]
#[error(mogh_error::Error)]
pub struct ListFullAlerters {
  /// Structured query to filter alerters.
  #[serde(default)]
  pub query: AlerterQuery,
}

#[typeshare]
pub type ListFullAlertersResponse = Vec<Alerter>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetAlertersSummary",
  description = "Gets a summary of data relating to all alerters.",
  request_body(content = GetAlertersSummary),
  responses(
    (status = 200, description = "The alerters summary", body = GetAlertersSummaryResponse),
  ),
)]
pub fn get_alerters_summary() {}

/// Gets a summary of data relating to all alerters.
/// Response: [GetAlertersSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetAlertersSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetAlertersSummary {}

/// Response for [GetAlertersSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetAlertersSummaryResponse {
  pub total: u32,
}
