use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  deployment::{_PartialDeploymentConfig, Deployment},
  update::Update,
};

use super::KomodoWriteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateDeployment",
  description = "Create a deployment.",
  request_body(content = CreateDeployment),
  responses(
    (status = 200, description = "The new deployment", body = crate::entities::deployment::DeploymentSchema),
  ),
)]
pub fn create_deployment() {}

/// Create a deployment. Response: [Deployment].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(mogh_error::Error)]
pub struct CreateDeployment {
  /// The name given to newly created deployment.
  pub name: String,
  /// Optional partial config to initialize the deployment with.
  #[serde(default)]
  pub config: _PartialDeploymentConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CopyDeployment",
  description = "Copy a deployment.",
  request_body(content = CopyDeployment),
  responses(
    (status = 200, description = "The new deployment", body = crate::entities::deployment::DeploymentSchema),
  ),
)]
pub fn copy_deployment() {}

/// Creates a new deployment with given `name` and the configuration
/// of the deployment at the given `id`. Response: [Deployment]
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(mogh_error::Error)]
pub struct CopyDeployment {
  /// The name of the new deployment.
  pub name: String,
  /// The id of the deployment to copy.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CreateDeploymentFromContainer",
  description = "Create a Deployment from an existing container.",
  request_body(content = CreateDeploymentFromContainer),
  responses(
    (status = 200, description = "The new deployment", body = crate::entities::deployment::DeploymentSchema),
  ),
)]
pub fn create_deployment_from_container() {}

/// Create a Deployment from an existing container. Response: [Deployment].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(mogh_error::Error)]
pub struct CreateDeploymentFromContainer {
  /// The name or id of the existing container.
  pub name: String,
  /// The server id or name on which container exists.
  pub server: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/DeleteDeployment",
  description = "Delete a deployment.",
  request_body(content = DeleteDeployment),
  responses(
    (status = 200, description = "The deleted deployment", body = crate::entities::deployment::DeploymentSchema),
  ),
)]
pub fn delete_deployment() {}

/// Deletes the deployment at the given id, and returns the deleted deployment.
/// Response: [Deployment].
///
/// Note. If the associated container is running, it will be deleted as part of
/// the deployment clean up.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(mogh_error::Error)]
pub struct DeleteDeployment {
  /// The id or name of the deployment to delete.
  pub id: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/UpdateDeployment",
  description = "Update a deployment.",
  request_body(content = UpdateDeployment),
  responses(
    (status = 200, description = "The updated deployment", body = crate::entities::deployment::DeploymentSchema),
  ),
)]
pub fn update_deployment() {}

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
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Deployment)]
#[error(mogh_error::Error)]
pub struct UpdateDeployment {
  /// The deployment id to update.
  pub id: String,
  /// The partial config update.
  pub config: _PartialDeploymentConfig,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/RenameDeployment",
  description = "Rename a deployment.",
  request_body(content = RenameDeployment),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn rename_deployment() {}

/// Rename the deployment at id to the given name. Response: [Update].
///
/// Note. If a container is created for the deployment, it will be renamed using
/// `docker rename ...`.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RenameDeployment {
  /// The id of the deployment to rename.
  pub id: String,
  /// The new name.
  pub name: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/CheckDeploymentForUpdate",
  description = "Checks for newer image than what is deployed.",
  request_body(content = CheckDeploymentForUpdate),
  responses(
    (status = 200, description = "Checked for update", body = CheckDeploymentForUpdateResponse),
  ),
)]
pub fn check_deployment_for_update() {}

/// Checks for newer image than what is deployed. Response: [CheckDeploymentForUpdateResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(CheckDeploymentForUpdateResponse)]
#[error(mogh_error::Error)]
pub struct CheckDeploymentForUpdate {
  /// Name or id
  pub deployment: String,
  /// Normally resources with 'auto_update' will be
  /// redeployed immediately if updates are found.
  /// With this enabled, convert this into an UpdateAvailable alert.
  #[serde(default)]
  pub skip_auto_update: bool,
  /// If check triggers auto deploy,
  /// whether this call should wait on the auto deploy,
  /// or run it in the background.
  #[serde(default)]
  pub wait_for_auto_update: bool,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CheckDeploymentForUpdateResponse {
  /// The deployment ID
  pub deployment: String,
  /// Whether update is available
  pub update_available: bool,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/BatchCheckDeploymentForUpdate",
  description = "Checks for newer image than what is deployed.",
  request_body(content = BatchCheckDeploymentForUpdate),
  responses(
    (status = 200, description = "Per deployment result", body = BatchCheckDeploymentForUpdateResponse),
  ),
)]
pub fn batch_check_deployment_for_update() {}

/// Checks for newer image than what is deployed. Response: [BatchCheckDeploymentForUpdateResponse]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoWriteRequest)]
#[response(BatchCheckDeploymentForUpdateResponse)]
#[error(mogh_error::Error)]
pub struct BatchCheckDeploymentForUpdate {
  /// Id or name or wildcard pattern or regex.
  /// Supports multiline and comma delineated combinations of the above.
  ///
  /// Example:
  /// ```text
  /// # match all foo-* deployments
  /// foo-*
  /// # add some more
  /// extra-deployment-1, extra-deployment-2
  /// ```
  pub pattern: String,
  /// Normally resources with 'auto_update' will be
  /// redeployed immediately if updates are found.
  /// With this enabled, convert this into an UpdateAvailable alert.
  #[serde(default)]
  pub skip_auto_update: bool,
  /// If check triggers auto deploy,
  /// whether this call should wait on the auto deploy,
  /// or run it in the background.
  #[serde(default)]
  pub wait_for_auto_update: bool,
}

#[typeshare]
pub type BatchCheckDeploymentForUpdateResponse =
  Vec<CheckDeploymentForUpdateResponse>;
