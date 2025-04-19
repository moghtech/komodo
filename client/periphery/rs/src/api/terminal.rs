use komodo_client::entities::NoData;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<String>)]
#[error(serror::Error)]
pub struct ListTerminals {}

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteTerminal {
  pub terminal: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecuteTerminal {
  /// Each periphery can keep multiple terminals open.
  /// If a terminal with the specified name already exists,
  /// the command will execute on it.
  /// Otherwise a new terminal will be created for the command.
  pub terminal: String,
  /// The command to execute in the shell.
  pub command: String,
}
