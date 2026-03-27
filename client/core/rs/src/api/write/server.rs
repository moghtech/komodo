use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  server::{_PartialServerConfig, Server},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateServer",
  description = "Create a server.",
  request_body(content = CreateServer),
  responses(
    (status = 200, description = "The new server", body = crate::entities::server::ServerSchema),
  ),
)]
pub fn create_server() {}

/// Create a server. Response: [Server].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(mogh_error::Error)]
pub struct CreateServer {
  /// The name given to newly created server.
  pub name: String,
  /// Optional partial config to initialize the server with.
  #[serde(default)]
  pub config: _PartialServerConfig,
  /// Initial public key to assign to Server.
  pub public_key: Option<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyServer",
  description = "Copy a server.",
  request_body(content = CopyServer),
  responses(
    (status = 200, description = "The new server", body = crate::entities::server::ServerSchema),
  ),
)]
pub fn copy_server() {}

/// Creates a new server with given `name` and the configuration
/// of the server at the given `id`. Response: [Server].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(mogh_error::Error)]
pub struct CopyServer {
  /// The name of the new server.
  pub name: String,
  /// The id of the server to copy.
  pub id: String,
  /// Initial public key to assign to Server.
  pub public_key: Option<String>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteServer",
  description = "Delete a server.",
  request_body(content = DeleteServer),
  responses(
    (status = 200, description = "The deleted server", body = crate::entities::server::ServerSchema),
  ),
)]
pub fn delete_server() {}

/// Deletes the server at the given id, and returns the deleted server.
/// Response: [Server]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(mogh_error::Error)]
pub struct DeleteServer {
  /// The id or name of the server to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateServer",
  description = "Update a server.",
  request_body(content = UpdateServer),
  responses(
    (status = 200, description = "The updated server", body = crate::entities::server::ServerSchema),
  ),
)]
pub fn update_server() {}

/// Update the server at the given id, and return the updated server.
/// Response: [Server].
///
/// Note. This method updates only the fields which are set in the [_PartialServerConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Server)]
#[error(mogh_error::Error)]
pub struct UpdateServer {
  /// The id or name of the server to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialServerConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameServer",
  description = "Rename a server.",
  request_body(content = RenameServer),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_server() {}

/// Rename an Server to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameServer {
  /// The id or name of the Server to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateNetwork",
  description = "Create a docker network on the server.",
  request_body(content = CreateNetwork),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn create_network() {}

/// Create a docker network on the server.
/// Response: [Update]
///
/// `docker network create {name}`
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct CreateNetwork {
  /// Server Id or name
  pub server: String,
  /// The name of the network to create.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateServerPublicKey",
  description = "Updates the Server with an explicit Public Key.",
  request_body(content = UpdateServerPublicKey),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn update_server_public_key() {}

/// Updates the Server with an explicit Public Key.
/// Response: [Update]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct UpdateServerPublicKey {
  /// Server Id or name
  pub server: String,
  /// Spki base64 public key
  pub public_key: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RotateServerKeys",
  description = "Rotate server keys.",
  request_body(content = RotateServerKeys),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rotate_server_keys() {}

/// Rotates the private / public keys for the server.
/// Response: [Update]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RotateServerKeys {
  /// Server Id or name
  pub server: String,
}
