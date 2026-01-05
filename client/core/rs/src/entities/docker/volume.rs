use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::I64;

use super::{ObjectVersion, PortBinding};

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct VolumeListItem {
  /// The name of the volume
  pub name: String,
  pub driver: String,
  pub mountpoint: String,
  pub created: Option<String>,
  pub scope: VolumeScopeEnum,
  /// Amount of disk space used by the volume (in bytes). This information is only available for volumes created with the `\"local\"` volume driver. For volumes created with other volume drivers, this field is set to `-1` (\"not available\")
  pub size: Option<I64>,
  /// Whether the volume is currently attached to any container
  pub in_use: bool,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Volume {
  /// Name of the volume.
  #[serde(rename = "Name")]
  pub name: String,

  /// Name of the volume driver used by the volume.
  #[serde(rename = "Driver")]
  pub driver: String,

  /// Mount path of the volume on the host.
  #[serde(rename = "Mountpoint")]
  pub mountpoint: String,

  /// Date/Time the volume was created.
  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  /// Low-level details about the volume, provided by the volume driver. Details are returned as a map with key/value pairs: `{\"key\":\"value\",\"key2\":\"value2\"}`.  The `Status` field is optional, and is omitted if the volume driver does not support this feature.
  #[serde(default, rename = "Status")]
  pub status: HashMap<String, HashMap<String, ()>>,

  /// User-defined key/value metadata.
  #[serde(default, rename = "Labels")]
  pub labels: HashMap<String, String>,

  /// The level at which the volume exists. Either `global` for cluster-wide, or `local` for machine level.
  #[serde(default, rename = "Scope")]
  pub scope: VolumeScopeEnum,

  #[serde(rename = "ClusterVolume")]
  pub cluster_volume: Option<ClusterVolume>,

  /// The driver specific options used when creating the volume.
  #[serde(default, rename = "Options")]
  pub options: HashMap<String, String>,

  #[serde(rename = "UsageData")]
  pub usage_data: Option<VolumeUsageData>,
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
pub enum VolumeScopeEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "local")]
  Local,
  #[serde(rename = "global")]
  Global,
}

/// Options and information specific to, and only present on, Swarm CSI cluster volumes.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolume {
  /// The Swarm ID of this volume. Because cluster volumes are Swarm objects, they have an ID, unlike non-cluster volumes. This ID can be used to refer to the Volume instead of the name.
  #[serde(rename = "ID")]
  pub id: Option<String>,

  #[serde(rename = "Version")]
  pub version: Option<ObjectVersion>,

  #[serde(rename = "CreatedAt")]
  pub created_at: Option<String>,

  #[serde(rename = "UpdatedAt")]
  pub updated_at: Option<String>,

  #[serde(rename = "Spec")]
  pub spec: Option<ClusterVolumeSpec>,

  #[serde(rename = "Info")]
  pub info: Option<ClusterVolumeInfo>,

  /// The status of the volume as it pertains to its publishing and use on specific nodes
  #[serde(default, rename = "PublishStatus")]
  pub publish_status: Vec<ClusterVolumePublishStatus>,
}

/// Information about the global status of the volume.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeInfo {
  /// The capacity of the volume in bytes. A value of 0 indicates that the capacity is unknown.
  #[serde(rename = "CapacityBytes")]
  pub capacity_bytes: Option<I64>,

  /// A map of strings to strings returned from the storage plugin when the volume is created.
  #[serde(default, rename = "VolumeContext")]
  pub volume_context: HashMap<String, String>,

  /// The ID of the volume as returned by the CSI storage plugin. This is distinct from the volume's ID as provided by Docker. This ID is never used by the user when communicating with Docker to refer to this volume. If the ID is blank, then the Volume has not been successfully created in the plugin yet.
  #[serde(rename = "VolumeID")]
  pub volume_id: Option<String>,

  /// The topology this volume is actually accessible from.
  #[serde(default, rename = "AccessibleTopology")]
  pub accessible_topology: Vec<Topology>,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumePublishStatus {
  /// The ID of the Swarm node the volume is published on.
  #[serde(rename = "NodeID")]
  pub node_id: Option<String>,

  /// The published state of the volume. * `pending-publish` The volume should be published to this node, but the call to the controller plugin to do so has not yet been successfully completed. * `published` The volume is published successfully to the node. * `pending-node-unpublish` The volume should be unpublished from the node, and the manager is awaiting confirmation from the worker that it has done so. * `pending-controller-unpublish` The volume is successfully unpublished from the node, but has not yet been successfully unpublished on the controller.
  #[serde(default, rename = "State")]
  pub state: ClusterVolumePublishStatusStateEnum,

  /// A map of strings to strings returned by the CSI controller plugin when a volume is published.
  #[serde(default, rename = "PublishContext")]
  pub publish_context: HashMap<String, String>,
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
pub enum ClusterVolumePublishStatusStateEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "pending-publish")]
  PendingPublish,
  #[serde(rename = "published")]
  Published,
  #[serde(rename = "pending-node-unpublish")]
  PendingNodeUnpublish,
  #[serde(rename = "pending-controller-unpublish")]
  PendingControllerUnpublish,
}

/// Cluster-specific options used to create the volume.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeSpec {
  /// Group defines the volume group of this volume. Volumes belonging to the same group can be referred to by group name when creating Services.  Referring to a volume by group instructs Swarm to treat volumes in that group interchangeably for the purpose of scheduling. Volumes with an empty string for a group technically all belong to the same, emptystring group.
  #[serde(rename = "Group")]
  pub group: Option<String>,

  #[serde(rename = "AccessMode")]
  pub access_mode: Option<ClusterVolumeSpecAccessMode>,
}

/// Defines how the volume is used by tasks.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeSpecAccessMode {
  /// The set of nodes this volume can be used on at one time. - `single` The volume may only be scheduled to one node at a time. - `multi` the volume may be scheduled to any supported number of nodes at a time.
  #[serde(default, rename = "Scope")]
  pub scope: ClusterVolumeSpecAccessModeScopeEnum,

  /// The number and way that different tasks can use this volume at one time. - `none` The volume may only be used by one task at a time. - `readonly` The volume may be used by any number of tasks, but they all must mount the volume as readonly - `onewriter` The volume may be used by any number of tasks, but only one may mount it as read/write. - `all` The volume may have any number of readers and writers.
  #[serde(default, rename = "Sharing")]
  pub sharing: ClusterVolumeSpecAccessModeSharingEnum,

  /// Swarm Secrets that are passed to the CSI storage plugin when operating on this volume.
  #[serde(default, rename = "Secrets")]
  pub secrets: Vec<ClusterVolumeSpecAccessModeSecrets>,

  #[serde(rename = "AccessibilityRequirements")]
  pub accessibility_requirements:
    Option<ClusterVolumeSpecAccessModeAccessibilityRequirements>,

  #[serde(rename = "CapacityRange")]
  pub capacity_range:
    Option<ClusterVolumeSpecAccessModeCapacityRange>,

  /// The availability of the volume for use in tasks. - `active` The volume is fully available for scheduling on the cluster - `pause` No new workloads should use the volume, but existing workloads are not stopped. - `drain` All workloads using this volume should be stopped and rescheduled, and no new ones should be started.
  #[serde(default, rename = "Availability")]
  pub availability: ClusterVolumeSpecAccessModeAvailabilityEnum,
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
pub enum ClusterVolumeSpecAccessModeScopeEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "single")]
  Single,
  #[serde(rename = "multi")]
  Multi,
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
pub enum ClusterVolumeSpecAccessModeSharingEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "none")]
  None,
  #[serde(rename = "readonly")]
  Readonly,
  #[serde(rename = "onewriter")]
  Onewriter,
  #[serde(rename = "all")]
  All,
}

/// One cluster volume secret entry. Defines a key-value pair that is passed to the plugin.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeSpecAccessModeSecrets {
  /// Key is the name of the key of the key-value pair passed to the plugin.
  #[serde(rename = "Key")]
  pub key: Option<String>,

  /// Secret is the swarm Secret object from which to read data. This can be a Secret name or ID. The Secret data is retrieved by swarm and used as the value of the key-value pair passed to the plugin.
  #[serde(rename = "Secret")]
  pub secret: Option<String>,
}

/// Requirements for the accessible topology of the volume. These fields are optional. For an in-depth description of what these fields mean, see the CSI specification.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeSpecAccessModeAccessibilityRequirements {
  /// A list of required topologies, at least one of which the volume must be accessible from.
  #[serde(default, rename = "Requisite")]
  pub requisite: Vec<Topology>,

  /// A list of topologies that the volume should attempt to be provisioned in.
  #[serde(default, rename = "Preferred")]
  pub preferred: Vec<Topology>,
}

#[typeshare]
pub type Topology = HashMap<String, Vec<PortBinding>>;

/// The desired capacity that the volume should be created with. If empty, the plugin will decide the capacity.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ClusterVolumeSpecAccessModeCapacityRange {
  /// The volume must be at least this big. The value of 0 indicates an unspecified minimum
  #[serde(rename = "RequiredBytes")]
  pub required_bytes: Option<I64>,

  /// The volume must not be bigger than this. The value of 0 indicates an unspecified maximum.
  #[serde(rename = "LimitBytes")]
  pub limit_bytes: Option<I64>,
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
pub enum ClusterVolumeSpecAccessModeAvailabilityEnum {
  #[default]
  #[serde(rename = "")]
  Empty,
  #[serde(rename = "active")]
  Active,
  #[serde(rename = "pause")]
  Pause,
  #[serde(rename = "drain")]
  Drain,
}

/// Usage details about the volume. This information is used by the `GET /system/df` endpoint, and omitted in other endpoints.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct VolumeUsageData {
  /// Amount of disk space used by the volume (in bytes). This information is only available for volumes created with the `\"local\"` volume driver. For volumes created with other volume drivers, this field is set to `-1` (\"not available\")
  #[serde(rename = "Size")]
  pub size: I64,

  /// The number of containers referencing this volume. This field is set to `-1` if the reference-count is not available.
  #[serde(rename = "RefCount")]
  pub ref_count: I64,
}
