use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  server::{_PartialServerConfig, Server},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a server. Response: [Server].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(serror::Error)]
pub struct CreateServer {
  /// The name given to newly created server.
  pub name: String,
  /// Optional partial config to initialize the server with.
  #[serde(default)]
  pub config: _PartialServerConfig,
}

//

/// Creates a new server with given `name` and the configuration
/// of the server at the given `id`. Response: [Server].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(serror::Error)]
pub struct CopyServer {
  /// The name of the new server.
  pub name: String,
  /// The id of the server to copy.
  pub id: String,
}

//

/// Deletes the server at the given id, and returns the deleted server.
/// Response: [Server]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(serror::Error)]
pub struct DeleteServer {
  /// The id or name of the server to delete.
  pub id: String,
}

//

/// Update the server at the given id, and return the updated server.
/// Response: [Server].
///
/// Note. This method updates only the fields which are set in the [_PartialServerConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(serror::Error)]
pub struct UpdateServer {
  /// The id or name of the server to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialServerConfig,
}

//

/// Rename an Server to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameServer {
  /// The id or name of the Server to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

/// Create a docker network on the server.
/// Response: [Update]
///
/// `docker network create {name}`
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct CreateNetwork {
  /// Server Id or name
  pub server: String,
  /// The name of the network to create.
  pub name: String,
}

//

/// Configures the behavior of [CreateTerminal] if the
/// specified terminal name already exists.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub enum TerminalRecreateMode {
  /// Never kill the old terminal if it already exists.
  /// If the command is different, returns error.
  #[default]
  Never,
  /// Always kill the old terminal and create new one
  Always,
  /// Only kill and recreate if the command is different.
  DifferentCommand,
}

/// Create a terminal on the server.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct CreateTerminal {
  /// Server Id or name
  pub server: String,
  /// The name of the terminal on the server to create.
  pub name: String,
  /// The shell command (eg `bash`) to init the shell.
  ///
  /// This can also include args:
  /// `docker exec -it container sh`
  ///
  /// Default: `bash`
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

/// Delete a terminal on the server.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteTerminal {
  /// Server Id or name
  pub server: String,
  /// The name of the terminal on the server to delete.
  pub terminal: String,
}

/// Delete all terminals on the server.
/// Response: [NoData]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeleteAllTerminals {
  /// Server Id or name
  pub server: String,
}
