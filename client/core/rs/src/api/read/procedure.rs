use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::procedure::{
  Procedure, ProcedureActionState, ProcedureListItem, ProcedureQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetProcedure",
  description = "Get a specific procedure.",
  request_body(content = GetProcedure),
  responses(
    (status = 200, description = "The procedure", body = crate::entities::procedure::ProcedureSchema),
  ),
)]
pub fn get_procedure() {}

/// Get a specific procedure. Response: [Procedure].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetProcedureResponse)]
#[error(mogh_error::Error)]
pub struct GetProcedure {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub procedure: String,
}

#[typeshare]
pub type GetProcedureResponse = Procedure;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListProcedures",
  description = "List procedures matching optional query.",
  request_body(content = ListProcedures),
  responses(
    (status = 200, description = "The list of procedures", body = ListProceduresResponse),
  ),
)]
pub fn list_procedures() {}

/// List procedures matching optional query. Response: [ListProceduresResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListProceduresResponse)]
#[error(mogh_error::Error)]
pub struct ListProcedures {
  /// optional structured query to filter procedures.
  #[serde(default)]
  pub query: ProcedureQuery,
}

#[typeshare]
pub type ListProceduresResponse = Vec<ProcedureListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullProcedures",
  description = "List procedures matching optional query.",
  request_body(content = ListFullProcedures),
  responses(
    (status = 200, description = "The list of procedures", body = ListFullProceduresResponse),
  ),
)]
pub fn list_full_procedures() {}

/// List procedures matching optional query. Response: [ListFullProceduresResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullProceduresResponse)]
#[error(mogh_error::Error)]
pub struct ListFullProcedures {
  /// optional structured query to filter procedures.
  #[serde(default)]
  pub query: ProcedureQuery,
}

#[typeshare]
pub type ListFullProceduresResponse = Vec<Procedure>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetProcedureActionState",
  description = "Get current action state for the procedure.",
  request_body(content = GetProcedureActionState),
  responses(
    (status = 200, description = "The procedure action state", body = GetProcedureActionStateResponse),
  ),
)]
pub fn get_procedure_action_state() {}

/// Get current action state for the procedure. Response: [ProcedureActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetProcedureActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetProcedureActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub procedure: String,
}

#[typeshare]
pub type GetProcedureActionStateResponse = ProcedureActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetProceduresSummary",
  description = "Gets a summary of data relating to all procedures.",
  request_body(content = GetProceduresSummary),
  responses(
    (status = 200, description = "The procedures summary", body = GetProceduresSummaryResponse),
  ),
)]
pub fn get_procedures_summary() {}

/// Gets a summary of data relating to all procedures.
/// Response: [GetProceduresSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetProceduresSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetProceduresSummary {}

/// Response for [GetProceduresSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetProceduresSummaryResponse {
  /// The total number of procedures.
  pub total: u32,
  /// The number of procedures with Ok state.
  pub ok: u32,
  /// The number of procedures currently running.
  pub running: u32,
  /// The number of procedures with failed state.
  pub failed: u32,
  /// The number of procedures with unknown state.
  pub unknown: u32,
}
