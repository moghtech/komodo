use crate::entities::{
  log_recording::{LogRecording, LogRecordingListItem},
  update::Log,
  ResourceTarget, U64,
};
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::KomodoReadRequest;

//=====================
// GetLogRecording
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetLogRecordingResponse)]
#[error(serror::Error)]
pub struct GetLogRecording {
  pub id: String,
}

#[typeshare]
pub type GetLogRecordingResponse = LogRecording;


//=====================
// ListLogRecordings
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListLogRecordingsResponse)]
#[error(serror::Error)]
pub struct ListLogRecordings {
  pub target: Option<ResourceTarget>,
  pub active_only: bool,
}

#[typeshare]
pub type ListLogRecordingsResponse = Vec<LogRecordingListItem>;


//=====================
// GetRecordedLogs
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetRecordedLogsResponse)]
#[error(serror::Error)]
pub struct GetRecordedLogs {
  pub recording_id: String,
  pub tail: U64,
  pub timestamps: bool,
}

#[typeshare]
pub type GetRecordedLogsResponse = Log;


//=====================
// SearchRecordedLogs
//=====================

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(SearchRecordedLogsResponse)]
#[error(serror::Error)]
pub struct SearchRecordedLogs {
  pub recording_id: String,
  pub terms: Vec<String>,
  pub combinator: crate::entities::SearchCombinator,
  pub invert: bool,
  pub timestamps: bool,
}

#[typeshare]
pub type SearchRecordedLogsResponse = Log;

