use bson::Document;
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::api::execute::Execution;

use super::{
  I64, ScheduleFormat,
  resource::{Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type ProcedureListItem = ResourceListItem<ProcedureListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcedureListItemInfo {
  /// Number of stages procedure has.
  pub stages: I64,
  /// Reflect whether last run successful / currently running.
  pub state: ProcedureState,
  /// Procedure last successful run timestamp in ms.
  pub last_run_at: Option<I64>,
  /// If the procedure has schedule enabled, this is the
  /// next scheduled run time in unix ms.
  pub next_scheduled_run: Option<I64>,
  /// If there is an error parsing schedule expression,
  /// it will be given here.
  pub schedule_error: Option<String>,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
  Display,
)]
pub enum ProcedureState {
  /// Currently running
  Running,
  /// Last run successful
  Ok,
  /// Last run failed
  Failed,
  /// Other case (never run)
  #[default]
  Unknown,
}

/// Procedures run a series of stages sequentially, where
/// each stage runs executions in parallel.
#[typeshare]
pub type Procedure = Resource<ProcedureConfig, ()>;

#[typeshare(serialized_as = "Partial<ProcedureConfig>")]
pub type _PartialProcedureConfig = PartialProcedureConfig;

/// Config for the [Procedure]
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Partial, Builder)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct ProcedureConfig {
  /// The stages to be run by the procedure.
  #[serde(default, alias = "stage")]
  #[partial_attr(serde(alias = "stage"))]
  #[builder(default)]
  pub stages: Vec<ProcedureStage>,

  /// Choose whether to specify schedule as regular CRON, or using the english to CRON parser.
  #[serde(default)]
  #[builder(default)]
  pub schedule_format: ScheduleFormat,

  /// Optionally provide a schedule for the procedure to run on.
  ///
  /// There are 2 ways to specify a schedule:
  ///
  /// 1. Regular CRON expression:
  ///
  /// (second, minute, hour, day, month, day-of-week)
  /// ```text
  /// 0 0 0 1,15 * ?
  /// ```
  ///
  /// 2. "English" expression via [english-to-cron](https://crates.io/crates/english-to-cron):
  ///
  /// ```text
  /// at midnight on the 1st and 15th of the month
  /// ```
  #[serde(default)]
  #[builder(default)]
  pub schedule: String,

  /// Whether schedule is enabled if one is provided.
  /// Can be used to temporarily disable the schedule.
  #[serde(default = "default_schedule_enabled")]
  #[builder(default = "default_schedule_enabled()")]
  #[partial_default(default_schedule_enabled())]
  pub schedule_enabled: bool,

  /// Optional. A TZ Identifier. If not provided, will use Core local timezone.
  /// https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
  #[serde(default)]
  #[builder(default)]
  pub schedule_timezone: String,

  /// Whether to send alerts when the schedule was run.
  #[serde(default = "default_schedule_alert")]
  #[builder(default = "default_schedule_alert()")]
  #[partial_default(default_schedule_alert())]
  pub schedule_alert: bool,

  /// Whether to send alerts when this procedure fails.
  #[serde(default = "default_failure_alert")]
  #[builder(default = "default_failure_alert()")]
  #[partial_default(default_failure_alert())]
  pub failure_alert: bool,

  /// Whether incoming webhooks actually trigger action.
  #[serde(default = "default_webhook_enabled")]
  #[builder(default = "default_webhook_enabled()")]
  #[partial_default(default_webhook_enabled())]
  pub webhook_enabled: bool,

  /// Optionally provide an alternate webhook secret for this procedure.
  /// If its an empty string, use the default secret from the config.
  #[serde(default)]
  #[builder(default)]
  pub webhook_secret: String,
}

impl ProcedureConfig {
  pub fn builder() -> ProcedureConfigBuilder {
    ProcedureConfigBuilder::default()
  }
}

fn default_schedule_enabled() -> bool {
  true
}

fn default_schedule_alert() -> bool {
  true
}

fn default_failure_alert() -> bool {
  true
}

fn default_webhook_enabled() -> bool {
  true
}

impl Default for ProcedureConfig {
  fn default() -> Self {
    Self {
      stages: Default::default(),
      schedule_format: Default::default(),
      schedule: Default::default(),
      schedule_enabled: default_schedule_enabled(),
      schedule_timezone: Default::default(),
      schedule_alert: default_schedule_alert(),
      failure_alert: default_failure_alert(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
    }
  }
}

/// A single stage of a procedure. Runs a list of executions in parallel.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcedureStage {
  /// A name for the procedure
  pub name: String,
  /// Whether the stage should be run as part of the procedure.
  #[serde(default = "default_enabled")]
  pub enabled: bool,
  /// The executions in the stage
  #[serde(default, alias = "execution")]
  pub executions: Vec<EnabledExecution>,
}

/// Allows to enable / disabled procedures in the sequence / parallel vec on the fly
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnabledExecution {
  /// The execution request to run.
  pub execution: Execution,
  /// Whether the execution is enabled to run in the procedure.
  #[serde(default = "default_enabled")]
  pub enabled: bool,
}

fn default_enabled() -> bool {
  true
}

#[typeshare]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ProcedureActionState {
  pub running: bool,
}

// QUERY

#[typeshare]
pub type ProcedureQuery = ResourceQuery<ProcedureQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct ProcedureQuerySpecifics {}

impl super::resource::AddFilters for ProcedureQuerySpecifics {
  fn add_filters(&self, _: &mut Document) {}
}
