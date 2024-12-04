use command::run_komodo_command;
use komodo_client::entities::{docker::volume::Volume, update::Log};
use periphery_client::api::volume::*;
use resolver_api::Resolve;

use crate::docker::docker_client;

//

impl Resolve<super::Args> for InspectVolume {
  #[instrument(name = "InspectVolume", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Volume> {
    Ok(docker_client().inspect_volume(&self.name).await?)
  }
}

//

impl Resolve<super::Args> for DeleteVolume {
  #[instrument(name = "DeleteVolume")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = format!("docker volume rm {}", self.name);
    Ok(
      run_komodo_command("delete volume", None, command, false).await,
    )
  }
}

//

impl Resolve<super::Args> for PruneVolumes {
  #[instrument(name = "PruneVolumes")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker volume prune -a -f");
    Ok(
      run_komodo_command("prune volumes", None, command, false).await,
    )
  }
}
