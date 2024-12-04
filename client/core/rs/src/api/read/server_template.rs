use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::server_template::{
  ServerTemplate, ServerTemplateListItem, ServerTemplateQuery,
};

use super::KomodoReadRequest;

//

/// Get a specific server template by id or name. Response: [ServerTemplate].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetServerTemplateResponse)]
#[error(serror::Error)]
pub struct GetServerTemplate {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server_template: String,
}

#[typeshare]
pub type GetServerTemplateResponse = ServerTemplate;

//

/// List server templates matching structured query. Response: [ListServerTemplatesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListServerTemplatesResponse)]
#[error(serror::Error)]
pub struct ListServerTemplates {
  #[serde(default)]
  pub query: ServerTemplateQuery,
}

#[typeshare]
pub type ListServerTemplatesResponse = Vec<ServerTemplateListItem>;

//

/// List server templates matching structured query. Response: [ListFullServerTemplatesResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullServerTemplatesResponse)]
#[error(serror::Error)]
pub struct ListFullServerTemplates {
  #[serde(default)]
  pub query: ServerTemplateQuery,
}

#[typeshare]
pub type ListFullServerTemplatesResponse = Vec<ServerTemplate>;

//

/// Gets a summary of data relating to all server templates.
/// Response: [GetServerTemplatesSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetServerTemplatesSummaryResponse)]
#[error(serror::Error)]
pub struct GetServerTemplatesSummary {}

/// Response for [GetServerTemplatesSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetServerTemplatesSummaryResponse {
  /// The total number of server templates.
  pub total: u32,
}
