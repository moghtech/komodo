use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::ResourceTarget;

use super::KomodoReadRequest;

/// Response containing pretty formatted toml contents.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TomlResponse {
  pub toml: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ExportAllResourcesToToml",
  description = "Get sync toml for all resources which the user has permissions to view.",
  request_body(content = ExportAllResourcesToToml),
  responses(
    (status = 200, description = "The toml response", body = ExportAllResourcesToTomlResponse),
  ),
)]
pub fn export_all_resources_to_toml() {}

/// Get sync toml for all resources which the user has permissions to view.
/// Response: [TomlResponse].
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ExportAllResourcesToTomlResponse)]
#[error(mogh_error::Error)]
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ExportResourcesToToml",
  description = "Get sync toml for specific resources, variables, and user groups.",
  request_body(content = ExportResourcesToToml),
  responses(
    (status = 200, description = "The toml response", body = ExportResourcesToTomlResponse),
  ),
)]
pub fn export_resources_to_toml() {}

/// Get sync toml for specific resources, variables, and user groups.
/// Response: [TomlResponse].
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ExportResourcesToTomlResponse)]
#[error(mogh_error::Error)]
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
