use komodo_client::entities::{
  deployment::Deployment,
  docker::container::{Container, ContainerStats},
  update::Log,
  SearchCombinator, TerminationSignal,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Container)]
#[error(serror::Error)]
pub struct InspectContainer {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct GetContainerLog {
  pub name: String,
  #[serde(default = "default_tail")]
  pub tail: u64,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
}

fn default_tail() -> u64 {
  50
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct GetContainerLogSearch {
  pub name: String,
  pub terms: Vec<String>,
  #[serde(default)]
  pub combinator: SearchCombinator,
  #[serde(default)]
  pub invert: bool,
  /// Enable `--timestamps`
  #[serde(default)]
  pub timestamps: bool,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(ContainerStats)]
#[error(serror::Error)]
pub struct GetContainerStats {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<ContainerStats>)]
#[error(serror::Error)]
pub struct GetContainerStatsList {}

//

// =======
// ACTIONS
// =======

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct Deploy {
  pub deployment: Deployment,
  pub stop_signal: Option<TerminationSignal>,
  pub stop_time: Option<i32>,
  /// Override registry token with one sent from core.
  pub registry_token: Option<String>,
  /// Propogate any secret replacers from core interpolation.
  #[serde(default)]
  pub replacers: Vec<(String, String)>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct StartContainer {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct RestartContainer {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PauseContainer {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct UnpauseContainer {
  pub name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct StopContainer {
  pub name: String,
  pub signal: Option<TerminationSignal>,
  pub time: Option<i32>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct RemoveContainer {
  pub name: String,
  pub signal: Option<TerminationSignal>,
  pub time: Option<i32>,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct RenameContainer {
  pub curr_name: String,
  pub new_name: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Log)]
#[error(serror::Error)]
pub struct PruneContainers {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(serror::Error)]
pub struct StartAllContainers {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(serror::Error)]
pub struct RestartAllContainers {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(serror::Error)]
pub struct PauseAllContainers {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(serror::Error)]
pub struct UnpauseAllContainers {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Log>)]
#[error(serror::Error)]
pub struct StopAllContainers {}
