use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    file_contents_deserializer, option_file_contents_deserializer,
  },
  entities::{FileFormat, I64, NoData},
};

use super::{
  ScheduleFormat,
  resource::{Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type ActionListItem = ResourceListItem<ActionListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionListItemInfo {
  /// Whether last action run successful
  pub state: ActionState,
  /// Action last successful run timestamp in ms.
  pub last_run_at: Option<I64>,
  /// If the action has schedule enabled, this is the
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
  Display,
  Serialize,
  Deserialize,
)]
pub enum ActionState {
  /// Unknown case
  #[default]
  Unknown,
  /// Last clone / pull successful (or never cloned)
  Ok,
  /// Last clone / pull failed
  Failed,
  /// Currently running
  Running,
}

#[typeshare]
pub type Action = Resource<ActionConfig, NoData>;

#[typeshare(serialized_as = "Partial<ActionConfig>")]
pub type _PartialActionConfig = PartialActionConfig;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct ActionConfig {
  /// Whether this action should run at startup.
  #[serde(default = "default_run_at_startup")]
  #[builder(default = "default_run_at_startup()")]
  #[partial_default(default_run_at_startup())]
  pub run_at_startup: bool,

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

  /// Whether to send alerts when this action fails.
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

  /// Whether deno will be instructed to reload all dependencies,
  /// this can usually be kept false outside of development.
  #[serde(default)]
  #[builder(default)]
  pub reload_deno_deps: bool,

  /// Typescript file contents using pre-initialized `komodo` client.
  /// Supports variable / secret interpolation.
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_file_contents_deserializer"
  ))]
  #[builder(default)]
  pub file_contents: String,

  /// Specify the format in which the arguments are defined.
  /// Default: `key_value` (like environment)
  #[serde(default)]
  #[builder(default)]
  pub arguments_format: FileFormat,

  /// Default arguments to give to the Action for use in the script at `ARGS`.
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_file_contents_deserializer"
  ))]
  #[builder(default)]
  pub arguments: String,
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

fn default_run_at_startup() -> bool {
  false
}

fn default_webhook_enabled() -> bool {
  true
}

impl ActionConfig {
  pub fn builder() -> ActionConfigBuilder {
    ActionConfigBuilder::default()
  }
}

impl Default for ActionConfig {
  fn default() -> Self {
    Self {
      schedule_format: Default::default(),
      schedule: Default::default(),
      schedule_enabled: default_schedule_enabled(),
      schedule_timezone: Default::default(),
      run_at_startup: default_run_at_startup(),
      schedule_alert: default_schedule_alert(),
      failure_alert: default_failure_alert(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
      reload_deno_deps: Default::default(),
      arguments_format: Default::default(),
      file_contents: Default::default(),
      arguments: Default::default(),
    }
  }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ActionActionState {
  /// Number of instances of the Action currently running
  pub running: u32,
}

#[typeshare]
pub type ActionQuery = ResourceQuery<ActionQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct ActionQuerySpecifics {}

impl super::resource::AddFilters for ActionQuerySpecifics {
  fn add_filters(&self, _filters: &mut Document) {}
}
