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

  let mut services = Vec::with_capacity(compose.services.capacity());

  for (
    service_name,
    ComposeService {
      container_name,
      image,
      ..
    },
  ) in compose.services
  {
    let image = image.unwrap_or_default();
    services.push(StackServiceNames {
      container_name: container_name
        .unwrap_or_else(|| format!("{project_name}-{service_name}")),
      image_digest: service_image_digests.get(&service_name).cloned(),
      service_name,
      image,
    });
  }

  res.extend(services);

  Ok(())
}
