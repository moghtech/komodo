use komodo_client::entities::NoData;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(RotatePrivateKeyResponse)]
#[error(anyhow::Error)]
pub struct RotatePrivateKey {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotatePrivateKeyResponse {
  /// The new public key
  pub public_key: String,
}

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct RotateCorePublicKey {
  /// The new Core public key.
  pub public_key: String,
}
