use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  procedure::{Procedure, _PartialProcedureConfig},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a procedure. Response: [Procedure].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateProcedureResponse)]
#[error(serror::Error)]
pub struct CreateProcedure {
  /// The name given to newly created build.
  pub name: String,
  /// Optional partial config to initialize the procedure with.
  #[serde(default)]
  pub config: _PartialProcedureConfig,
}

#[typeshare]
pub type CreateProcedureResponse = Procedure;

//

/// Creates a new procedure with given `name` and the configuration
/// of the procedure at the given `id`. Response: [Procedure].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CopyProcedureResponse)]
#[error(serror::Error)]
pub struct CopyProcedure {
  /// The name of the new procedure.
  pub name: String,
  /// The id of the procedure to copy.
  pub id: String,
}

#[typeshare]
pub type CopyProcedureResponse = Procedure;

//

/// Deletes the procedure at the given id, and returns the deleted procedure.
/// Response: [Procedure]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteProcedureResponse)]
#[error(serror::Error)]
pub struct DeleteProcedure {
  /// The id or name of the procedure to delete.
  pub id: String,
}

#[typeshare]
pub type DeleteProcedureResponse = Procedure;

//

/// Update the procedure at the given id, and return the updated procedure.
/// Response: [Procedure].
///
/// Note. This method updates only the fields which are set in the [_PartialProcedureConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateProcedureResponse)]
#[error(serror::Error)]
pub struct UpdateProcedure {
  /// The id of the procedure to update.
  pub id: String,
  /// The partial config update.
  pub config: _PartialProcedureConfig,
}

#[typeshare]
pub type UpdateProcedureResponse = Procedure;

//

/// Rename the Procedure at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
#[error(serror::Error)]
pub struct RenameProcedure {
  /// The id or name of the Procedure to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
