use komodo_client::entities::{
  docker::network::Network, update::Log,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Network)]
#[error(serror::Error)]
pub struct InspectNetwork {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct CreateNetwork {
  pub name: String,
  pub driver: Option<String>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct DeleteNetwork {
  /// Id or name
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PruneNetworks {}
