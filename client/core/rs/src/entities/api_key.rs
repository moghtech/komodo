use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::I64;

/// An api key used to authenticate requests via request headers.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct ApiKey {
  /// Unique key associated with secret
  #[cfg_attr(feature = "mongo", unique_index)]
  pub key: String,

  /// Hash of the secret
  pub secret: String,

  /// User associated with the api key
  #[cfg_attr(feature = "mongo", index)]
  pub user_id: String,

  /// Name associated with the api key for management
  pub name: String,

  /// Timestamp of key creation
  pub created_at: I64,

  /// Expiry of key, or 0 if never expires
  pub expires: I64,
}

impl ApiKey {
  pub fn sanitize(&mut self) {
    self.secret.clear()
  }
}
