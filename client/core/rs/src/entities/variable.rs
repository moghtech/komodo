use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// A non-secret global variable which can be interpolated into deployment
/// environment variable values and build argument values.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct Variable {
  /// Unique name associated with the variable.
  /// Instances of '[[variable.name]]' in value will be replaced with 'variable.value'.
  #[cfg_attr(feature = "mongo", unique_index)]
  pub name: String,
  /// A description for the variable.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub description: String,
  /// The value associated with the variable.
  #[serde(default)]
  pub value: String,
  /// If marked as secret, the variable value will be hidden in updates / logs.
  /// Additionally the value will not be served in read requests by non admin users.
  ///
  /// Note that the value is NOT encrypted in the database, and will likely show up in database logs.
  /// The security of these variables comes down to the security
  /// of the database (system level encryption, network isolation, etc.)
  #[serde(default)]
  pub is_secret: bool,
}
