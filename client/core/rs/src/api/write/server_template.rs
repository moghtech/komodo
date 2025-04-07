use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  server_template::{PartialServerTemplateConfig, ServerTemplate},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a server template. Response: [ServerTemplate].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ServerTemplate)]
#[error(serror::Error)]
pub struct CreateServerTemplate {
  /// The name given to newly created server template.
  pub name: String,
  /// Optional partial config to initialize the server template with.
  #[serde(default)]
  pub config: PartialServerTemplateConfig,
}

//

/// Creates a new server template with given `name` and the configuration
/// of the server template at the given `id`. Response: [ServerTemplate]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ServerTemplate)]
#[error(serror::Error)]
pub struct CopyServerTemplate {
  /// The name of the new server template.
  pub name: String,
  /// The id of the server template to copy.
  pub id: String,
}

//

/// Deletes the server template at the given id, and returns the deleted server template.
/// Response: [ServerTemplate]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ServerTemplate)]
#[error(serror::Error)]
pub struct DeleteServerTemplate {
  /// The id or name of the server template to delete.
  pub id: String,
}

//

/// Update the server template at the given id, and return the updated server template.
/// Response: [ServerTemplate].
///
/// Note. This method updates only the fields which are set in the [PartialServerTemplateConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(ServerTemplate)]
#[error(serror::Error)]
pub struct UpdateServerTemplate {
  /// The id of the server template to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: PartialServerTemplateConfig,
}

//

/// Rename the ServerTemplate at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameServerTemplate {
  /// The id or name of the ServerTemplate to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
