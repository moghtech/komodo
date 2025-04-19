use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Execute a terminal command on the given server.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecuteTerminal {
  /// Server Id or name
  pub server: String,
  /// The name of the terminal on the server to use to execute.
  /// If the terminal at name exists, it will be used to execute the command.
  /// Otherwise, a new terminal will be created for this command, which will
  /// persist until it exits or is deleted.
  pub terminal: String,
  /// The command to execute.
  pub command: String,
}
