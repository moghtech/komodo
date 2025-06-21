use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{NoData, ResourceTarget};

use super::KomodoWriteRequest;

/// Update a resources common meta fields.
/// - description
/// - template
/// - tags
/// Response: [NoData].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateResourceMetaResponse)]
#[error(serror::Error)]
pub struct UpdateResourceMeta {
  /// The target resource to set update meta.
  pub target: ResourceTarget,
  /// New description to set,
  /// or null for no update
  pub description: Option<String>,
  /// New template value (true or false),
  /// or null for no update
  pub template: Option<bool>,
  /// The exact tags to set,
  /// or null for no update
  pub tags: Option<Vec<String>>,
}

#[typeshare]
pub type UpdateResourceMetaResponse = NoData;
