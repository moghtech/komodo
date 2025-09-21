use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use typeshare::typeshare;

use super::{I64, MongoId, ResourceTarget};

/// Represents a log recording session for a resource
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", doc_index({ "target.type": 1 }))]
#[cfg_attr(feature = "mongo", doc_index({ "target.id": 1 }))]
#[cfg_attr(feature = "mongo", index({ "user_id": 1 }))]
#[cfg_attr(feature = "mongo", index({ "status": 1 }))]
pub struct LogRecording {
  /// The Mongo ID of the recording
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,

  /// The target resource being recorded
  pub target: ResourceTarget,

  /// Optional custom name for the recording
  pub name: Option<String>,

  /// The user who started the recording
  pub user_id: String,

  /// When the recording started
  #[cfg_attr(feature = "mongo", index)]
  pub start_ts: I64,

  /// When the recording should expire (None = manual stop only)
  #[cfg_attr(feature = "mongo", index)]
  pub expire_ts: Option<I64>,

  /// Current status of the recording
  pub status: LogRecordingStatus,

  /// For stacks: which services to record (empty = all)
  #[serde(default)]
  pub services: Vec<String>,
}

/// Status of a log recording
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Display, EnumString, PartialEq, Eq,
)]
pub enum LogRecordingStatus {
  #[default]
  Recording,
  Stopped,
  Expired,
}

/// Minimal representation for listing
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogRecordingListItem {
  pub id: String,
  pub target: ResourceTarget,
  pub name: Option<String>,
  pub user_id: String,
  pub username: String,
  pub start_ts: I64,
  pub expire_ts: Option<I64>,
  pub status: LogRecordingStatus,
  pub services: Vec<String>,
}

/// A single log entry from a recording
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", index({ "recording_id": 1 }))]
#[cfg_attr(feature = "mongo", index({ "ts": 1 }))]
pub struct RecordedLog {
  /// The Mongo ID of the log entry
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,

  /// The recording this log belongs to
  pub recording_id: String,

  /// Timestamp of the log entry
  pub ts: I64,

  /// Standard output
  #[serde(default)]
  pub stdout: String,

  /// Standard error
  #[serde(default)]
  pub stderr: String,

  /// For stack services
  #[serde(skip_serializing_if = "Option::is_none")]
  pub service: Option<String>,
}