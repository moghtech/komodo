use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, docker::HealthConfig};

use super::GraphDriverData;

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Image {
  /// ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image.
  #[serde(rename = "Id")]
  pub id: Option<String>,

  /// Descriptor is an OCI descriptor of the image target. In case of a multi-platform image, this descriptor points to the OCI index or a manifest list.  This field is only present if the daemon provides a multi-platform image store.  WARNING: This is experimental and may change at any time without any backward compatibility.
  #[serde(rename = "Descriptor")]
  pub descriptor: Option<OciDescriptor>,

  /// Manifests is a list of image manifests available in this image. It provides a more detailed view of the platform-specific image manifests or other image-attached data like build attestations.  Only available if the daemon provides a multi-platform image store and the `manifests` option is set in the inspect request.  WARNING: This is experimental and may change at any time without any backward compatibility.
  #[serde(rename = "Manifests")]
  pub manifests: Option<Vec<ImageManifestSummary>>,

  /// List of image names/tags in the local image cache that reference this image.  Multiple image tags can refer to the same image, and this list may be empty if no tags reference the image, in which case the image is \"untagged\", in which case it can still be referenced by its ID.
  #[serde(rename = "RepoTags")]
  pub repo_tags: Option<Vec<String>>,

  /// List of content-addressable digests of locally available image manifests that the image is referenced from. Multiple manifests can refer to the same image.  These digests are usually only available if the image was either pulled from a registry, or if the image was pushed to a registry, which is when the manifest is generated and its digest calculated.
  #[serde(rename = "RepoDigests")]
  pub repo_digests: Option<Vec<String>>,

  /// Optional message that was set when committing or importing the image.
  #[serde(rename = "Comment")]
  pub comment: Option<String>,

  /// Date and time at which the image was created, formatted in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if present in the image, and omitted otherwise.
  #[serde(rename = "Created")]
  pub created: Option<String>,

  /// Name of the author that was specified when committing the image, or as specified through MAINTAINER (deprecated) in the Dockerfile.
  #[serde(rename = "Author")]
  pub author: Option<String>,

  #[serde(rename = "Config")]
  pub config: Option<ImageConfig>,

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

/// A descriptor struct containing digest, media type, and size, as defined in the [OCI Content Descriptors Specification](https://github.com/opencontainers/image-spec/blob/v1.0.1/descriptor.md).
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OciDescriptor {
  /// The media type of the object this schema refers to.
  #[serde(rename = "mediaType")]
  pub media_type: Option<String>,

  /// The digest of the targeted content.
  #[serde(rename = "digest")]
  pub digest: Option<String>,

  /// The size in bytes of the blob.
  #[serde(rename = "size")]
  pub size: Option<I64>,

  /// List of URLs from which this object MAY be downloaded.
  #[serde(rename = "urls")]
  pub urls: Option<Vec<String>>,

  /// Arbitrary metadata relating to the targeted content.
  #[serde(rename = "annotations")]
  pub annotations: Option<HashMap<String, String>>,

  /// Data is an embedding of the targeted content. This is encoded as a base64 string when marshalled to JSON (automatically, by encoding/json). If present, Data can be used directly to avoid fetching the targeted content.
  #[serde(rename = "data")]
  pub data: Option<String>,

  #[serde(rename = "platform")]
  pub platform: Option<OciPlatform>,

  /// ArtifactType is the IANA media type of this artifact.
  #[serde(rename = "artifactType")]
  pub artifact_type: Option<String>,
}

/// Describes the platform which the image in the manifest runs on, as defined in the [OCI Image Index Specification](https://github.com/opencontainers/image-spec/blob/v1.0.1/image-index.md).
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OciPlatform {
  /// The CPU architecture, for example `amd64` or `ppc64`.
  #[serde(rename = "architecture")]
  pub architecture: Option<String>,

  /// The operating system, for example `linux` or `windows`.
  #[serde(rename = "os")]
  pub os: Option<String>,

  /// Optional field specifying the operating system version, for example on Windows `10.0.19041.1165`.
  pub os_version: Option<String>,

  /// Optional field specifying an array of strings, each listing a required OS feature (for example on Windows `win32k`).
  pub os_features: Option<Vec<String>>,

  /// Optional field specifying a variant of the CPU, for example `v7` to specify ARMv7 when architecture is `arm`.
  #[serde(rename = "variant")]
  pub variant: Option<String>,
}

/// ImageManifestSummary represents a summary of an image manifest.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageManifestSummary {
  /// ID is the content-addressable ID of an image and is the same as the digest of the image manifest.
  #[serde(rename = "ID")]
  pub id: String,

  #[serde(rename = "Descriptor")]
  pub descriptor: OciDescriptor,

  /// Indicates whether all the child content (image config, layers) is fully available locally.
  #[serde(rename = "Available")]
  pub available: bool,

  #[serde(rename = "Size")]
  pub size: ImageManifestSummarySize,

  /// The kind of the manifest.  kind         | description -------------|----------------------------------------------------------- image        | Image manifest that can be used to start a container. attestation  | Attestation manifest produced by the Buildkit builder for a specific image manifest.
  #[serde(rename = "Kind")]
  pub kind: Option<ImageManifestSummaryKindEnum>,

  #[serde(rename = "ImageData")]
  pub image_data: Option<ImageManifestSummaryImageData>,

  #[serde(rename = "AttestationData")]
  pub attestation_data: Option<ImageManifestSummaryAttestationData>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageManifestSummarySize {
  /// Total is the total size (in bytes) of all the locally present data (both distributable and non-distributable) that's related to this manifest and its children. This equal to the sum of [Content] size AND all the sizes in the [Size] struct present in the Kind-specific data struct. For example, for an image kind (Kind == \"image\") this would include the size of the image content and unpacked image snapshots ([Size.Content] + [ImageData.Size.Unpacked]).
  #[serde(rename = "Total")]
  pub total: I64,

  /// Content is the size (in bytes) of all the locally present content in the content store (e.g. image config, layers) referenced by this manifest and its children. This only includes blobs in the content store.
  #[serde(rename = "Content")]
  pub content: I64,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  Default,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum ImageManifestSummaryKindEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "image")]
  Image,
  #[serde(rename = "attestation")]
  Attestation,
  #[serde(rename = "unknown")]
  Unknown,
}

/// The image data for the image manifest. This field is only populated when Kind is \"image\".
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageManifestSummaryImageData {
  /// OCI platform of the image. This will be the platform specified in the manifest descriptor from the index/manifest list. If it's not available, it will be obtained from the image config.
  #[serde(rename = "Platform")]
  pub platform: OciPlatform,

  /// The IDs of the containers that are using this image.
  #[serde(rename = "Containers")]
  pub containers: Vec<String>,

  #[serde(rename = "Size")]
  pub size: ImageManifestSummaryImageDataSize,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageManifestSummaryImageDataSize {
  /// Unpacked is the size (in bytes) of the locally unpacked (uncompressed) image content that's directly usable by the containers running this image. It's independent of the distributable content - e.g. the image might still have an unpacked data that's still used by some container even when the distributable/compressed content is already gone.
  #[serde(rename = "Unpacked")]
  pub unpacked: I64,
}

/// The image data for the attestation manifest. This field is only populated when Kind is \"attestation\".
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageManifestSummaryAttestationData {
  /// The digest of the image manifest that this attestation is for.
  #[serde(rename = "For")]
  pub _for: String,
}

/// Configuration of the image. These fields are used as defaults when starting a container from the image.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImageConfig {
  /// The user that commands are run as inside the container.
  #[serde(rename = "User")]
  pub user: Option<String>,

  /// An object mapping ports to an empty object in the form:  `{\"<port>/<tcp|udp|sctp>\": {}}`
  #[serde(rename = "ExposedPorts")]
  pub exposed_ports: Option<Vec<String>>,

  /// A list of environment variables to set inside the container in the form `[\"VAR=value\", ...]`. A variable without `=` is removed from the environment, rather than to have an empty value.
  #[serde(rename = "Env")]
  pub env: Option<Vec<String>>,

  /// Command to run specified as a string or an array of strings.
  #[serde(rename = "Cmd")]
  pub cmd: Option<Vec<String>>,

  #[serde(rename = "Healthcheck")]
  pub healthcheck: Option<HealthConfig>,

  /// Command is already escaped (Windows only)
  #[serde(rename = "ArgsEscaped")]
  pub args_escaped: Option<bool>,

  /// An object mapping mount point paths inside the container to empty objects.
  #[serde(rename = "Volumes")]
  pub volumes: Option<Vec<String>>,

  /// The working directory for commands to run in.
  #[serde(rename = "WorkingDir")]
  pub working_dir: Option<String>,

  /// The entry point for the container as a string or an array of strings.  If the array consists of exactly one empty string (`[\"\"]`) then the entry point is reset to system default (i.e., the entry point used by docker when there is no `ENTRYPOINT` instruction in the `Dockerfile`).
  #[serde(rename = "Entrypoint")]
  pub entrypoint: Option<Vec<String>>,

  /// `ONBUILD` metadata that were defined in the image's `Dockerfile`.
  #[serde(rename = "OnBuild")]
  pub on_build: Option<Vec<String>>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  pub labels: Option<HashMap<String, String>>,

  /// Signal to stop a container as a string or unsigned integer.
  #[serde(rename = "StopSignal")]
  pub stop_signal: Option<String>,

  /// Shell for when `RUN`, `CMD`, and `ENTRYPOINT` uses a shell.
  #[serde(rename = "Shell")]
  pub shell: Option<Vec<String>>,
}

/// Information about the image's RootFS, including the layer IDs.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
