use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::terminal::{Terminal, TerminalTarget};

use super::KomodoReadRequest;

//

/// List Terminals.
/// Response: [ListTerminalsResponse].
#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListTerminalsResponse)]
#[error(serror::Error)]
pub struct ListTerminals {
  /// Filter the Terminals returned by the Target.
  pub target: Option<TerminalTarget>,
  /// Return results with resource names instead of ids.
  #[serde(default)]
  pub use_names: bool,
}

#[typeshare]
pub type ListTerminalsResponse = Vec<Terminal>;
