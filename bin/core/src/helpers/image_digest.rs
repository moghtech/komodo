use std::{sync::Arc, time::Duration};

use async_timing_util::get_timelength_in_ms;
use komodo_client::entities::{
  ImageDigest, SwarmOrServer, komodo_timestamp,
};
use mogh_cache::CloneCache;
use periphery_client::api::docker::GetLatestImageDigest;

use crate::{config::core_config, helpers::swarm_or_server_request};

/// Maps images -> (digest, valid until milliseconds)
pub struct ImageDigestCache {
  cache: CloneCache<String, (ImageDigest, i64)>,
  ttl_ms: i64,
}

impl ImageDigestCache {
  /// Also spawns a task to periodically clean up expired image digests.
  pub fn new() -> Arc<ImageDigestCache> {
    let ttl = core_config()
      .image_digest_cache_ttl
      .try_into()
      .unwrap_or_else(|_| {
        error!(
          "Invalid 'image_digest_cache_ttl', using default 10-min"
        );
        async_timing_util::Timelength::TenMinutes
      });
    let cache = Arc::new(ImageDigestCache {
      cache: Default::default(),
      ttl_ms: get_timelength_in_ms(ttl) as i64,
    });
    let clone = cache.clone();
    tokio::spawn(async move {
      let mut interval =
        tokio::time::interval(Duration::from_secs(60 * 60));
      interval.tick().await;
      loop {
        interval.tick().await;
        let ts = komodo_timestamp();
        clone
          .cache
          .retain(|_, (_, valid_until)| *valid_until > ts)
          .await;
      }
    });
    cache
  }

  /// Pass `use_cache: false` to always query the registry,
  /// the fresh digest is still stored for subsequent cached reads.
  pub async fn get(
    &self,
    swarm_or_server: &SwarmOrServer,
    image: &String,
    account: Option<String>,
    token: Option<String>,
    use_cache: bool,
  ) -> anyhow::Result<ImageDigest> {
    if use_cache
      && let Some((digest, valid_until)) = self.cache.get(image).await
      // Ensure the query time was within the configured ttl to use cache.
      && valid_until > komodo_timestamp()
    {
      return Ok(digest);
    }

    let digest = swarm_or_server_request(
      swarm_or_server,
      GetLatestImageDigest {
        name: image.clone(),
        account,
        token,
      },
    )
    .await?
    .digest;

    let digest = ImageDigest::new(image, &digest);

    self
      .cache
      .insert(
        image,
        (digest.clone(), komodo_timestamp() + self.ttl_ms),
      )
      .await;

    Ok(digest)
  }
}
