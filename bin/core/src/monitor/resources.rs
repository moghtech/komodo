use std::{
  collections::HashSet,
  hash::Hash,
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
      service::SwarmServiceListItem,
      stack::SwarmStackListItem,
      task::SwarmTaskListItem,
    },
    komodo_timestamp,
    stack::{Stack, StackService, StackServiceNames, StackState},
    swarm::SwarmState,
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
    CachedDeploymentStatus, CachedStackStatus, History,
    action_states, db_client, deployment_status_cache,
    stack_status_cache,
  },
};

/// (StackId, Service)
fn stack_alert_sent_cache() -> &'static AlertCache<(String, String)> {
  static CACHE: OnceLock<AlertCache<(String, String)>> =
    OnceLock::new();
  CACHE.get_or_init(Default::default)
}

pub async fn update_swarm_stack_cache(
  stacks: Vec<Stack>,
  swarm_stacks: &[SwarmStackListItem],
  swarm_services: &[SwarmServiceListItem],
) {
  let stack_status_cache = stack_status_cache();
  for stack in stacks {
    let project_name = stack.project_name(false);
    let current_state = swarm_stacks
      .iter()
      .find(|stack| {
        stack
          .name
          .as_ref()
          .map(|name| name == &project_name)
          .unwrap_or_default()
      })
      .map(|stack| match stack.state {
        Some(SwarmState::Healthy) => StackState::Running,
        Some(SwarmState::Unhealthy) => StackState::Unhealthy,
        Some(SwarmState::Unknown) | None => StackState::Unknown,
      })
      .unwrap_or(StackState::Down);
    let services = extract_services_from_stack(&stack);
    let service_prefix = format!("{project_name}_");
    let mut services_with_swarm_services = services
      .iter()
      .map(
        |StackServiceNames {
           service_name,
           image,
           ..
         }| {
          let swarm_service = swarm_services
            .iter()
            .find(|service| {
              service
                .name
                .as_ref()
                .map(|name| {
                  // The services are named like {stackname_servicename}.
                  // If they don't match this pattern, they aren't part of stack.
                  let Some(name) = name.strip_prefix(&service_prefix)
                  else {
                    return false;
                  };
                  name == service_name
                })
                .unwrap_or_default()
            })
            .cloned();
          StackService {
            service: service_name.clone(),
            image: image.clone(),
            container: None,
            swarm_service,
            update_available: false,
          }
        },
      )
      .collect::<Vec<_>>();
    services_with_swarm_services
      .sort_by(|a, b| a.service.cmp(&b.service));
    let prev_state = stack_status_cache
      .get(&stack.id)
      .await
      .map(|s| s.curr.state);
    let status = CachedStackStatus {
      id: stack.id.clone(),
      state: current_state,
      services: services_with_swarm_services,
    };
    stack_status_cache
      .insert(
        stack.id,
        History {
          curr: status,
          prev: prev_state,
        }
        .into(),
      )
      .await;
  }
}

pub async fn update_server_stack_cache(
  server_name: String,
  stacks: Vec<Stack>,
  containers: &[ContainerListItem],
  images: &[ImageListItem],
) {
  let stack_status_cache = stack_status_cache();
  let stack_alert_sent_cache = stack_alert_sent_cache();
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
          && let Some(container) = &container
          && container.state == ContainerStateStatusEnum::Running
          && !stack_alert_sent_cache
            .contains(&(stack.id.clone(), service_name.clone()))
        {
          stack_alert_sent_cache
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
        // If it sees there is no longer update available, remove
        // from the sent cache, so on next `update_available = true`
        // the cache is empty and a fresh alert will be sent.
        stack_alert_sent_cache
          .remove(&(stack.id.clone(), service_name.clone()));
      }
      StackService {
        service: service_name.clone(),
        image: image.clone(),
        container,
        swarm_service: None,
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

    let current_state = get_stack_state_from_containers(
      &stack.config.ignore_services,
      &services,
      containers,
    );
    if !services_to_update.is_empty()
      && stack.config.auto_update
      && current_state == StackState::Running
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
    let prev_state = stack_status_cache
      .get(&stack.id)
      .await
      .map(|s| s.curr.state);
    let status = CachedStackStatus {
      id: stack.id.clone(),
      state: current_state,
      services: services_with_containers,
    };
    stack_status_cache
      .insert(
        stack.id,
        History {
          curr: status,
          prev: prev_state,
        }
        .into(),
      )
      .await;
  }
}

fn deployment_alert_sent_cache() -> &'static AlertCache<String> {
  static CACHE: OnceLock<AlertCache<String>> = OnceLock::new();
  CACHE.get_or_init(Default::default)
}

pub async fn update_swarm_deployment_cache(
  deployments: Vec<Deployment>,
  swarm_services: &[SwarmServiceListItem],
  swarm_tasks: &[SwarmTaskListItem],
) {
  let deployment_status_cache = deployment_status_cache();
  for deployment in deployments {
    let service = swarm_services
      .iter()
      .find(|service| {
        service
          .name
          .as_ref()
          .map(|name| name == &deployment.name)
          .unwrap_or_default()
      })
      .cloned();
    let prev_state = deployment_status_cache
      .get(&deployment.id)
      .await
      .map(|s| s.curr.state);
    let current_state = service
      .as_ref()
      .map(|service| {
        let Some(service_id) = &service.id else {
          return DeploymentState::Unknown;
        };
        let tasks = swarm_tasks
          .iter()
          .filter(|task| {
            task
              .service_id
              .as_ref()
              .map(|id| id == service_id)
              .unwrap_or_default()
          })
          .collect::<Vec<_>>();
        // If service exists but no tasks, it is unhealthy
        if tasks.is_empty() {
          return DeploymentState::Unhealthy;
        }
        for task in tasks {
          if task.desired_state != task.state {
            return DeploymentState::Unhealthy;
          }
        }
        DeploymentState::Running
      })
      .unwrap_or(DeploymentState::NotDeployed);
    deployment_status_cache
      .insert(
        deployment.id.clone(),
        History {
          curr: CachedDeploymentStatus {
            id: deployment.id,
            state: current_state,
            service,
            container: None,
            update_available: false,
          },
          prev: prev_state,
        }
        .into(),
      )
      .await;
  }
}

pub async fn update_server_deployment_cache(
  server_name: String,
  deployments: Vec<Deployment>,
  containers: &[ContainerListItem],
  images: &[ImageListItem],
  builds: &[Build],
) {
  let deployment_status_cache = deployment_status_cache();
  let deployment_alert_sent_cache = deployment_alert_sent_cache();

  for deployment in deployments {
    let container = containers
      .iter()
      .find(|container| container.name == deployment.name)
      .cloned();
    let prev_state = deployment_status_cache
      .get(&deployment.id)
      .await
      .map(|s| s.curr.state);
    let current_state = container
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
        if current_state == DeploymentState::Running
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
      } else if current_state == DeploymentState::Running
        && deployment.config.send_alerts
        && deployment_alert_sent_cache.contains(&deployment.id)
      {
        // Add that it is already sent to the cache, so another alert won't be sent.
        deployment_alert_sent_cache.insert(deployment.id.clone());
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
      deployment_alert_sent_cache.remove(&deployment.id);
    }
    deployment_status_cache
      .insert(
        deployment.id.clone(),
        History {
          curr: CachedDeploymentStatus {
            id: deployment.id,
            state: current_state,
            container,
            service: None,
            update_available,
          },
          prev: prev_state,
        }
        .into(),
      )
      .await;
  }
}

struct AlertCache<K>(Mutex<HashSet<K>>);

impl<K> Default for AlertCache<K> {
  fn default() -> Self {
    Self(Default::default())
  }
}

impl<K: Eq + Hash> AlertCache<K> {
  /// Checks the cache for an existing key entry.
  /// If the cache is poisoned, will always return 'true' and include error log.
  fn contains(&self, key: &K) -> bool {
    self
      .0
      .lock()
      .map(|cache| cache.contains(key))
      .map_err(|_| error!("Alert Cache poisoned, this blocks container state change alerts, please restart Komodo Core."))
      .unwrap_or(true)
  }

  fn insert(&self, key: K) {
    let _ = self
      .0
      .lock()
      .map(|mut cache| cache.insert(key))
      .map_err(|_| error!("Alert Cache poisoned, this blocks container state change alerts, please restart Komodo Core."));
  }

  fn remove(&self, key: &K) {
    let _ = self
      .0
      .lock()
      .map(|mut cache| cache.remove(key))
      .map_err(|_| error!("Alert Cache poisoned, this blocks container state change alerts, please restart Komodo Core."));
  }
}
