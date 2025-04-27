use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Query to connect to a terminal (interactive shell over websocket) on the given server.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectTerminalQuery {
  /// Server Id or name
  pub server: String,
  /// Each periphery can keep multiple terminals open.
  /// If a terminals with the specified name already exists,
  /// it will be attached to.
  /// Otherwise a new terminal will be created for the command,
  /// which will persist until it is deleted using
  /// [DeleteTerminal][crate::api::write::server::DeleteTerminal]
  pub terminal: String,
  /// Optional. The initial command to execute on connection to the shell.
  pub init: Option<String>,
}
