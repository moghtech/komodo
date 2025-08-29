use std::{
  collections::HashSet,
  sync::{Mutex, OnceLock},
};

use anyhow::Context;
use komodo_client::{
  api::execute::{Deploy, DeployStack},
  entities::{
    ResourceTarget,
    alert::{Alert, AlertData, SeverityLevel},
    build::Build,
    deployment::{Deployment, DeploymentImage, DeploymentState},
    docker::{
      container::{ContainerListItem, ContainerStateStatusEnum},
      image::ImageListItem,
    },
    komodo_timestamp,
    stack::{Stack, StackService, StackServiceNames, StackState},
    user::auto_redeploy_user,
  },
};

use crate::{
  alert::send_alerts,
  api::execute::{self, ExecuteRequest},
  helpers::query::get_stack_state_from_containers,
  stack::{
    compose_container_match_regex,
    services::extract_services_from_stack,
  },
  state::{
    action_states, db_client, deployment_status_cache,
    stack_status_cache,
  },
};

use super::{CachedDeploymentStatus, CachedStackStatus, History};

fn deployment_alert_sent_cache() -> &'static Mutex<HashSet<String>> {
  static CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
  CACHE.get_or_init(Default::default)
}

pub async fn update_deployment_cache(
  server_name: String,
  deployments: Vec<Deployment>,
  containers: &[ContainerListItem],
  images: &[ImageListItem],
  builds: &[Build],
) {
  let deployment_status_cache = deployment_status_cache();
  for deployment in deployments {
    let container = containers
      .iter()
      .find(|container| container.name == deployment.name)
      .cloned();
    let prev = deployment_status_cache
      .get(&deployment.id)
      .await
      .map(|s| s.curr.state);
    let state = container
      .as_ref()
      .map(|c| c.state.into())
      .unwrap_or(DeploymentState::NotDeployed);
    let image = match deployment.config.image {
      DeploymentImage::Build { build_id, version } => {
        let (build_name, build_version) = builds
          .iter()
          .find(|build| build.id == build_id)
          .map(|b| (b.name.as_ref(), b.config.version))
          .unwrap_or(("Unknown", Default::default()));
        let version = if version.is_none() {
          build_version.to_string()
        } else {
          version.to_string()
        };
        format!("{build_name}:{version}")
      }
      DeploymentImage::Image { image } => {
        // If image already has tag, leave it,
        // otherwise default the tag to latest
        if image.contains(':') {
          image.to_string()
        } else {
          format!("{image}:latest")
        }
      }
    };
    let update_available = if let Some(ContainerListItem {
      image_id: Some(curr_image_id),
      ..
    }) = &container
    {
      // Docker will automatically strip `docker.io` from incoming image names re #468.
      // Need to strip it in order to match by image name and find available updates.
      let image = image.strip_prefix("docker.io/").unwrap_or(&image);
      images
        .iter()
        .find(|i| i.name == image)
        .map(|i| &i.id != curr_image_id)
        .unwrap_or_default()
    } else {
      false
    };

    if update_available {
      if deployment.config.auto_update {
        if state == DeploymentState::Running
          && !action_states()
            .deployment
            .get_or_insert_default(&deployment.id)
            .await
            .busy()
            .unwrap_or(true)
        {
          let id = deployment.id.clone();
          let server_name = server_name.clone();
          tokio::spawn(async move {
            match execute::inner_handler(
              ExecuteRequest::Deploy(Deploy {
                deployment: deployment.name.clone(),
                stop_time: None,
                stop_signal: None,
              }),
              auto_redeploy_user().to_owned(),
            )
            .await
            {
              Ok(_) => {
                let ts = komodo_timestamp();
                let alert = Alert {
                  id: Default::default(),
                  ts,
                  resolved: true,
                  resolved_ts: ts.into(),
                  level: SeverityLevel::Ok,
                  target: ResourceTarget::Deployment(id.clone()),
                  data: AlertData::DeploymentAutoUpdated {
                    id,
                    name: deployment.name,
                    server_name,
                    server_id: deployment.config.server_id,
                    image,
                  },
                };
                let res = db_client().alerts.insert_one(&alert).await;
                if let Err(e) = res {
                  error!(
                    "Failed to record DeploymentAutoUpdated to db | {e:#}"
                  );
                }
                send_alerts(&[alert]).await;
              }
              Err(e) => {
                warn!(
                  "Failed to auto update Deployment {} | {e:#}",
                  deployment.name
                )
              }
            }
          });
        }
      } else if state == DeploymentState::Running
        && deployment.config.send_alerts
        && !deployment_alert_sent_cache()
          .lock()
          .unwrap()
          .contains(&deployment.id)
      {
        // Add that it is already sent to the cache, so another alert won't be sent.
        deployment_alert_sent_cache()
          .lock()
          .unwrap()
          .insert(deployment.id.clone());
        let ts = komodo_timestamp();
        let alert = Alert {
          id: Default::default(),
          ts,
          resolved: true,
          resolved_ts: ts.into(),
          level: SeverityLevel::Ok,
          target: ResourceTarget::Deployment(deployment.id.clone()),
          data: AlertData::DeploymentImageUpdateAvailable {
            id: deployment.id.clone(),
            name: deployment.name,
            server_name: server_name.clone(),
            server_id: deployment.config.server_id,
            image,
          },
        };
        let res = db_client().alerts.insert_one(&alert).await;
        if let Err(e) = res {
          error!(
            "Failed to record DeploymentImageUpdateAvailable to db | {e:#}"
          );
        }
        send_alerts(&[alert]).await;
      }
    } else {
      // If it sees there is no longer update available, remove
      // from the sent cache, so on next `update_available = true`
      // the cache is empty and a fresh alert will be sent.
      deployment_alert_sent_cache()
        .lock()
        .unwrap()
        .remove(&deployment.id);
    }
    deployment_status_cache
      .insert(
        deployment.id.clone(),
        History {
          curr: CachedDeploymentStatus {
            id: deployment.id,
            state,
            container,
            update_available,
          },
          prev,
        }
        .into(),
      )
      .await;
  }
}

/// (StackId, Service)
fn stack_alert_sent_cache()
-> &'static Mutex<HashSet<(String, String)>> {
  static CACHE: OnceLock<Mutex<HashSet<(String, String)>>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

pub async fn update_stack_cache(
  server_name: String,
  stacks: Vec<Stack>,
  containers: &[ContainerListItem],
  images: &[ImageListItem],
) {
  let stack_status_cache = stack_status_cache();
  for stack in stacks {
    let services = extract_services_from_stack(&stack);
    let mut services_with_containers = services.iter().map(|StackServiceNames { service_name, container_name, image }| {
      let container = containers.iter().find(|container| {
        match compose_container_match_regex(container_name)
          .with_context(|| format!("failed to construct container name matching regex for service {service_name}")) 
        {
          Ok(regex) => regex,
          Err(e) => {
            warn!("{e:#}");
            return false
          }
        }.is_match(&container.name)
      }).cloned();
      let image = if image.contains(':') {
        image.to_string()
      } else {
        format!("{image}:latest")
      };
      let update_available = if let Some(ContainerListItem { image_id: Some(curr_image_id), .. }) = &container {
        // Docker will automatically strip `docker.io` from incoming image names re #468.
        // Need to strip it in order to match by image tag and find available update.
        let image =
          image.strip_prefix("docker.io/").unwrap_or(&image);
        images
          .iter()
          .find(|i| i.name == image)
          .map(|i| &i.id != curr_image_id)
          .unwrap_or_default()
      } else {
        false
      };
      if update_available {
        if !stack.config.auto_update
          && stack.config.send_alerts
          && container.is_some()
          && container.as_ref().unwrap().state == ContainerStateStatusEnum::Running
          && !stack_alert_sent_cache()
            .lock()
            .unwrap()
            .contains(&(stack.id.clone(), service_name.clone()))
        {
          stack_alert_sent_cache()
            .lock()
            .unwrap()
            .insert((stack.id.clone(), service_name.clone()));
          let ts = komodo_timestamp();
          let alert = Alert {
            id: Default::default(),
            ts,
            resolved: true,
            resolved_ts: ts.into(),
            level: SeverityLevel::Ok,
            target: ResourceTarget::Stack(stack.id.clone()),
            data: AlertData::StackImageUpdateAvailable {
              id: stack.id.clone(),
              name: stack.name.clone(),
              server_name: server_name.clone(),
              server_id: stack.config.server_id.clone(),
              service: service_name.clone(),
              image: image.clone(),
            },
          };
          tokio::spawn(async move {
            let res = db_client().alerts.insert_one(&alert).await;
            if let Err(e) = res {
              error!(
                "Failed to record StackImageUpdateAvailable to db | {e:#}"
              );
            }
            send_alerts(&[alert]).await;
          });
        }
      } else {
        stack_alert_sent_cache()
          .lock()
          .unwrap()
          .remove(&(stack.id.clone(), service_name.clone()));
      }
      StackService {
        service: service_name.clone(),
        image: image.clone(),
        container,
        update_available,
      }
    }).collect::<Vec<_>>();

    let mut images_with_update = Vec::new();
    let mut services_to_update = Vec::new();

    for service in services_with_containers.iter() {
      if service.update_available {
        images_with_update.push(service.image.clone());
        // Only allow it to actually trigger an auto update deploy
        // if the service is running.
        if service
          .container
          .as_ref()
          .map(|c| c.state == ContainerStateStatusEnum::Running)
          .unwrap_or_default()
        {
          services_to_update.push(service.service.clone());
        }
      }
    }

    let state = get_stack_state_from_containers(
      &stack.config.ignore_services,
      &services,
      containers,
    );
    if !services_to_update.is_empty()
      && stack.config.auto_update
      && state == StackState::Running
      && !action_states()
        .stack
        .get_or_insert_default(&stack.id)
        .await
        .busy()
        .unwrap_or(true)
    {
      let id = stack.id.clone();
      let server_name = server_name.clone();
      let services = if stack.config.auto_update_all_services {
        Vec::new()
      } else {
        services_to_update
      };
      tokio::spawn(async move {
        match execute::inner_handler(
          ExecuteRequest::DeployStack(DeployStack {
            stack: stack.name.clone(),
            services,
            stop_time: None,
          }),
          auto_redeploy_user().to_owned(),
        )
        .await
        {
          Ok(_) => {
            let ts = komodo_timestamp();
            let alert = Alert {
              id: Default::default(),
              ts,
              resolved: true,
              resolved_ts: ts.into(),
              level: SeverityLevel::Ok,
              target: ResourceTarget::Stack(id.clone()),
              data: AlertData::StackAutoUpdated {
                id,
                name: stack.name.clone(),
                server_name,
                server_id: stack.config.server_id,
                images: images_with_update,
              },
            };
            let res = db_client().alerts.insert_one(&alert).await;
            if let Err(e) = res {
              error!(
                "Failed to record StackAutoUpdated to db | {e:#}"
              );
            }
            send_alerts(&[alert]).await;
          }
          Err(e) => {
            warn!("Failed auto update Stack {} | {e:#}", stack.name,)
          }
        }
      });
    }
    services_with_containers
      .sort_by(|a, b| a.service.cmp(&b.service));
    let prev = stack_status_cache
      .get(&stack.id)
      .await
      .map(|s| s.curr.state);
    let status = CachedStackStatus {
      id: stack.id.clone(),
      state,
      services: services_with_containers,
    };
    stack_status_cache
      .insert(stack.id, History { curr: status, prev }.into())
      .await;
  }
}
