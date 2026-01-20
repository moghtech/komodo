use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{ResourceTargetVariant, update::Update};

use super::KomodoExecuteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RunSync",
  description = "Runs the target resource sync.",
  request_body(content = RunSync),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn run_sync() {}

/// Runs the target resource sync. Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
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
