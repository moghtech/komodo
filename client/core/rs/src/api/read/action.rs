use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::action::{
  Action, ActionActionState, ActionListItem, ActionQuery,
};

use super::KomodoReadRequest;

//

/// Get a specific action. Response: [Action].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionResponse)]
#[error(serror::Error)]
pub struct GetAction {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type GetActionResponse = Action;

//

/// List actions matching optional query. Response: [ListActionsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListActionsResponse)]
#[error(serror::Error)]
pub struct ListActions {
  /// optional structured query to filter actions.
  #[serde(default)]
  pub query: ActionQuery,
}

#[typeshare]
pub type ListActionsResponse = Vec<ActionListItem>;

//

/// List actions matching optional query. Response: [ListFullActionsResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullActionsResponse)]
#[error(serror::Error)]
pub struct ListFullActions {
  /// optional structured query to filter actions.
  #[serde(default)]
  pub query: ActionQuery,
}

#[typeshare]
pub type ListFullActionsResponse = Vec<Action>;

//

/// Get current action state for the action. Response: [ActionActionState].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionActionStateResponse)]
#[error(serror::Error)]
pub struct GetActionActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub action: String,
}

#[typeshare]
pub type GetActionActionStateResponse = ActionActionState;

//

/// Gets a summary of data relating to all actions.
/// Response: [GetActionsSummaryResponse].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetActionsSummaryResponse)]
#[error(serror::Error)]
pub struct GetActionsSummary {}

/// Response for [GetActionsSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
