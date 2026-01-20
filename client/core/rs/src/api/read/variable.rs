use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::variable::Variable;

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetVariable",
  description = "List all available global variables.",
  request_body(content = GetVariable),
  responses(
    (status = 200, description = "The variable", body = GetVariableResponse),
  ),
)]
pub fn get_variable() {}

/// List all available global variables.
/// Response: [Variable]
///
/// Note. For non admin users making this call,
/// secret variables will have their values obscured.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetVariableResponse)]
#[error(mogh_error::Error)]
pub struct GetVariable {
  /// The name of the variable to get.
  pub name: String,
}

#[typeshare]
pub type GetVariableResponse = Variable;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListVariables",
  description = "List all available global variables.",
  request_body(content = ListVariables),
  responses(
    (status = 200, description = "The list of variables", body = ListVariablesResponse),
  ),
)]
pub fn list_variables() {}

/// List all available global variables.
/// Response: [ListVariablesResponse]
///
/// Note. For non admin users making this call,
/// secret variables will have their values obscured.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListVariablesResponse)]
#[error(mogh_error::Error)]
pub struct ListVariables {}

#[typeshare]
pub type ListVariablesResponse = Vec<Variable>;
