use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  deployment::{_PartialDeploymentConfig, Deployment},
  update::Update,
};

use super::KomodoWriteRequest;

//

/// Create a deployment. Response: [Deployment].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(serror::Error)]
pub struct CreateDeployment {
  /// The name given to newly created deployment.
  pub name: String,
  /// Optional partial config to initialize the deployment with.
  #[serde(default)]
  pub config: _PartialDeploymentConfig,
}

//

/// Creates a new deployment with given `name` and the configuration
/// of the deployment at the given `id`. Response: [Deployment]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(serror::Error)]
pub struct CopyDeployment {
  /// The name of the new deployment.
  pub name: String,
  /// The id of the deployment to copy.
  pub id: String,
}

//

/// Create a Deployment from an existing container. Response: [Deployment].
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(serror::Error)]
pub struct CreateDeploymentFromContainer {
  /// The name or id of the existing container.
  pub name: String,
  /// The server id or name on which container exists.
  pub server: String,
}

//

/// Deletes the deployment at the given id, and returns the deleted deployment.
/// Response: [Deployment].
///
/// Note. If the associated container is running, it will be deleted as part of
/// the deployment clean up.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(serror::Error)]
pub struct DeleteDeployment {
  /// The id or name of the deployment to delete.
  pub id: String,
}

//

/// Update the deployment at the given id, and return the updated deployment.
/// Response: [Deployment].
///
/// Note. If the attached server for the deployment changes,
/// the deployment will be deleted / cleaned up on the old server.
///
/// Note. This method updates only the fields which are set in the [_PartialDeploymentConfig],
/// effectively merging diffs into the final document.
/// This is helpful when multiple users are using
/// the same resources concurrently by ensuring no unintentional
/// field changes occur from out of date local state.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(serror::Error)]
pub struct UpdateDeployment {
  /// The deployment id to update.
  pub id: String,
  /// The partial config update.
  pub config: _PartialDeploymentConfig,
}

//

/// Rename the deployment at id to the given name. Response: [Update].
///
/// Note. If a container is created for the deployment, it will be renamed using
/// `docker rename ...`.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EmptyTraits,
)]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct RenameDeployment {
  /// The id of the deployment to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}
