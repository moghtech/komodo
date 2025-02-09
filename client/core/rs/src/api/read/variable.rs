use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::variable::Variable;

use super::KomodoReadRequest;

/// List all available global variables.
/// Response: [Variable]
///
/// Note. For non admin users making this call,
/// secret variables will have their values obscured.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(GetVariableResponse)]
#[error(serror::Error)]
pub struct GetVariable {
  /// The name of the variable to get.
  pub name: String,
}

#[typeshare]
pub type GetVariableResponse = Variable;

//

/// List all available global variables.
/// Response: [ListVariablesResponse]
///
/// Note. For non admin users making this call,
/// secret variables will have their values obscured.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoReadRequest)]
#[response(ListVariablesResponse)]
#[error(serror::Error)]
pub struct ListVariables {}

#[typeshare]
pub type ListVariablesResponse = Vec<Variable>;
