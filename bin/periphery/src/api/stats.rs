use komodo_client::entities::stats::{
  SystemInformation, SystemProcess, SystemStats,
};
use periphery_client::api::stats::{
  GetSystemInformation, GetSystemProcesses, GetSystemStats,
};
use resolver_api::Resolve;

use crate::stats::stats_client;

impl Resolve<super::Args> for GetSystemInformation {
  #[instrument(
    name = "GetSystemInformation",
    level = "debug",
    skip_all
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<SystemInformation> {
    Ok(stats_client().read().await.info.clone())
  }
}

//

impl Resolve<super::Args> for GetSystemStats {
  #[instrument(name = "GetSystemStats", level = "debug", skip_all)]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<SystemStats> {
    Ok(stats_client().read().await.stats.clone())
  }
}

//

impl Resolve<super::Args> for GetSystemProcesses {
  #[instrument(name = "GetSystemProcesses", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<SystemProcess>> {
    Ok(stats_client().read().await.get_processes())
  }
}
