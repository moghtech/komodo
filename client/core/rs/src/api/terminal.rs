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
  /// If a terminals with the specified name does not exist,
  /// the call will fail.
  /// Create a terminal using [CreateTerminal][super::write::server::CreateTerminal]
  pub terminal: String,
}

/// Query to connect to a container exec session (interactive shell over websocket) on the given server.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectContainerExecQuery {
  /// Server Id or name
  pub server: String,
  /// The container name
  pub container: String,
  /// The shell to connect to
  pub shell: String,
}

/// Query to connect to a container exec session (interactive shell over websocket) on the given Deployment.
/// This call will use access to the Deployment Terminal to permission the call.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectDeploymentExecQuery {
  /// Deployment Id or name
  pub deployment: String,
  /// The shell to connect to
  pub shell: String,
}

/// Query to connect to a container exec session (interactive shell over websocket) on the given Stack / service.
/// This call will use access to the Stack Terminal to permission the call.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectStackExecQuery {
  /// Stack Id or name
  pub stack: String,
  /// The service name to connect to
  pub service: String,
  /// The shell to connect to
  pub shell: String,
}

/// Execute a terminal command on the given server.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecuteTerminalBody {
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
