use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  NoData,
  stack::{_PartialStackConfig, Stack},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateStack",
  description = "Create a stack.",
  request_body(content = CreateStack),
  responses(
    (status = 200, description = "The created stack", body = crate::entities::stack::StackSchema),
  ),
)]
pub fn create_stack() {}

/// Create a stack. Response: [Stack].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Stack)]
#[error(mogh_error::Error)]
pub struct CreateStack {
  /// The name given to newly created stack.
  pub name: String,
  /// Optional partial config to initialize the stack with.
  #[serde(default)]
  pub config: _PartialStackConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyStack",
  description = "Copy a stack.",
  request_body(content = CopyStack),
  responses(
    (status = 200, description = "The new stack", body = crate::entities::stack::StackSchema),
  ),
)]
pub fn copy_stack() {}

/// Creates a new stack with given `name` and the configuration
/// of the stack at the given `id`. Response: [Stack].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Stack)]
#[error(mogh_error::Error)]
pub struct CopyStack {
  /// The name of the new stack.
  pub name: String,
  /// The id of the stack to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteStack",
  description = "Delete a stack.",
  request_body(content = DeleteStack),
  responses(
    (status = 200, description = "The deleted stack", body = crate::entities::stack::StackSchema),
  ),
)]
pub fn delete_stack() {}

/// Deletes the stack at the given id, and returns the deleted stack.
/// Response: [Stack]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Stack)]
#[error(mogh_error::Error)]
pub struct DeleteStack {
  /// The id or name of the stack to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateStack",
  description = "Update a stack.",
  request_body(content = UpdateStack),
  responses(
    (status = 200, description = "The updated stack", body = crate::entities::stack::StackSchema),
  ),
)]
pub fn update_stack() {}

/// Update the stack at the given id, and return the updated stack.
/// Response: [Stack].
///
/// Note. If the attached server for the stack changes,
/// the stack will be deleted / cleaned up on the old server.
///
/// Note. This method updates only the fields which are set in the [_PartialStackConfig],
/// merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Stack)]
#[error(mogh_error::Error)]
pub struct UpdateStack {
  /// The id of the Stack to update.
  pub id: String,
  /// The partial config update to apply.
  pub config: _PartialStackConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameStack",
  description = "Rename a stack.",
  request_body(content = RenameStack),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_stack() {}

/// Rename the stack at id to the given name. Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameStack {
  /// The id of the stack to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/WriteStackFileContents",
  description = "Write file contents to a stack.",
  request_body(content = WriteStackFileContents),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn write_stack_file_contents() {}

/// Update file contents in Files on Server or Git Repo mode. Response: [Update].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct WriteStackFileContents {
  /// The name or id of the target Stack.
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
  /// The file path relative to the stack run directory,
  /// or absolute path.
  pub file_path: String,
  /// The contents to write.
  pub contents: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RefreshStackCache",
  description = "Trigger a refresh of the cached compose file contents.",
  request_body(content = RefreshStackCache),
  responses(
    (status = 200, description = "No data", body = NoData),
  ),
)]
pub fn refresh_stack_cache() {}

/// Trigger a refresh of the cached compose file contents.
/// Refreshes:
///   - Whether the remote file is missing
///   - The latest json, and for repos, the remote contents, hash, and message.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(NoData)]
#[error(mogh_error::Error)]
pub struct RefreshStackCache {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub stack: String,
}

//

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum StackWebhookAction {
  Refresh,
  Deploy,
}
