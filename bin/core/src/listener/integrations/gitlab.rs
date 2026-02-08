use anyhow::{Context, anyhow};
use serde::Deserialize;

use crate::{
  config::core_config,
  listener::{ExtractBranch, VerifySecret},
};

/// Listener implementation for Gitlab type API
pub struct Gitlab;

impl VerifySecret for Gitlab {
  #[instrument("VerifyGitlabSecret", skip_all)]
  fn verify_secret(
    headers: axum::http::HeaderMap,
    _body: &str,
    custom_secret: &str,
  ) -> anyhow::Result<()> {
    let token = headers
      .get("x-gitlab-token")
      .context("No gitlab token in headers")?;
    let token =
      token.to_str().context("Failed to get token as string")?;
    let secret = if custom_secret.is_empty() {
      core_config().webhook_secret.as_str()
    } else {
      custom_secret
    };
    if token == secret {
      Ok(())
    } else {
      Err(anyhow!("Webhook secret does not match expected."))
    }
  }
}

#[derive(Deserialize)]
struct GitlabWebhookBody {
  #[serde(rename = "ref")]
  git_ref: Option<String>,
  /// Present in Release Hook payloads
  tag: Option<String>,
}

impl ExtractBranch for Gitlab {
  fn extract_branch(body: &str) -> anyhow::Result<String> {
    let payload = serde_json::from_str::<GitlabWebhookBody>(body)
      .context("Failed to parse gitlab request body")?;

    // Release Hook: use the `tag` field directly
    if let Some(tag) = payload.tag {
      return Ok(tag);
    }

    // Push Hook / Tag Push Hook: strip refs/heads/ or refs/tags/ prefix
    let git_ref = payload
      .git_ref
      .context("Gitlab webhook body has no 'ref' or 'tag' field")?;
    let branch = git_ref
      .strip_prefix("refs/heads/")
      .or_else(|| git_ref.strip_prefix("refs/tags/"))
      .unwrap_or(&git_ref);
    Ok(branch.to_string())
  }
}
