use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  action::{_PartialActionConfig, Action},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a action. Response: [Action].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Action)]
#[error(serror::Error)]
pub struct CreateAction {
  /// The name given to newly created action.
  pub name: String,
  /// Optional partial config to initialize the action with.
  #[serde(default)]
  pub config: _PartialActionConfig,
}

//

/// Creates a new action with given `name` and the configuration
/// of the action at the given `id`. Response: [Action].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Action)]
#[error(serror::Error)]
pub struct CopyAction {
  /// The name of the new action.
  pub name: String,
  /// The id of the action to copy.
  pub id: String,
}

//

/// Deletes the action at the given id, and returns the deleted action.
/// Response: [Action]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Action)]
#[error(serror::Error)]
pub struct DeleteAction {
  /// The id or name of the action to delete.
  pub id: String,
}

//

/// Update the action at the given id, and return the updated action.
/// Response: [Action].
///
/// Note. This method updates only the fields which are set in the [_PartialActionConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Action)]
#[error(serror::Error)]
pub struct UpdateAction {
  /// The id of the action to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialActionConfig,
}

//

/// Rename the Action at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameAction {
  /// The id or name of the Action to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

/// Create a webhook on the github action attached to the Action resource.
/// passed in request. Response: [CreateActionWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateActionWebhookResponse)]
#[error(serror::Error)]
pub struct CreateActionWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type CreateActionWebhookResponse = NoData;

//

/// Delete the webhook on the github action attached to the Action resource.
/// passed in request. Response: [DeleteActionWebhookResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteActionWebhookResponse)]
#[error(serror::Error)]
pub struct DeleteActionWebhook {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type DeleteActionWebhookResponse = NoData;
