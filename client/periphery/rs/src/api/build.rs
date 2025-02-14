use komodo_client::entities::update::Log;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(BuildResponse)]
#[error(serror::Error)]
pub struct Build {
  pub build: komodo_client::entities::build::Build,
  /// Override registry token with one sent from core.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
  /// Add more tags for this build in addition to the version tags.
  #[serde(default)]
  pub additional_tags: Vec<String>,
}

pub type BuildResponse = Vec<Log>;

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PruneBuilders {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PruneBuildx {}
