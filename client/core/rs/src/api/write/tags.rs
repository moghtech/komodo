use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::tag::{Tag, TagColor};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateTag",
  description = "Create a tag.",
  request_body(content = CreateTag),
  responses(
    (status = 200, description = "The created tag", body = Tag),
  ),
)]
pub fn create_tag() {}

/// Create a tag. Response: [Tag].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Tag)]
#[error(mogh_error::Error)]
pub struct CreateTag {
  /// The name of the tag.
  pub name: String,
  /// Tag color. Default: Slate.
  pub color: Option<TagColor>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteTag",
  description = "Delete a tag.",
  request_body(content = DeleteTag),
  responses(
    (status = 200, description = "The deleted tag", body = Tag),
  ),
)]
pub fn delete_tag() {}

/// Delete a tag, and return the deleted tag. Response: [Tag].
///
/// Note. Will also remove this tag from all attached resources.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Tag)]
#[error(mogh_error::Error)]
pub struct DeleteTag {
  /// The id of the tag to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameTag",
  description = "Rename a tag.",
  request_body(content = RenameTag),
  responses(
    (status = 200, description = "The renamed tag", body = Tag),
  ),
)]
pub fn rename_tag() {}

/// Rename a tag at id. Response: [Tag].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Tag)]
#[error(mogh_error::Error)]
pub struct RenameTag {
  /// The id of the tag to rename.
  pub id: String,
  /// The new name of the tag.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateTagColor",
  description = "Update tag color.",
  request_body(content = UpdateTagColor),
  responses(
    (status = 200, description = "The updated tag", body = Tag),
  ),
)]
pub fn update_tag_color() {}

/// Update color for tag. Response: [Tag].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Tag)]
#[error(mogh_error::Error)]
pub struct UpdateTagColor {
  /// The name or id of the tag to update.
  pub tag: String,
  /// The new color for the tag.
  pub color: TagColor,
}
