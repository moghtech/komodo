use komodo_client::entities::{docker::volume::Volume, update::Log};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Volume)]
#[error(serror::Error)]
pub struct InspectVolume {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct DeleteVolume {
  /// Id or name
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PruneVolumes {}
