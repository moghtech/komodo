use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, MongoDocument, U64, alert::Alert};

use super::KomodoReadRequest;

/// Get a paginated list of alerts sorted by timestamp descending.
/// Response: [ListAlertsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListAlertsResponse)]
#[error(serror::Error)]
pub struct ListAlerts {
  /// Pass a custom mongo query to filter the alerts.
  ///
  /// ## Example JSON
  /// ```json
  /// {
  ///   "resolved": "false",
  ///   "level": "CRITICAL",
  ///   "$or": [
  ///     {
  ///       "target": {
  ///         "type": "Server",
  ///         "id": "6608bf89cb2a12b257ab6c09"
  ///       }
  ///     },
  ///     {
  ///       "target": {
  ///         "type": "Server",
  ///         "id": "660a5f60b74f90d5dae45fa3"
  ///       }
  ///     }
  ///   ]
  /// }
  /// ```
  /// This will filter to only include open alerts that have CRITICAL level on those two servers.
  pub query: Option<MongoDocument>,
  /// Retrieve older results by incrementing the page.
  /// `page: 0` is default, and returns the most recent results.
  #[serde(default)]
  pub page: U64,
}

/// Response for [ListAlerts].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListAlertsResponse {
  pub alerts: Vec<Alert>,
  /// If more alerts exist, the next page will be given here.
  /// Otherwise it will be `null`
  pub next_page: Option<I64>,
}

//

/// Get an alert: Response: [Alert].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetAlertResponse)]
#[error(serror::Error)]
pub struct GetAlert {
  pub id: String,
}

#[typeshare]
pub type GetAlertResponse = Alert;
