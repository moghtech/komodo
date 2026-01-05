use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::*;

/// Swarm secret list item.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmSecretListItem {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  /// User-defined name of the secret.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// Name of the secrets driver used to fetch the secret's value from an external secret store.
  #[serde(rename = "Driver")]
  pub driver: Option<String>,

  /// Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template.
  /// If no driver is set, no templating is used.
  #[serde(rename = "Templating")]
  pub templating: Option<String>,

  /// Whether the secret is in use by any service
  #[serde(rename = "InUse")]
  pub in_use: bool,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,
}

/// Swarm secret details.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SwarmSecret {
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<SecretSpec>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SecretSpec {
  /// User-defined name of the secret.
  #[serde(rename = "Name")]
  pub name: Option<String>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  /// Data is the data to store as a secret, formatted as a Base64-url-safe-encoded ([RFC 4648](https://tools.ietf.org/html/rfc4648#section-5)) string.
  /// It must be empty if the Driver field is set, in which case the data is loaded from an external secret store.
  /// The maximum allowed size is 500KB, as defined in [MaxSecretSize](https://pkg.go.dev/github.com/moby/swarmkit/v2@v2.0.0-20250103191802-8c1959736554/api/validation#MaxSecretSize).
  /// This field is only used to _create_ a secret, and is not returned by other endpoints.
  #[serde(rename = "Data")]
  pub data: Option<String>,

  /// Name of the secrets driver used to fetch the secret's value from an external secret store.
  #[serde(rename = "Driver")]
  pub driver: Option<Driver>,

  /// Templating driver, if applicable  Templating controls whether and how to evaluate the config payload as a template.
  /// If no driver is set, no templating is used.
  #[serde(rename = "Templating")]
  pub templating: Option<Driver>,
}
