use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  server::ServerQuery,
  terminal::{
    ContainerTerminalMode, TerminalRecreateMode, TerminalTarget,
  },
};

use super::KomodoWriteRequest;

//

/// Create a Terminal.
/// Requires minimum Read + Terminal permission on the target Resource.
/// Response: [NoData]
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct CreateTerminal {
  /// A name for the Terminal session.
  pub name: String,
  /// The target to create terminal for
  pub target: TerminalTarget,
  /// The shell command (eg `bash`) to init the shell.
  ///
  /// Default:
  ///  - Server: Configured on each Periphery
  ///  - ContainerExec: `sh`
  ///  - Attach: unused
  pub command: Option<String>,
  /// For container terminals, choose 'exec' or 'attach'.
  ///
  /// Default
  ///  - Server: ignored
  ///  - Container / Stack / Deployment: `exec`
  pub mode: Option<ContainerTerminalMode>,
  /// Default: `Never`
  #[serde(default)]
  pub recreate: TerminalRecreateMode,
}

//

/// Delete a terminal.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteTerminal {
  /// Server / Container / Stack / Deployment
  pub target: TerminalTarget,
  /// The name of the Terminal to delete.
  pub terminal: String,
}

//

/// Delete all Terminals on the Server.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteAllTerminals {
  /// Server Id or name
  pub server: String,
}

//

/// Delete all terminals on many or all Servers.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct BatchDeleteAllTerminals {
  /// Optional structured query to filter servers.
  #[serde(default)]
  pub query: ServerQuery,
}
