use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct NetworkListItem {
  pub name: Option<String>,
  pub id: Option<String>,
  pub created: Option<String>,
  pub scope: Option<String>,
  pub driver: Option<String>,
  pub enable_ipv6: Option<bool>,
  pub ipam_driver: Option<String>,
  pub ipam_subnet: Option<String>,
  pub ipam_gateway: Option<String>,
  pub internal: Option<bool>,
  pub attachable: Option<bool>,
  pub ingress: Option<bool>,
  /// Whether the network is attached to one or more containers
  pub in_use: bool,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Network {
  #[serde(rename = "Name")]
  pub name: Option<String>,

  #[serde(rename = "Id")]
  pub id: Option<String>,

  #[serde(rename = "Created")]
  pub created: Option<String>,

  #[serde(rename = "Scope")]
  pub scope: Option<String>,

  #[serde(rename = "Driver")]
  pub driver: Option<String>,

  #[serde(rename = "EnableIPv6")]
  pub enable_ipv6: Option<bool>,

  #[serde(rename = "IPAM")]
  pub ipam: Option<Ipam>,

  #[serde(rename = "Internal")]
  pub internal: Option<bool>,

  #[serde(rename = "Attachable")]
  pub attachable: Option<bool>,

  #[serde(rename = "Ingress")]
  pub ingress: Option<bool>,

  /// This field is turned from map into array for easier usability.
  #[serde(rename = "Containers")]
  pub containers: Vec<NetworkContainer>,

  #[serde(default, rename = "Options")]
  pub options: HashMap<String, String>,

  #[serde(default, rename = "Labels")]
  pub labels: HashMap<String, String>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Ipam {
  /// Name of the IPAM driver to use.
  #[serde(rename = "Driver")]
  pub driver: Option<String>,
  /// List of IPAM configuration options, specified as a map:  ``` {\"Subnet\": <CIDR>, \"IPRange\": <CIDR>, \"Gateway\": <IP address>, \"AuxAddress\": <device_name:IP address>} ```
  #[serde(rename = "Config")]
  pub config: Vec<IpamConfig>,
  /// Driver-specific options, specified as a map.
  #[serde(rename = "Options")]
  pub options: HashMap<String, String>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct IpamConfig {
  #[serde(rename = "Subnet")]
  pub subnet: Option<String>,
  #[serde(rename = "IPRange")]
  pub ip_range: Option<String>,
  #[serde(rename = "Gateway")]
  pub gateway: Option<String>,
  #[serde(rename = "AuxiliaryAddresses")]
  pub auxiliary_addresses: HashMap<String, String>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct NetworkContainer {
  /// This is the key on the incoming map of NetworkContainer
  #[serde(default, rename = "ContainerID")]
  pub container_id: String,
  #[serde(rename = "Name")]
  pub name: Option<String>,
  #[serde(rename = "EndpointID")]
  pub endpoint_id: Option<String>,
  #[serde(rename = "MacAddress")]
  pub mac_address: Option<String>,
  #[serde(rename = "IPv4Address")]
  pub ipv4_address: Option<String>,
  #[serde(rename = "IPv6Address")]
  pub ipv6_address: Option<String>,
}
