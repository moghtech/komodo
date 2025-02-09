use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{update::Update, ResourceTargetVariant};

use super::KomodoExecuteRequest;

/// Runs the target resource sync. Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RunSync {
  /// Id or name
  pub sync: String,
  /// Only execute sync on a specific resource type.
  /// Combine with `resource_id` to specify resource.
  pub resource_type: Option<ResourceTargetVariant>,
  /// Only execute sync on a specific resources.
  /// Combine with `resource_type` to specify resources.
  /// Supports name or id.
  pub resources: Option<Vec<String>>,
}
