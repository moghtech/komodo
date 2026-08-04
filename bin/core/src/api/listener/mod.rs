use std::sync::Arc;

use axum::{Router, http::HeaderMap};
use komodo_client::entities::resource::Resource;
use mogh_cache::CloneCache;
use tokio::sync::Mutex;

use crate::resource::KomodoResource;

mod integrations;
mod resources;
mod router;

use integrations::*;

pub fn router() -> Router {
  Router::new()
    .nest("/github", router::router::<github::Github>())
    .nest("/gitlab", router::router::<gitlab::Gitlab>())
}

type ListenerLockCache = CloneCache<String, Arc<Mutex<()>>>;

/// Implemented for all resources which can recieve webhook.
trait CustomSecret: KomodoResource {
  fn custom_secret(
    resource: &Resource<Self::Config, Self::Info>,
  ) -> &str;
}

/// Implemented on the integration struct, eg [integrations::github::Github]
trait VerifySecret {
  fn verify_secret(
    headers: &HeaderMap,
    body: &str,
    custom_secret: &str,
  ) -> anyhow::Result<()>;
}

/// Implemented on the integration struct, eg [integrations::github::Github]
trait ExtractBranch {
  fn extract_branch(body: &str) -> anyhow::Result<String>;
  /// Whether the webhook body's branch matches `expected`.
  /// A mismatch is routine and only logged at debug; errors
  /// only when the branch cannot be extracted from the body.
  fn branch_matches(
    body: &str,
    expected: &str,
  ) -> anyhow::Result<bool> {
    let branch = Self::extract_branch(body)?;
    if branch == expected {
      Ok(true)
    } else {
      debug!(
        "Ignoring webhook | push to branch '{branch}' does not match expected branch '{expected}'"
      );
      Ok(false)
    }
  }
}

/// For Procedures and Actions, incoming webhook
/// can be triggered by any branch by using `__ANY__`
/// as the branch in the webhook URL.
const ANY_BRANCH: &str = "__ANY__";
