use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  server::ServerQuery,
  terminal::{
    ContainerTerminalMode, Terminal, TerminalRecreateMode,
    TerminalTarget,
  },
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateTerminal",
  description = "Create a Terminal.",
  request_body(content = CreateTerminal),
  responses(
    (status = 200, description = "The created terminal", body = Terminal),
  ),
)]
pub fn create_terminal() {}

/// Create a Terminal.
/// Requires minimum Read + Terminal permission on the target Resource.
/// Response: [Terminal]
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Terminal)]
#[error(mogh_error::Error)]
pub struct CreateTerminal {
  /// A name for the Terminal session.
  /// If not specified, a default will be given.
  pub name: Option<String>,
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

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteTerminal",
  description = "Delete a terminal.",
  request_body(content = DeleteTerminal),
  responses(
    (status = 200, description = "No data", body = NoData),
  ),
)]
pub fn delete_terminal() {}

/// Delete a terminal.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct DeleteTerminal {
  /// Server / Container / Stack / Deployment
  pub target: TerminalTarget,
  /// The name of the Terminal to delete.
  pub terminal: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteAllTerminals",
  description = "Delete all Terminals on the Server.",
  request_body(content = DeleteAllTerminals),
  responses(
    (status = 200, description = "No data", body = NoData),
  ),
)]
pub fn delete_all_terminals() {}

/// Delete all Terminals on the Server.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct DeleteAllTerminals {
  /// Server Id or name
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchDeleteAllTerminals",
  description = "Delete all terminals on many or all Servers.",
  request_body(content = BatchDeleteAllTerminals),
  responses(
    (status = 200, description = "No data", body = NoData),
  ),
)]
pub fn batch_delete_all_terminals() {}

/// Delete all terminals on many or all Servers.
/// Response: [NoData]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct BatchDeleteAllTerminals {
  /// Optional structured query to filter servers.
  #[serde(default)]
  pub query: ServerQuery,
}
