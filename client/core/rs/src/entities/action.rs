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
  entities::I64,
};

use super::resource::{Resource, ResourceListItem, ResourceQuery};

#[typeshare]
pub type ActionListItem = ResourceListItem<ActionListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionListItemInfo {
  /// Action last run timestamp in ms.
  pub last_run_at: I64,
  /// Whether last action run successful
  pub state: ActionState,
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, Display,
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
pub type Action = Resource<ActionConfig, ActionInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionInfo {
  /// When action was last run
  #[serde(default)]
  pub last_run_at: I64,
}

#[typeshare(serialized_as = "Partial<ActionConfig>")]
pub type _PartialActionConfig = PartialActionConfig;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct ActionConfig {
  /// Typescript file contents using pre-initialized `komodo` client.
  /// Supports variable / secret interpolation.
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_file_contents_deserializer"
  ))]
  #[builder(default)]
  pub file_contents: String,

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
      file_contents: Default::default(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
    }
  }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ActionActionState {
  /// Whether the action is currently running.
  pub running: bool,
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
