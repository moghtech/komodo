use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{I64, ResourceTarget, ScheduleFormat};

/// A scheduled Action / Procedure run.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Schedule {
  /// Procedure or Alerter
  pub target: ResourceTarget,
  /// Readable name of the target resource
  pub name: String,
  /// The format of the schedule expression
  pub schedule_format: ScheduleFormat,
  /// The schedule for the run
  pub schedule: String,
  /// Whether the scheduled run is enabled
  pub enabled: bool,
  /// Custom schedule timezone if it exists
  pub schedule_timezone: String,
  /// Last run timestamp in ms.
  pub last_run_at: Option<I64>,
  /// Next scheduled run time in unix ms.
  pub next_scheduled_run: Option<I64>,
  /// If there is an error parsing schedule expression,
  /// it will be given here.
  pub schedule_error: Option<String>,
  /// Resource tags.
  pub tags: Vec<String>,
}
