use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::I64;

use super::{ContainerConfig, GraphDriverData};

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImageListItem {
  /// The first tag in `repo_tags`, or Id if no tags.
  pub name: String,
  /// ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image.
  pub id: String,
  /// ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry.
  pub parent_id: String,
  /// Date and time at which the image was created as a Unix timestamp (number of seconds sinds EPOCH).
  pub created: I64,
  /// Total size of the image including all layers it is composed of.
  pub size: I64,
  /// Whether the image is in use by any container
  pub in_use: bool,
}

/// Information about an image in the local image cache.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Image {
  /// ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image.
  #[serde(rename = "Id")]
  pub id: Option<String>,

  /// List of image names/tags in the local image cache that reference this image.  Multiple image tags can refer to the same image, and this list may be empty if no tags reference the image, in which case the image is \"untagged\", in which case it can still be referenced by its ID.
  #[serde(default, rename = "RepoTags")]
  pub repo_tags: Vec<String>,

  /// List of content-addressable digests of locally available image manifests that the image is referenced from. Multiple manifests can refer to the same image.  These digests are usually only available if the image was either pulled from a registry, or if the image was pushed to a registry, which is when the manifest is generated and its digest calculated.
  #[serde(default, rename = "RepoDigests")]
  pub repo_digests: Vec<String>,

  /// ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry.
  #[serde(rename = "Parent")]
  pub parent: Option<String>,

  /// Optional message that was set when committing or importing the image.
  #[serde(rename = "Comment")]
  pub comment: Option<String>,

  /// Date and time at which the image was created, formatted in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if present in the image, and omitted otherwise.
  #[serde(rename = "Created")]
  pub created: Option<String>,

  /// The version of Docker that was used to build the image.  Depending on how the image was created, this field may be empty.
  #[serde(rename = "DockerVersion")]
  pub docker_version: Option<String>,

  /// Name of the author that was specified when committing the image, or as specified through MAINTAINER (deprecated) in the Dockerfile.
  #[serde(rename = "Author")]
  pub author: Option<String>,

  /// Configuration for a container that is portable between hosts.
  #[serde(rename = "Config")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config: Option<ContainerConfig>,

  /// Hardware CPU architecture that the image runs on.
  #[serde(rename = "Architecture")]
  pub architecture: Option<String>,

  /// CPU architecture variant (presently ARM-only).
  #[serde(rename = "Variant")]
  pub variant: Option<String>,

  /// Operating System the image is built to run on.
  #[serde(rename = "Os")]
  pub os: Option<String>,

  /// Operating System version the image is built to run on (especially for Windows).
  #[serde(rename = "OsVersion")]
  pub os_version: Option<String>,

  /// Total size of the image including all layers it is composed of.
  #[serde(rename = "Size")]
  pub size: Option<I64>,

  #[serde(rename = "GraphDriver")]
  pub graph_driver: Option<GraphDriverData>,

  #[serde(rename = "RootFS")]
  pub root_fs: Option<ImageInspectRootFs>,

  #[serde(rename = "Metadata")]
  pub metadata: Option<ImageInspectMetadata>,
}

/// Information about the image's RootFS, including the layer IDs.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImageInspectRootFs {
  #[serde(default, rename = "Type")]
  pub typ: String,

  #[serde(default, rename = "Layers")]
  pub layers: Vec<String>,
}

/// Additional metadata of the image in the local cache. This information is local to the daemon, and not part of the image itself.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImageInspectMetadata {
  /// Date and time at which the image was last tagged in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if the image was tagged locally, and omitted otherwise.
  #[serde(rename = "LastTagTime")]
  pub last_tag_time: Option<String>,
}

/// individual image layer information in response to ImageHistory operation
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImageHistoryResponseItem {
  #[serde(rename = "Id")]
  pub id: String,

  #[serde(rename = "Created")]
  pub created: I64,

  #[serde(rename = "CreatedBy")]
  pub created_by: String,

  #[serde(default, rename = "Tags")]
  pub tags: Vec<String>,

  #[serde(rename = "Size")]
  pub size: I64,

  #[serde(rename = "Comment")]
  pub comment: String,
}
