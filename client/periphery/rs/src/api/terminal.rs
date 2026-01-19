use komodo_client::entities::{
  NoData,
  terminal::{Terminal, TerminalRecreateMode, TerminalTarget},
};
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Execute Sentinels
pub const START_OF_OUTPUT: &str = "__KOMODO_START_OF_OUTPUT__";
pub const END_OF_OUTPUT: &str = "__KOMODO_END_OF_OUTPUT__";

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<Terminal>)]
#[error(anyhow::Error)]
pub struct ListTerminals {
  /// Optionally restrict list to specific target.
  pub target: Option<TerminalTarget>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct CreateServerTerminal {
  /// A name for the terminal session
  pub name: String,
  /// The shell command (eg `bash`) to init the shell.
  ///
  /// This can also include args:
  /// `docker exec -it container sh`
  ///
  /// Default: Set in Periphery config.
  pub command: Option<String>,
  /// Specify the recreate behavior.
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct CreateContainerExecTerminal {
  /// A name for the terminal session
  pub name: String,
  /// The target for the terminal sessions (Container, Stack, Deployment).
  pub target: TerminalTarget,
  /// The name of the container to connect to
  pub container: String,
  /// The command to init shell inside container.
  /// Default: `sh`
  pub command: Option<String>,
  /// Specify the recreate behavior.
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct CreateContainerAttachTerminal {
  /// A name for the terminal session
  pub name: String,
  /// The target for the terminal sessions (Container, Stack, Deployment).
  pub target: TerminalTarget,
  /// The name of the container to attach to
  pub container: String,
  /// Specify the recreate behavior.
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Uuid)]
#[error(anyhow::Error)]
pub struct ConnectTerminal {
  /// The name of the terminal to connect to
  pub terminal: String,
  /// The target for the terminal session
  pub target: TerminalTarget,
}

//

/// Used to disconnect both Terminals and Container Exec sessions.
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct DisconnectTerminal {
  /// The channel id of the terminal to disconnect from
  pub channel: Uuid,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct DeleteTerminal {
  /// The name of the terminal to delete.
  pub terminal: String,
  /// The terminal target.
  pub target: TerminalTarget,
}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(anyhow::Error)]
pub struct DeleteAllTerminals {}

//

/// Note: The `terminal` must already exist, created by [CreateTerminal].
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Uuid)]
#[error(anyhow::Error)]
pub struct ExecuteTerminal {
  /// Specify the terminal to execute the command on.
  pub terminal: String,
  /// The terminal target.
  pub target: TerminalTarget,
  /// The command to execute.
  pub command: String,
}
