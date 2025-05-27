use anyhow::Context;
use formatting::format_serror;
use indexmap::IndexSet;
use komodo_client::entities::{
  Operation, ResourceTarget, ResourceTargetVariant,
  build::Build,
  deployment::{
    Deployment, DeploymentConfig, DeploymentConfigDiff,
    DeploymentImage, DeploymentListItem, DeploymentListItemInfo,
    DeploymentQuerySpecifics, DeploymentState,
    PartialDeploymentConfig, conversions_from_str,
  },
  environment_vars_from_str,
  permission::{PermissionLevel, SpecificPermission},
  resource::Resource,
  server::Server,
  to_docker_compatible_name,
  update::Update,
  user::User,
};
use mungos::mongodb::Collection;
use periphery_client::api::container::RemoveContainer;

use crate::{
  config::core_config,
  helpers::{
    empty_or_only_spaces, periphery_client,
    query::get_deployment_state,
  },
  monitor::update_cache_for_server,
  state::{action_states, db_client, deployment_status_cache},
};

use super::get_check_permissions;

impl super::KomodoResource for Deployment {
  type Config = DeploymentConfig;
  type PartialConfig = PartialDeploymentConfig;
  type ConfigDiff = DeploymentConfigDiff;
  type Info = ();
  type ListItem = DeploymentListItem;
  type QuerySpecifics = DeploymentQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Deployment
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Deployment(id.into())
  }

  fn validated_name(name: &str) -> String {
    to_docker_compatible_name(name)
  }

  fn creator_specific_permissions() -> IndexSet<SpecificPermission> {
    [SpecificPermission::Terminal].into_iter().collect()
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().deployments
  }

  async fn to_list_item(
    deployment: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let status = deployment_status_cache().get(&deployment.id).await;
    let (build_image, build_id) = match deployment.config.image {
      DeploymentImage::Build { build_id, version } => {
        let (build_name, build_id, build_version) =
          super::get::<Build>(&build_id)
            .await
            .map(|b| (b.name, b.id, b.config.version))
            .unwrap_or((
              String::from("unknown"),
              String::new(),
              Default::default(),
            ));
        let version = if version.is_none() {
          build_version.to_string()
        } else {
          version.to_string()
        };
        (format!("{build_name}:{version}"), Some(build_id))
      }
      DeploymentImage::Image { image } => (image, None),
    };
    let (image, update_available) = status
      .as_ref()
      .and_then(|s| {
        s.curr.container.as_ref().map(|c| {
          (
            c.image
              .clone()
              .unwrap_or_else(|| String::from("Unknown")),
            s.curr.update_available,
          )
        })
      })
      .unwrap_or((build_image, false));
    DeploymentListItem {
      name: deployment.name,
      id: deployment.id,
      tags: deployment.tags,
      resource_type: ResourceTargetVariant::Deployment,
      info: DeploymentListItemInfo {
        state: status
          .as_ref()
          .map(|s| s.curr.state)
          .unwrap_or_default(),
        status: status.as_ref().and_then(|s| {
          s.curr.container.as_ref().and_then(|c| c.status.to_owned())
        }),
        image,
        update_available,
        server_id: deployment.config.server_id,
        build_id,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .deployment
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateDeployment
  }

  fn user_can_create(user: &User) -> bool {
    user.admin || !core_config().disable_non_admin_create
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_create(
    created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    if created.config.server_id.is_empty() {
      return Ok(());
    }
    let Ok(server) = super::get::<Server>(&created.config.server_id)
      .await
      .inspect_err(|e| {
        warn!(
          "Failed to get Server for Deployment {} | {e:#}",
          created.name
        )
      })
    else {
      return Ok(());
    };
    update_cache_for_server(&server).await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateDeployment
  }

  async fn validate_update_config(
    _id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_update(
    updated: &Self,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    Self::post_create(updated, update).await
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameDeployment
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteDeployment
  }

  async fn pre_delete(
    deployment: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    let state = get_deployment_state(deployment)
      .await
      .context("failed to get container state")?;
    if matches!(
      state,
      DeploymentState::NotDeployed | DeploymentState::Unknown
    ) {
      return Ok(());
    }
    // container needs to be destroyed
    let server = match super::get::<Server>(
      &deployment.config.server_id,
    )
    .await
    {
      Ok(server) => server,
      Err(e) => {
        update.push_error_log(
          "remove container",
          format_serror(
            &e.context(format!(
              "failed to retrieve server at {} from db.",
              deployment.config.server_id
            ))
            .into(),
          ),
        );
        return Ok(());
      }
    };
    if !server.config.enabled {
      // Don't need to
      update.push_simple_log(
        "remove container",
        "skipping container removal, server is disabled.",
      );
      return Ok(());
    }
    let periphery = match periphery_client(&server) {
      Ok(periphery) => periphery,
      Err(e) => {
        // This case won't ever happen, as periphery_client only fallible if the server is disabled.
        // Leaving it for completeness sake
        update.push_error_log(
          "remove container",
          format_serror(
            &e.context("failed to get periphery client").into(),
          ),
        );
        return Ok(());
      }
    };
    match periphery
      .request(RemoveContainer {
        name: deployment.name.clone(),
        signal: deployment.config.termination_signal.into(),
        time: deployment.config.termination_timeout.into(),
      })
      .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "remove container",
        format_serror(
          &e.context("failed to remove container").into(),
        ),
      ),
    };
    Ok(())
  }

  async fn post_delete(
    _resource: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    Ok(())
  }
}

#[instrument(skip(user))]
async fn validate_config(
  config: &mut PartialDeploymentConfig,
  user: &User,
) -> anyhow::Result<()> {
  if let Some(server_id) = &config.server_id {
    if !server_id.is_empty() {
      let server = get_check_permissions::<Server>(
        server_id,
        user,
        PermissionLevel::Read.attach(),
      )
      .await
      .context("Cannot attach Deployment to this Server")?;
      config.server_id = Some(server.id);
    }
  }
  if let Some(DeploymentImage::Build { build_id, version }) =
    &config.image
  {
    if !build_id.is_empty() {
      let build = get_check_permissions::<Build>(
        build_id,
        user,
        PermissionLevel::Read.attach(),
      )
      .await
      .context(
        "Cannot update deployment with this build attached.",
      )?;
      config.image = Some(DeploymentImage::Build {
        build_id: build.id,
        version: *version,
      });
    }
  }
  if let Some(volumes) = &config.volumes {
    conversions_from_str(volumes).context("Invalid volumes")?;
  }
  if let Some(ports) = &config.ports {
    conversions_from_str(ports).context("Invalid ports")?;
  }
  if let Some(environment) = &config.environment {
    environment_vars_from_str(environment)
      .context("Invalid environment")?;
  }
  if let Some(extra_args) = &mut config.extra_args {
    extra_args.retain(|v| !empty_or_only_spaces(v))
  }
  Ok(())
}
