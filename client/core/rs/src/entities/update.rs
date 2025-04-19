use async_timing_util::unix_timestamp_ms;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use typeshare::typeshare;

use crate::entities::{
  I64, MongoId, Operation, all_logs_success, komodo_timestamp,
};

use super::{ResourceTarget, Version};

/// Represents an action performed by Komodo.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
#[cfg_attr(feature = "mongo", doc_index({ "target.type": 1 }))]
#[cfg_attr(feature = "mongo", sparse_doc_index({ "target.id": 1 }))]
pub struct Update {
  /// The Mongo ID of the update.
  /// This field is de/serialized from/to JSON as
  /// `{ "_id": { "$oid": "..." }, ...(rest of serialized Update) }`
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,

  /// The operation performed
  #[cfg_attr(feature = "mongo", index)]
  pub operation: Operation,

  /// The time the operation started
  #[cfg_attr(feature = "mongo", index)]
  pub start_ts: I64,

  /// Whether the operation was successful
  #[cfg_attr(feature = "mongo", index)]
  pub success: bool,

  /// The user id that triggered the update.
  ///
  /// Also can take these values for operations triggered automatically:
  /// - `Procedure`: The operation was triggered as part of a procedure run
  /// - `Github`: The operation was triggered by a github webhook
  /// - `Auto Redeploy`: The operation (always `Deploy`) was triggered by an attached build finishing.
  #[cfg_attr(feature = "mongo", index)]
  pub operator: String,

  /// The target resource to which this update refers
  pub target: ResourceTarget,
  /// Logs produced as the operation is performed
  pub logs: Vec<Log>,
  /// The time the operation completed.
  pub end_ts: Option<I64>,
  /// The status of the update
  /// - `Queued`
  /// - `InProgress`
  /// - `Complete`
  #[cfg_attr(feature = "mongo", index)]
  pub status: UpdateStatus,
  /// An optional version on the update, ie build version or deployed version.
  #[serde(default, skip_serializing_if = "Version::is_none")]
  pub version: Version,
  /// An optional commit hash associated with the update, ie cloned hash or deployed hash.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub commit_hash: String,
  /// Some unstructured, operation specific data. Not for general usage.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub other_data: String,
  /// If the update is for resource config update, give the previous toml contents
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub prev_toml: String,
  /// If the update is for resource config update, give the current (at time of Update) toml contents
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub current_toml: String,
}

impl Update {
  pub fn push_simple_log(
    &mut self,
    stage: &str,
    msg: impl Into<String>,
  ) {
    self.logs.push(Log::simple(stage, msg.into()));
  }

  pub fn push_error_log(
    &mut self,
    stage: &str,
    msg: impl Into<String>,
  ) {
    self.logs.push(Log::error(stage, msg.into()));
  }

  pub fn in_progress(&mut self) {
    self.status = UpdateStatus::InProgress;
  }

  pub fn finalize(&mut self) {
    self.success = all_logs_success(&self.logs);
    self.end_ts = Some(komodo_timestamp());
    self.status = UpdateStatus::Complete;
  }
}

/// Minimal representation of an action performed by Komodo.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateListItem {
  /// The id of the update
  pub id: String,
  /// Which operation was run
  pub operation: Operation,
  /// The starting time of the operation
  pub start_ts: I64,
  /// Whether the operation was successful
  pub success: bool,
  /// The username of the user performing update
  pub username: String,
  /// The user id that triggered the update.
  ///
  /// Also can take these values for operations triggered automatically:
  /// - `Procedure`: The operation was triggered as part of a procedure run
  /// - `Github`: The operation was triggered by a github webhook
  /// - `Auto Redeploy`: The operation (always `Deploy`) was triggered by an attached build finishing.
  pub operator: String,
  /// The target resource to which this update refers
  pub target: ResourceTarget,
  /// The status of the update
  /// - `Queued`
  /// - `InProgress`
  /// - `Complete`
  pub status: UpdateStatus,
  /// An optional version on the update, ie build version or deployed version.
  #[serde(default, skip_serializing_if = "Version::is_none")]
  pub version: Version,
  /// Some unstructured, operation specific data. Not for general usage.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub other_data: String,
}

/// Represents the output of some command being run
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Log {
  /// A label for the log
  pub stage: String,
  /// The command which was executed
  pub command: String,
  /// The output of the command in the standard channel
  pub stdout: String,
  /// The output of the command in the error channel
  pub stderr: String,
  /// Whether the command run was successful
  pub success: bool,
  /// The start time of the command execution
  pub start_ts: I64,
  /// The end time of the command execution
  pub end_ts: I64,
}

impl Log {
  pub fn simple(stage: &str, msg: String) -> Log {
    let ts = unix_timestamp_ms() as i64;
    Log {
      stage: stage.to_string(),
      stdout: msg,
      success: true,
      start_ts: ts,
      end_ts: ts,
      ..Default::default()
    }
  }

  pub fn error(stage: &str, msg: String) -> Log {
    let ts = unix_timestamp_ms() as i64;
    Log {
      stage: stage.to_string(),
      stderr: msg,
      start_ts: ts,
      end_ts: ts,
      success: false,
      ..Default::default()
    }
  }

  /// Combines stdout / stderr into one log
  pub fn combined(&self) -> String {
    match (self.stdout.is_empty(), self.stderr.is_empty()) {
      (true, true) => {
        format!("stdout: {}\n\nstderr: {}", self.stdout, self.stderr)
      }
      (true, false) => self.stdout.to_string(),
      (false, true) => self.stderr.to_string(),
      (false, false) => String::from("No log"),
    }
  }
}

/// An update's status
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Display,
  EnumString,
  PartialEq,
  Hash,
  Eq,
  Clone,
  Copy,
  Default,
)]
pub enum UpdateStatus {
  /// The run is in the system but hasn't started yet
  Queued,
  /// The run is currently running
  InProgress,
  /// The run is complete
  #[default]
  Complete,
}
