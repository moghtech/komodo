use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::*;

/// Swarm config list item.
/// Returned by `docker config ls --format json`
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmConfigListItem {
  /// User-defined name of the config.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// Whether the config is in use by any service
  #[serde(default, rename = "InUse")]
  pub in_use: bool,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  /// User-defined key/value metadata, formatted as a string:
  /// `"lab1=val1,lab2=val2"`.
  #[serde(rename = "Labels")]
  pub labels: Option<String>,
}

/// Swarm config details.
///
/// This would be just "SwarmConfig", but that would
/// conflict with the Swarm (Komodo resource) Config type,
/// which is also SwarmConfig.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmConfigDetails {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<ConfigSpec>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ConfigSpec {
  /// User-defined name of the config.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  /// Data is the data to store as a config, formatted as a Base64-url-safe-encoded ([RFC 4648](https://tools.ietf.org/html/rfc4648#section-5)) string.
  /// It must be empty if the Driver field is set, in which case the data is loaded from an external secret store.
  /// The maximum allowed size is 500KB, as defined in [MaxSecretSize](https://pkg.go.dev/github.com/moby/swarmkit/v2@v2.0.0-20250103191802-8c1959736554/api/validation#MaxSecretSize).
  #[serde(rename = "Data")]
  pub data: Option<String>,

  /// Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template. If no driver is set, no templating is used.
  #[serde(rename = "Templating")]
  pub templating: Option<Driver>,
}
