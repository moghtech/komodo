use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::ResourceTarget;

use super::KomodoReadRequest;

/// Response containing pretty formatted toml contents.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlResponse {
  pub toml: String,
}

//

/// Get pretty formatted monrun sync toml for all resources
/// which the user has permissions to view.
/// Response: [TomlResponse].
#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ExportAllResourcesToTomlResponse)]
#[error(serror::Error)]
pub struct ExportAllResourcesToToml {
  /// Whether to include any resources (servers, stacks, etc.)
  /// in the exported contents.
  /// Default: `true`
  #[serde(default = "default_include_resources")]
  pub include_resources: bool,
  /// Filter resources by tag.
  /// Accepts tag name or id. Empty array will not filter by tag.
  #[serde(default)]
  pub tags: Vec<String>,
  /// Whether to include variables in the exported contents.
  /// Default: false
  #[serde(default)]
  pub include_variables: bool,
  /// Whether to include user groups in the exported contents.
  /// Default: false
  #[serde(default)]
  pub include_user_groups: bool,
}

fn default_include_resources() -> bool {
  true
}

#[typeshare]
pub type ExportAllResourcesToTomlResponse = TomlResponse;

//

/// Get pretty formatted monrun sync toml for specific resources and user groups.
/// Response: [TomlResponse].
#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ExportResourcesToTomlResponse)]
#[error(serror::Error)]
pub struct ExportResourcesToToml {
  /// The targets to include in the export.
  #[serde(default)]
  pub targets: Vec<ResourceTarget>,
  /// The user group names or ids to include in the export.
  #[serde(default)]
  pub user_groups: Vec<String>,
  /// Whether to include variables
  #[serde(default)]
  pub include_variables: bool,
}

#[typeshare]
pub type ExportResourcesToTomlResponse = TomlResponse;
