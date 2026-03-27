use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::action::{
  Action, ActionActionState, ActionListItem, ActionQuery,
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetAction",
  description = "Get a specific action.",
  request_body(content = GetAction),
  responses(
    (status = 200, description = "The action", body = crate::entities::action::ActionSchema),
  ),
)]
pub fn get_action() {}

/// Get a specific action. Response: [Action].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionResponse)]
#[error(mogh_error::Error)]
pub struct GetAction {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type GetActionResponse = Action;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListActions",
  description = "List actions matching optional query.",
  request_body(content = ListActions),
  responses(
    (status = 200, description = "The list of actions", body = ListActionsResponse),
  ),
)]
pub fn list_actions() {}

/// List actions matching optional query. Response: [ListActionsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListActionsResponse)]
#[error(mogh_error::Error)]
pub struct ListActions {
  /// optional structured query to filter actions.
  #[serde(default)]
  pub query: ActionQuery,
}

#[typeshare]
pub type ListActionsResponse = Vec<ActionListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListFullActions",
  description = "List actions matching optional query.",
  request_body(content = ListFullActions),
  responses(
    (status = 200, description = "The list of actions", body = ListFullActionsResponse),
  ),
)]
pub fn list_full_actions() {}

/// List actions matching optional query. Response: [ListFullActionsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullActionsResponse)]
#[error(mogh_error::Error)]
pub struct ListFullActions {
  /// optional structured query to filter actions.
  #[serde(default)]
  pub query: ActionQuery,
}

#[typeshare]
pub type ListFullActionsResponse = Vec<Action>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetActionActionState",
  description = "Get current action state for the action.",
  request_body(content = GetActionActionState),
  responses(
    (status = 200, description = "The action action state", body = GetActionActionStateResponse),
  ),
)]
pub fn get_action_action_state() {}

/// Get current action state for the action. Response: [ActionActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionActionStateResponse)]
#[error(mogh_error::Error)]
pub struct GetActionActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type GetActionActionStateResponse = ActionActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetActionsSummary",
  description = "Gets a summary of data relating to all actions.",
  request_body(content = GetActionsSummary),
  responses(
    (status = 200, description = "The actions summary", body = GetActionsSummaryResponse),
  ),
)]
pub fn get_actions_summary() {}

/// Gets a summary of data relating to all actions.
/// Response: [GetActionsSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionsSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetActionsSummary {}

/// Response for [GetActionsSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetActionsSummaryResponse {
  /// The total number of actions.
  pub total: u32,
  /// The number of actions with Ok state.
  pub ok: u32,
  /// The number of actions currently running.
  pub running: u32,
  /// The number of actions with failed state.
  pub failed: u32,
  /// The number of actions with unknown state.
  pub unknown: u32,
}
