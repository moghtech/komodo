use std::time::Duration;

use anyhow::{Context, anyhow};
use komodo_client::entities::{SwarmOrServer, komodo_timestamp};
use periphery_client::api::docker::GetLatestImageCreated;
use serde::Deserialize;

use crate::helpers::swarm_or_server_request;

/// Checks the latest image age against `min_update_age_hours`,
/// preferring the time the image was pushed to the registry.
/// Fails open (`true`) if it's `0`, or the age can't be determined.
pub async fn image_meets_min_age(
  swarm_or_server: &SwarmOrServer,
  image: &str,
  account: Option<String>,
  token: Option<String>,
  min_update_age_hours: u32,
) -> anyhow::Result<bool> {
  if min_update_age_hours == 0 {
    return Ok(true);
  }

  let pushed = docker_hub_pushed(image).await.unwrap_or_else(|e| {
    warn!("Failed to get image push time for {image} | {e:#}");
    None
  });

  let published = match pushed {
    Some(pushed) => pushed,
    // No other registry exposes a push time. The fallback is the
    // build time, which whoever built the image can backdate.
    None => {
      let res = swarm_or_server_request(
        swarm_or_server,
        GetLatestImageCreated {
          name: image.to_string(),
          account,
          token,
        },
      )
      .await?;
      let Some(created) = res.created else {
        return Ok(true);
      };
      created
    }
  };

  let Ok(published) =
    chrono::DateTime::parse_from_rfc3339(&published)
  else {
    warn!("Failed to parse image time '{published}' for {image}");
    return Ok(true);
  };

  let age_ms = komodo_timestamp() - published.timestamp_millis();
  let min_age_ms = min_update_age_hours as i64 * 60 * 60 * 1_000;

  Ok(age_ms >= min_age_ms)
}

/// Returns when the image's tag was last pushed to Docker Hub,
/// or None if it isn't a public image hosted there.
async fn docker_hub_pushed(
  image: &str,
) -> anyhow::Result<Option<String>> {
  let Some((repository, tag)) = docker_hub_repo_tag(image) else {
    return Ok(None);
  };
  // Tags have no characters needing url encoding
  let res = reqwest::Client::new()
    .get(format!(
      "https://hub.docker.com/v2/repositories/{repository}/tags/{tag}"
    ))
    .timeout(Duration::from_secs(10))
    .send()
    .await
    .context("Failed to reach Docker Hub api")?;

  let status = res.status();
  // Private repositories are hidden rather than refused
  if status == 404 {
    return Ok(None);
  }
  if !status.is_success() {
    let text = res.text().await.unwrap_or_default();
    return Err(anyhow!(
      "Failed to get Docker Hub tag | {status} | {text}"
    ));
  }

  #[derive(Deserialize)]
  struct Tag {
    tag_last_pushed: Option<String>,
  }

  Ok(
    res
      .json::<Tag>()
      .await
      .context("Failed to parse Docker Hub tag response")?
      .tag_last_pushed,
  )
}

/// Splits an image into the `namespace/repository` and tag
/// Docker Hub's api takes, or None if it isn't a Docker Hub image.
fn docker_hub_repo_tag(image: &str) -> Option<(String, &str)> {
  // Images with a hardcoded digest have no tag to look up.
  if image.contains('@') {
    return None;
  }
  // Like `extract_registry_domain`, only a leading
  // segment which looks like a host is the registry.
  let image = match image.split_once('/') {
    Some((domain, rest))
      if domain.contains('.') || domain.contains(':') =>
    {
      matches!(domain, "docker.io" | "index.docker.io")
        .then_some(rest)?
    }
    _ => image,
  };
  let (repository, tag) = match image.rsplit_once(':') {
    Some((repository, tag)) => (repository, tag),
    None => (image, "latest"),
  };
  if repository.is_empty() {
    return None;
  }
  // Official images live under the `library` namespace
  let repository = if repository.contains('/') {
    repository.to_string()
  } else {
    format!("library/{repository}")
  };
  Some((repository, tag))
}

#[cfg(test)]
mod tests {
  use super::docker_hub_repo_tag;

  #[test]
  fn docker_hub_repo_tag_images() {
    assert_eq!(
      docker_hub_repo_tag("alpine"),
      Some((String::from("library/alpine"), "latest"))
    );
    assert_eq!(
      docker_hub_repo_tag("moghtech/komodo-core:2.3.3"),
      Some((String::from("moghtech/komodo-core"), "2.3.3"))
    );
    assert_eq!(
      docker_hub_repo_tag("docker.io/moghtech/komodo-core:latest"),
      Some((String::from("moghtech/komodo-core"), "latest"))
    );
    // Only Docker Hub has a push time api
    assert_eq!(
      docker_hub_repo_tag("ghcr.io/moghtech/komodo-core:latest"),
      None
    );
    assert_eq!(
      docker_hub_repo_tag("localhost:5000/komodo-core"),
      None
    );
    assert_eq!(docker_hub_repo_tag("alpine@sha256:abc123"), None);
  }
}
