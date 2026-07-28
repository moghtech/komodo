use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entities::ResourceTargetVariant;

/// Semi-anonymous core data reporting.
/// Reports only include the reporting-specific core public key
/// (not the keys used for server connection).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreReport {
  /// Reporting specific public key.
  /// Must match public key obtained through request signature.
  pub report_public_key: String,
  /// The total number of users
  pub users: u64,
  /// Resource counts by type
  pub count: HashMap<ResourceTargetVariant, u64>,
}
