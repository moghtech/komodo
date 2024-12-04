use command::run_komodo_command;
use komodo_client::entities::{
  docker::network::Network, update::Log,
};
use periphery_client::api::network::*;
use resolver_api::Resolve;

use crate::docker::docker_client;

//

impl Resolve<super::Args> for InspectNetwork {
  #[instrument(name = "InspectNetwork", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Network> {
    Ok(docker_client().inspect_network(&self.name).await?)
  }
}

//

impl Resolve<super::Args> for CreateNetwork {
  #[instrument(name = "CreateNetwork", skip(self))]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let CreateNetwork { name, driver } = self;
    let driver = match driver {
      Some(driver) => format!(" -d {driver}"),
      None => String::new(),
    };
    let command = format!("docker network create{driver} {name}");
    Ok(
      run_komodo_command("create network", None, command, false)
        .await,
    )
  }
}

//

impl Resolve<super::Args> for DeleteNetwork {
  #[instrument(name = "DeleteNetwork", skip(self))]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = format!("docker network rm {}", self.name);
    Ok(
      run_komodo_command("delete network", None, command, false)
        .await,
    )
  }
}

//

impl Resolve<super::Args> for PruneNetworks {
  #[instrument(name = "PruneNetworks", skip(self))]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker network prune -f");
    Ok(
      run_komodo_command("prune networks", None, command, false)
        .await,
    )
  }
}
