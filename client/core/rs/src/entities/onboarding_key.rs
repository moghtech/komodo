use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::I64;

/// An public key used to authenticate new Periphery -> Core connections
/// to join Komodo as a newly created Server.
///
/// Server onboarding keys correspond to private / public key pairs.
/// While the public key is stored, the private key will only be returned to the user,
/// The private key will not be stored or available afterwards, just like the api key "secret".
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct OnboardingKey {
  /// Unique public key associated the creation private key.
  #[cfg_attr(feature = "mongo", unique_index)]
  pub public_key: String,

  /// Disable the onboarding key when not in use.
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default)]
  pub enabled: bool,

  /// Expiry of key, or 0 if never expires
  #[serde(default)]
  #[cfg_attr(feature = "mongo", index)]
  pub expires: I64,

  /// Name associated with the api key for management
  #[serde(default)]
  pub name: String,

  /// The [Server](crate::entities::server::Server) ids onboarded by this Creation Key
  #[serde(default)]
  pub onboarded: Vec<String>,

  /// Timestamp of key creation
  #[serde(default)]
  pub created_at: I64,

  /// Default tags to give to Servers created with this key.
  #[serde(default)]
  pub tags: Vec<String>,

  /// Allows the Onboarding Key to be used to:
  ///
  /// 1. Enable a disabled Server
  /// 2. Remove Server 'address' configuration, allowing Periphery -> Core connection.
  /// 3. Update existing Server's public keys.
  #[serde(default)]
  pub fix_existing_servers: bool,

  /// Optional. If specified, copy this Server config when initializing
  /// the Server.
  #[serde(default)]
  pub copy_server: String,

  /// Also create a Builder for the Server.
  #[serde(default)]
  pub create_builder: bool,
}
