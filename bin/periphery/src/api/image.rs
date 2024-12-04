use std::sync::OnceLock;

use cache::TimeoutCache;
use command::run_komodo_command;
use komodo_client::entities::{
  deployment::extract_registry_domain,
  docker::image::{Image, ImageHistoryResponseItem},
  komodo_timestamp,
  update::Log,
};
use periphery_client::api::image::*;
use resolver_api::Resolve;

use crate::docker::{docker_client, docker_login};

//

impl Resolve<super::Args> for InspectImage {
  #[instrument(name = "InspectImage", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Image> {
    Ok(docker_client().inspect_image(&self.name).await?)
  }
}

//

impl Resolve<super::Args> for ImageHistory {
  #[instrument(name = "ImageHistory", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<ImageHistoryResponseItem>> {
    Ok(docker_client().image_history(&self.name).await?)
  }
}

//

/// Wait this long after a pull to allow another pull through
const PULL_TIMEOUT: i64 = 5_000;

fn pull_cache() -> &'static TimeoutCache<String, Log> {
  static PULL_CACHE: OnceLock<TimeoutCache<String, Log>> =
    OnceLock::new();
  PULL_CACHE.get_or_init(Default::default)
}

impl Resolve<super::Args> for PullImage {
  #[instrument(name = "PullImage", skip_all, fields(name = &self.name))]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let PullImage {
      name,
      account,
      token,
    } = self;
    // Acquire the image lock
    let lock = pull_cache().get_lock(name.clone()).await;

    // Lock the image lock, prevents simultaneous pulls by
    // ensuring simultaneous pulls will wait for first to finish
    // and checking cached results.
    let mut locked = lock.lock().await;

    // Early return from cache if lasted pulled with PULL_TIMEOUT
    if locked.last_ts + PULL_TIMEOUT > komodo_timestamp() {
      return locked.clone_res().map_err(Into::into);
    }

    let res = async {
      docker_login(
        &extract_registry_domain(&name)?,
        account.as_deref().unwrap_or_default(),
        token.as_deref(),
      )
      .await?;
      anyhow::Ok(
        run_komodo_command(
          "docker pull",
          None,
          format!("docker pull {name}"),
          false,
        )
        .await,
      )
    }
    .await;

    // Set the cache with results. Any other calls waiting on the lock will
    // then immediately also use this same result.
    locked.set(&res, komodo_timestamp());

    res.map_err(Into::into)
  }
}

//

impl Resolve<super::Args> for DeleteImage {
  #[instrument(name = "DeleteImage")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = format!("docker image rm {}", self.name);
    Ok(run_komodo_command("delete image", None, command, false).await)
  }
}

//

impl Resolve<super::Args> for PruneImages {
  #[instrument(name = "PruneImages")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker image prune -a -f");
    Ok(run_komodo_command("prune images", None, command, false).await)
  }
}
