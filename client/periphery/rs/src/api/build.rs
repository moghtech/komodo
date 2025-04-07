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

/// Get the dockerfile contents on the host, for builds using
/// `files_on_host`.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(GetDockerfileContentsOnHostResponse)]
#[error(serror::Error)]
pub struct GetDockerfileContentsOnHost {
  /// The name of the build
  pub name: String,
  /// The build path for the build.
  pub build_path: String,
  /// The dockerfile path for the build, relative to the build_path
  pub dockerfile_path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetDockerfileContentsOnHostResponse {
  /// The dockerfile contents
  pub contents: String,
}

//

/// Write the dockerfile contents to the file on the host, for build using
/// `files_on_host`.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct WriteDockerfileContentsToHost {
  /// The name of the build
  pub name: String,
  /// The build path for the build.
  pub build_path: String,
  /// The dockerfile path for the build, relative to the build_path
  pub dockerfile_path: String,
  /// The contents to write.
  pub contents: String,
}

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
