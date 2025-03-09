use bson::Document;
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::api::execute::Execution;

use super::{
  I64,
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
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, Display,
)]
pub enum ProcedureState {
  /// Last run successful
  Ok,
  /// Last run failed
  Failed,
  /// Currently running
  Running,
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

fn default_webhook_enabled() -> bool {
  true
}

impl Default for ProcedureConfig {
  fn default() -> Self {
    Self {
      stages: Default::default(),
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
