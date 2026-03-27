use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::terminal::{
  ContainerTerminalMode, TerminalRecreateMode, TerminalTarget,
};

/// Connect to a Terminal.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ConnectTerminalQuery {
  /// The target to create terminal for.
  pub target: TerminalTarget,
  /// Terminal name to connect to.
  /// If it may not exist yet, also pass 'init' params
  /// to include initialization.
  /// Default: Depends on target.
  pub terminal: Option<String>,
  /// Pass to init the terminal session
  /// for when the terminal doesn't already exist.
  ///
  /// Example: ?...(query)&init[command]=bash&init[recreate]=DifferentCommand
  pub init: Option<InitTerminal>,
}

/// Args to init the Terminal if needed.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct InitTerminal {
  /// The shell command (eg `bash`) to init the shell.
  ///
  /// Default:
  ///   - Server: Configured on each Periphery
  ///   - Container: `sh`
  pub command: Option<String>,
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
  /// Only relevant for container-type terminals.
  /// Specify the container terminal mode (`exec` or `attach`).
  /// Default: `exec`
  pub mode: Option<ContainerTerminalMode>,
}

/// Execute a terminal command on the given server.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ExecuteTerminalBody {
  /// The target to create terminal for.
  pub target: TerminalTarget,
  /// Terminal name to connect to.
  /// If it may not exist yet, also pass 'init' params
  /// to include initialization.
  /// Default: Depends on target.
  pub terminal: Option<String>,
  /// The command to execute.
  pub command: String,
  /// Pass to init the terminal session
  /// for when the terminal doesn't already exist.
  pub init: Option<InitTerminal>,
}
