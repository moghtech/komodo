use komodo_client::{
  api::write::TerminalRecreateMode,
  entities::{NoData, server::TerminalInfo},
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<TerminalInfo>)]
#[error(serror::Error)]
pub struct ListTerminals {}

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(serror::Error)]
pub struct CreateTerminal {
  /// The name of the terminal to create
  pub name: String,
  /// The shell command (eg `bash`) to init the shell.
  ///
  /// This can also include args:
  /// `docker exec -it container sh`
  #[serde(default = "default_command")]
  pub command: String,
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
}

fn default_command() -> String {
  String::from("bash")
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteTerminal {
  /// The name of the terminal to delete
  pub terminal: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteAllTerminals {}

//

/// Create a single use auth token to connect to periphery terminal websocket.
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(CreateTerminalAuthTokenResponse)]
#[error(serror::Error)]
pub struct CreateTerminalAuthToken {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateTerminalAuthTokenResponse {
  pub token: String,
}

//

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectTerminalQuery {
  /// Use [CreateTerminalAuthToken] to create a single-use
  /// token to send in the query.
  pub token: String,
  /// Each periphery can keep multiple terminals open.
  /// If a terminal with the specified name already exists,
  /// it will be attached to.
  /// Otherwise a new terminal will be created,
  /// which will persist until it is either exited via command (ie `exit`),
  /// or deleted using [DeleteTerminal]
  pub terminal: String,
  /// Optional. The initial command to execute on connection to the shell.
  pub init: Option<String>,
}
