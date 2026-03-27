use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::variable::Variable;

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateVariable",
  description = "**Admin only.** Create variable.",
  request_body(content = CreateVariable),
  responses(
    (status = 200, description = "The created variable", body = CreateVariableResponse),
  ),
)]
pub fn create_variable() {}

/// **Admin only.** Create variable. Response: [Variable].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateVariableResponse)]
#[error(mogh_error::Error)]
pub struct CreateVariable {
  /// The name of the variable to create.
  pub name: String,
  /// The initial value of the variable. defualt: "".
  #[serde(default)]
  pub value: String,
  /// The initial value of the description. default: "".
  #[serde(default)]
  pub description: String,
  /// Whether to make this a secret variable.
  #[serde(default)]
  pub is_secret: bool,
}

#[typeshare]
pub type CreateVariableResponse = Variable;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateVariableValue",
  description = "**Admin only.** Update variable value.",
  request_body(content = UpdateVariableValue),
  responses(
    (status = 200, description = "The updated variable", body = UpdateVariableValueResponse),
  ),
)]
pub fn update_variable_value() {}

/// **Admin only.** Update variable value. Response: [Variable].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableValueResponse)]
#[error(mogh_error::Error)]
pub struct UpdateVariableValue {
  /// The name of the variable to update.
  pub name: String,
  /// The value to set.
  pub value: String,
}

#[typeshare]
pub type UpdateVariableValueResponse = Variable;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateVariableDescription",
  description = "**Admin only.** Update variable description.",
  request_body(content = UpdateVariableDescription),
  responses(
    (status = 200, description = "The updated variable", body = UpdateVariableDescriptionResponse),
  ),
)]
pub fn update_variable_description() {}

/// **Admin only.** Update variable description. Response: [Variable].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableDescriptionResponse)]
#[error(mogh_error::Error)]
pub struct UpdateVariableDescription {
  /// The name of the variable to update.
  pub name: String,
  /// The description to set.
  pub description: String,
}

#[typeshare]
pub type UpdateVariableDescriptionResponse = Variable;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateVariableIsSecret",
  description = "**Admin only.** Update whether variable is secret.",
  request_body(content = UpdateVariableIsSecret),
  responses(
    (status = 200, description = "The updated variable", body = UpdateVariableIsSecretResponse),
  ),
)]
pub fn update_variable_is_secret() {}

/// **Admin only.** Update whether variable is secret. Response: [Variable].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableIsSecretResponse)]
#[error(mogh_error::Error)]
pub struct UpdateVariableIsSecret {
  /// The name of the variable to update.
  pub name: String,
  /// Whether variable is secret.
  pub is_secret: bool,
}

#[typeshare]
pub type UpdateVariableIsSecretResponse = Variable;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteVariable",
  description = "**Admin only.** Delete a variable.",
  request_body(content = DeleteVariable),
  responses(
    (status = 200, description = "The deleted variable", body = DeleteVariableResponse),
  ),
)]
pub fn delete_variable() {}

/// **Admin only.** Delete a variable. Response: [Variable].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteVariableResponse)]
#[error(mogh_error::Error)]
pub struct DeleteVariable {
  pub name: String,
}

#[typeshare]
pub type DeleteVariableResponse = Variable;
