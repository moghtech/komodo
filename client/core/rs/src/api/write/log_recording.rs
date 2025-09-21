use crate::entities::{
  log_recording::LogRecording,
  NoData, ResourceTarget,
};
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::KomodoWriteRequest;

//=====================
// StartLogRecording
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(StartLogRecordingResponse)]
#[error(serror::Error)]
pub struct StartLogRecording {
  pub target: ResourceTarget,
  /// Optional custom name for the recording
  pub name: Option<String>,
  /// Duration in days. None means until manually stopped.
  pub duration_days: Option<f64>,
  /// For stacks: which services to record (empty = all)
  #[serde(default)]
  pub services: Vec<String>,
}

#[typeshare]
pub type StartLogRecordingResponse = LogRecording;


//=====================
// StopLogRecording
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(StopLogRecordingResponse)]
#[error(serror::Error)]
pub struct StopLogRecording {
  pub id: String,
}

#[typeshare]
pub type StopLogRecordingResponse = LogRecording;


//=====================
// DeleteLogRecording
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteLogRecordingResponse)]
#[error(serror::Error)]
pub struct DeleteLogRecording {
  pub id: String,
}

#[typeshare]
pub type DeleteLogRecordingResponse = NoData;

