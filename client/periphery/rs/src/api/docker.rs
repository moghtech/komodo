use komodo_client::entities::{
  docker::{
    image::{Image, ImageHistoryResponseItem},
    network::Network,
    volume::Volume,
  },
  update::Log,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

// =====
// IMAGE
// =====

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Image)]
#[error(anyhow::Error)]
pub struct InspectImage {
  pub name: String,
}

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Vec<ImageHistoryResponseItem>)]
#[error(anyhow::Error)]
pub struct ImageHistory {
  pub name: String,
}

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(GetLatestImageDigestResponse)]
#[error(anyhow::Error)]
pub struct GetLatestImageDigest {
  /// The name of the image.
  pub name: String,
  /// Optional account to use to pull the image
  pub account: Option<String>,
  /// Override registry token for account with one sent from core.
  pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLatestImageDigestResponse {
  /// The latest digest for the image.
  pub digest: String,
}

//

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct PullImage {
  /// The name of the image.
  pub name: String,
  /// Optional account to use to pull the image
  pub account: Option<String>,
  /// Override registry token for account with one sent from core.
  pub token: Option<String>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct DeleteImage {
  /// Id or name
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct PruneImages {}

// =======
// NETWORK
// =======

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Network)]
#[error(anyhow::Error)]
pub struct InspectNetwork {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct CreateNetwork {
  pub name: String,
  pub driver: Option<String>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct DeleteNetwork {
  /// Id or name
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct PruneNetworks {}

// ======
// VOLUME
// ======

#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Volume)]
#[error(anyhow::Error)]
pub struct InspectVolume {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct DeleteVolume {
  /// Id or name
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(anyhow::Error)]
pub struct PruneVolumes {}
