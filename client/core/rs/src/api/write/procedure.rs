use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  procedure::{_PartialProcedureConfig, Procedure},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateProcedure",
  description = "Create a procedure.",
  request_body(content = CreateProcedure),
  responses(
    (status = 200, description = "The new procedure", body = crate::entities::procedure::ProcedureSchema),
  ),
)]
pub fn create_procedure() {}

/// Create a procedure. Response: [Procedure].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateProcedureResponse)]
#[error(mogh_error::Error)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CopyProcedureResponse)]
#[error(mogh_error::Error)]
pub struct CopyProcedure {
  /// The name of the new procedure.
  pub name: String,
  /// The id of the procedure to copy.
  pub id: String,
}

#[typeshare]
pub type CopyProcedureResponse = Procedure;

//
//
#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyProcedure",
  description = "Copy a procedure.",
  request_body(content = CopyProcedure),
  responses(
    (status = 200, description = "The new procedure", body = crate::entities::procedure::ProcedureSchema),
  ),
)]
pub fn copy_procedure() {}

//
#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteProcedure",
  description = "Delete a procedure.",
  request_body(content = DeleteProcedure),
  responses(
    (status = 200, description = "The deleted procedure", body = crate::entities::procedure::ProcedureSchema),
  ),
)]
pub fn delete_procedure() {}

/// Deletes the procedure at the given id, and returns the deleted procedure.
/// Response: [Procedure]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteProcedureResponse)]
#[error(mogh_error::Error)]
pub struct DeleteProcedure {
  /// The id or name of the procedure to delete.
  pub id: String,
}

#[typeshare]
pub type DeleteProcedureResponse = Procedure;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateProcedure",
  description = "Update a procedure.",
  request_body(content = UpdateProcedure),
  responses(
    (status = 200, description = "The updated procedure", body = crate::entities::procedure::ProcedureSchema),
  ),
)]
pub fn update_procedure() {}

/// Update the procedure at the given id, and return the updated procedure.
/// Response: [Procedure].
///
/// Note. This method updates only the fields which are set in the [_PartialProcedureConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateProcedureResponse)]
#[error(mogh_error::Error)]
pub struct UpdateProcedure {
  /// The id of the procedure to update.
  pub id: String,
  /// The partial config update.
  pub config: _PartialProcedureConfig,
}

#[typeshare]
pub type UpdateProcedureResponse = Procedure;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameProcedure",
  description = "Rename a procedure.",
  request_body(content = RenameProcedure),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_procedure() {}

/// Rename the Procedure at id to the given name.
/// Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameProcedure {
  /// The id or name of the Procedure to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
