use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::entities::ResourceTargetVariant;

/// Semi-anonymous Komodo Core reporting.
/// Reports only include the reporting-specific public key
/// (not the key pair used for server connection).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KomodoReport {
  /// Reporting specific public key.
  /// Must match public key obtained through request signature.
  pub public_key: String,
  /// The Komodo Core version string
  pub version: String,
  /// The total number of users
  pub users: u64,
  /// Resource counts by type
  pub count: HashMap<ResourceTargetVariant, u64>,
}
