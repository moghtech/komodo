use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::variable::Variable;

use super::KomodoWriteRequest;

/// **Admin only.** Create variable. Response: [Variable].
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(CreateVariableResponse)]
#[error(serror::Error)]
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

/// **Admin only.** Update variable value. Response: [Variable].
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableValueResponse)]
#[error(serror::Error)]
pub struct UpdateVariableValue {
  /// The name of the variable to update.
  pub name: String,
  /// The value to set.
  pub value: String,
}

#[typeshare]
pub type UpdateVariableValueResponse = Variable;

//

/// **Admin only.** Update variable description. Response: [Variable].
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableDescriptionResponse)]
#[error(serror::Error)]
pub struct UpdateVariableDescription {
  /// The name of the variable to update.
  pub name: String,
  /// The description to set.
  pub description: String,
}

#[typeshare]
pub type UpdateVariableDescriptionResponse = Variable;

//

/// **Admin only.** Update whether variable is secret. Response: [Variable].
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(UpdateVariableIsSecretResponse)]
#[error(serror::Error)]
pub struct UpdateVariableIsSecret {
  /// The name of the variable to update.
  pub name: String,
  /// Whether variable is secret.
  pub is_secret: bool,
}

#[typeshare]
pub type UpdateVariableIsSecretResponse = Variable;

//

/// **Admin only.** Delete a variable. Response: [Variable].
#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(DeleteVariableResponse)]
#[error(serror::Error)]
pub struct DeleteVariable {
  pub name: String,
}

#[typeshare]
pub type DeleteVariableResponse = Variable;
