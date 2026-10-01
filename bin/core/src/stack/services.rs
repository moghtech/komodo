use std::collections::HashMap;

use anyhow::Context;
use komodo_client::entities::{
  ImageDigest,
  stack::{ComposeFile, ComposeService, Stack, StackServiceNames},
};

pub fn extract_services_from_stack(
  stack: &Stack,
) -> Vec<StackServiceNames> {
  if let Some(mut services) = stack.info.deployed_services.clone() {
    for service in services.iter_mut().filter(|s| s.image.is_empty())
    {
      service.image = stack
        .info
        .latest_services
        .iter()
        .find(|s| s.service_name == service.service_name)
        .map(|s| s.image.clone())
        .unwrap_or_default();
    }
    services
  } else {
    stack.info.latest_services.clone()
  }
}

/// If `services` is a single entry Komodo lists for one replica
/// of a compose service with `deploy.replicas > 1` (eg `web-2`),
/// returns the exact container name of that replica.
pub fn single_replica_container(
  stack: &Stack,
  services: &[String],
) -> Option<String> {
  let [service] = services else {
    return None;
  };
  extract_services_from_stack(stack)
    .into_iter()
    .find(|s| {
      &s.service_name == service && s.compose_service.is_some()
    })
    .map(|s| s.container_name)
}

/// Maps the names Komodo lists for replicas of a compose service
/// with `deploy.replicas > 1` (eg `web-2`) back to the actual
/// compose service name (`web`), for use with `docker compose`.
pub fn to_compose_services(
  stack: &Stack,
  services: Vec<String>,
) -> Vec<String> {
  let known = extract_services_from_stack(stack);
  let mut res = Vec::with_capacity(services.len());
  for service in services {
    let service = known
      .iter()
      .find(|s| s.service_name == service)
      .and_then(|s| s.compose_service.clone())
      .unwrap_or(service);
    if !res.contains(&service) {
      res.push(service);
    }
  }
  res
}

pub fn extract_services_into_res(
  project_name: &str,
  compose_contents: &str,
  service_image_digests: &HashMap<String, ImageDigest>,
  res: &mut Vec<StackServiceNames>,
) -> anyhow::Result<()> {
  let compose =
    serde_yaml_ng::from_str::<ComposeFile>(compose_contents)
      .context(
        "failed to parse service names from compose contents",
      )?;

  for (
    service_name,
    ComposeService {
      container_name,
      image,
      ..
    },
  ) in compose.services
  {
    if let Some(existing) =
      res.iter_mut().find(|s| s.service_name == service_name)
    {
      // Override any defined fields
      if let Some(container_name) = container_name {
        existing.container_name = container_name;
      }
      if let Some(image) = image {
        existing.image = image;
      }
    } else {
      res.push(StackServiceNames {
        container_name: container_name.unwrap_or_else(|| {
          format!("{project_name}-{service_name}")
        }),
        image_digest: service_image_digests
          .get(&service_name)
          .cloned(),
        image: image.unwrap_or_default(),
        service_name,
        compose_service: None,
      });
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn replica_stack() -> Stack {
    let mut stack = Stack::default();
    let service =
      |name: &str, compose_service: Option<&str>| StackServiceNames {
        service_name: name.to_string(),
        container_name: format!("proj-{name}"),
        compose_service: compose_service.map(str::to_string),
        ..Default::default()
      };
    stack.info.deployed_services = Some(vec![
      service("web-1", Some("web")),
      service("web-2", Some("web")),
      service("db", None),
    ]);
    stack
  }

  #[test]
  fn single_replica_resolves_container() {
    let stack = replica_stack();
    assert_eq!(
      single_replica_container(&stack, &["web-2".to_string()])
        .as_deref(),
      Some("proj-web-2")
    );
    assert_eq!(
      single_replica_container(&stack, &["db".to_string()]),
      None
    );
    assert_eq!(
      single_replica_container(
        &stack,
        &["web-1".to_string(), "web-2".to_string()]
      ),
      None
    );
  }

  #[test]
  fn replicas_map_to_compose_service() {
    let stack = replica_stack();
    let services = ["web-1", "web-2", "db", "unknown"]
      .map(str::to_string)
      .to_vec();
    assert_eq!(
      to_compose_services(&stack, services),
      ["web", "db", "unknown"]
    );
  }
}
