use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  U64,
  terminal::{Terminal, TerminalSortBy, TerminalTarget},
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListTerminals",
  description = "List Terminals.",
  request_body(content = ListTerminals),
  responses(
    (status = 200, description = "The list of terminals", body = ListTerminalsResponse),
  ),
)]
pub fn list_terminals() {}

/// List Terminals.
/// Response: [ListTerminalsResponse].
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListTerminalsResponse)]
#[error(mogh_error::Error)]
pub struct ListTerminals {
  /// Filter the Terminals returned by the Target.
  pub target: Option<TerminalTarget>,
  /// Return results with resource names instead of ids.
  #[serde(default)]
  pub use_names: bool,

  /// Filter by terminal name.
  /// Returned terminals have names which contain all terms.
  #[serde(default)]
  pub terms: Vec<String>,

  /// Retrieve more results by incrementing the page.
  /// `page: 0` is default.
  #[serde(default)]
  pub page: U64,

  /// Set the limit for number of terminals per-page.
  /// `limit: 100` is default.
  ///
  /// Passing `limit: 0` returns all results (unlimited).
  ///
  /// Note: the page logic relies on this being consistent
  /// across queries for more pages.
  pub limit: Option<U64>,

  /// Sort the results by this field.
  /// Defaults to Name.
  #[serde(default)]
  pub sort_by: TerminalSortBy,

  /// Reverse the sort direction.
  #[serde(default)]
  pub sort_desc: bool,
}

#[typeshare]
pub type ListTerminalsResponse = Vec<Terminal>;
