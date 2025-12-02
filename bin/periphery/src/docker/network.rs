use bollard::query_parameters::{
  InspectNetworkOptions, ListNetworksOptions,
};
use komodo_client::entities::docker::{
  container::ContainerListItem, network::*,
};

use super::DockerClient;

impl DockerClient {
  pub async fn list_networks(
    &self,
    containers: &[ContainerListItem],
  ) -> anyhow::Result<Vec<NetworkListItem>> {
    let mut networks = self
      .docker
      .list_networks(Option::<ListNetworksOptions>::None)
      .await?
      .into_iter()
      .map(|network| {
        let (ipam_driver, ipam_subnet, ipam_gateway) =
          if let Some(ipam) = network.ipam {
            let (subnet, gateway) = if let Some(config) = ipam
              .config
              .and_then(|configs| configs.into_iter().next())
            {
              (config.subnet, config.gateway)
            } else {
              (None, None)
            };
            (ipam.driver, subnet, gateway)
          } else {
            (None, None, None)
          };
        let in_use = match &network.name {
          Some(name) => containers.iter().any(|container| {
            container.networks.iter().any(|_name| name == _name)
          }),
          None => false,
        };
        NetworkListItem {
          name: network.name,
          id: network.id,
          created: network.created,
          scope: network.scope,
          driver: network.driver,
          enable_ipv6: network.enable_ipv6,
          ipam_driver,
          ipam_subnet,
          ipam_gateway,
          internal: network.internal,
          attachable: network.attachable,
          ingress: network.ingress,
          in_use,
        }
      })
      .collect::<Vec<_>>();

    networks.sort_by(|a, b| {
      a.in_use.cmp(&b.in_use).then_with(|| a.name.cmp(&b.name))
    });

    Ok(networks)
  }

  pub async fn inspect_network(
    &self,
    network_name: &str,
  ) -> anyhow::Result<Network> {
    let network = self
      .docker
      .inspect_network(
        network_name,
        InspectNetworkOptions {
          verbose: true,
          ..Default::default()
        }
        .into(),
      )
      .await?;
    Ok(Network {
      name: network.name,
      id: network.id,
      created: network.created,
      scope: network.scope,
      driver: network.driver,
      enable_ipv6: network.enable_ipv6,
      ipam: network.ipam.map(|ipam| Ipam {
        driver: ipam.driver,
        config: ipam
          .config
          .unwrap_or_default()
          .into_iter()
          .map(|config| IpamConfig {
            subnet: config.subnet,
            ip_range: config.ip_range,
            gateway: config.gateway,
            auxiliary_addresses: config
              .auxiliary_addresses
              .unwrap_or_default(),
          })
          .collect(),
        options: ipam.options.unwrap_or_default(),
      }),
      internal: network.internal,
      attachable: network.attachable,
      ingress: network.ingress,
      containers: network
        .containers
        .unwrap_or_default()
        .into_iter()
        .map(|(container_id, container)| NetworkContainer {
          container_id,
          name: container.name,
          endpoint_id: container.endpoint_id,
          mac_address: container.mac_address,
          ipv4_address: container.ipv4_address,
          ipv6_address: container.ipv6_address,
        })
        .collect(),
      options: network.options.unwrap_or_default(),
      labels: network.labels.unwrap_or_default(),
    })
  }
}
