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
  pub name: String,
}
