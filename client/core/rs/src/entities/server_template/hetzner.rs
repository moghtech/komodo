use std::collections::HashMap;

use derive_builder::Builder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use typeshare::typeshare;

use crate::{
  deserializers::{
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::I64,
};

#[typeshare(serialized_as = "Partial<HetznerServerTemplateConfig>")]
pub type _PartialHetznerServerTemplateConfig =
  PartialHetznerServerTemplateConfig;

/// Hetzner server config.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct HetznerServerTemplateConfig {
  /// ID or name of the Image the Server is created from
  #[serde(default = "default_image")]
  #[builder(default = "default_image()")]
  #[partial_default(default_image())]
  pub image: String,
  /// ID or name of Datacenter to create Server in
  #[serde(default)]
  #[builder(default)]
  pub datacenter: HetznerDatacenter,
  /// ID of the Placement Group the server should be in,
  /// Or 0 to not use placement group.
  #[serde(default)]
  #[builder(default)]
  pub placement_group: I64,
  /// ID or name of the Server type this Server should be created with
  #[serde(default)]
  #[builder(default)]
  pub server_type: HetznerServerType,
  /// SSH key IDs ( integer ) or names ( string ) which should be injected into the Server at creation time
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub ssh_keys: Vec<String>,
  /// Network IDs which should be attached to the Server private network interface at the creation time
  #[serde(default)]
  #[builder(default)]
  pub private_network_ids: Vec<I64>,
  /// Attach an IPv4 on the public NIC. If false, no IPv4 address will be attached.
  #[serde(default)]
  #[builder(default)]
  pub enable_public_ipv4: bool,
  /// Attach an IPv6 on the public NIC. If false, no IPv6 address will be attached.
  #[serde(default)]
  #[builder(default)]
  pub enable_public_ipv6: bool,
  /// Connect to the instance using it's public ip.
  #[serde(default)]
  #[builder(default)]
  pub use_public_ip: bool,
  /// The port periphery will be running on in AMI.
  /// Default: `8120`
  #[serde(default = "default_port")]
  #[builder(default = "default_port()")]
  #[partial_default(default_port())]
  pub port: i32,
  /// Whether Periphery will be running on https
  #[serde(default = "default_use_https")]
  #[builder(default = "default_use_https()")]
  #[partial_default(default_use_https())]
  pub use_https: bool,
  /// The firewalls to attach to the instance
  #[serde(default)]
  #[builder(default)]
  pub firewall_ids: Vec<I64>,
  /// Labels for the server
  #[serde(default)]
  #[builder(default)]
  pub labels: HashMap<String, String>,
  /// Specs for volumes to attach
  #[serde(default)]
  #[builder(default)]
  pub volumes: Vec<HetznerVolumeSpecs>,
  /// Cloud-Init user data to use during Server creation. This field is limited to 32KiB.
  #[serde(default = "default_user_data")]
  #[builder(default = "default_user_data()")]
  #[partial_default(default_user_data())]
  pub user_data: String,
}

impl HetznerServerTemplateConfig {
  pub fn builder() -> HetznerServerTemplateConfigBuilder {
    HetznerServerTemplateConfigBuilder::default()
  }
}

fn default_image() -> String {
  String::from("ubuntu-24.04")
}

fn default_port() -> i32 {
  8120
}

fn default_use_https() -> bool {
  true
}

fn default_user_data() -> String {
  String::from("#cloud-config
runcmd:
  - apt update
  - apt upgrade -y
  - curl -fsSL https://get.docker.com | sh
  - systemctl enable docker.service
  - systemctl enable containerd.service
  - curl -sSL 'https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py' | HOME=/root python3
  - systemctl enable periphery.service")
}

impl Default for HetznerServerTemplateConfig {
  fn default() -> Self {
    Self {
      port: default_port(),
      use_https: default_use_https(),
      image: default_image(),
      datacenter: Default::default(),
      private_network_ids: Default::default(),
      placement_group: Default::default(),
      enable_public_ipv4: Default::default(),
      enable_public_ipv6: Default::default(),
      firewall_ids: Default::default(),
      server_type: Default::default(),
      ssh_keys: Default::default(),
      user_data: default_user_data(),
      use_public_ip: Default::default(),
      labels: Default::default(),
      volumes: Default::default(),
    }
  }
}

#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Builder,
)]
pub struct HetznerVolumeSpecs {
  /// A name for the volume
  pub name: String,
  /// Size of the volume in GB
  pub size_gb: I64,
  /// The format for the volume
  #[serde(default)]
  #[builder(default)]
  pub format: HetznerVolumeFormat,
  /// Labels for the volume
  #[serde(default)]
  #[builder(default)]
  pub labels: HashMap<String, String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum HetznerVolumeFormat {
  #[default]
  Xfs,
  Ext4,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Serialize,
  Deserialize,
  AsRefStr,
)]
#[allow(clippy::enum_variant_names)]
pub enum HetznerServerType {
  // Shared
  #[default]
  /// CPX11 - AMD 2 Cores, 2 Gb Ram, 40 Gb disk
  SharedAmd2Core2Ram40Disk,
  /// CAX11 - Arm 2 Cores, 4 Gb Ram, 40 Gb disk
  SharedArm2Core4Ram40Disk,
  /// CX22 - Intel 2 Cores, 4 Gb Ram, 40 Gb disk
  SharedIntel2Core4Ram40Disk,
  /// CPX21 - AMD 3 Cores, 4 Gb Ram, 80 Gb disk
  SharedAmd3Core4Ram80Disk,
  /// CAX21 - Arm 4 Cores, 8 Gb Ram, 80 Gb disk
  SharedArm4Core8Ram80Disk,
  /// CX32 - Intel 4 Cores, 8 Gb Ram, 80 Gb disk
  SharedIntel4Core8Ram80Disk,
  /// CPX31 - AMD 4 Cores, 8 Gb Ram, 160 Gb disk
  SharedAmd4Core8Ram160Disk,
  /// CAX31 - Arm 8 Cores, 16 Gb Ram, 160 Gb disk
  SharedArm8Core16Ram160Disk,
  /// CX42 - Intel 8 Cores, 16 Gb Ram, 160 Gb disk
  SharedIntel8Core16Ram160Disk,
  /// CPX41 - AMD 8 Cores, 16 Gb Ram, 240 Gb disk
  SharedAmd8Core16Ram240Disk,
  /// CAX41 - Arm 16 Cores, 32 Gb Ram, 320 Gb disk
  SharedArm16Core32Ram320Disk,
  /// CX52 - Intel 16 Cores, 32 Gb Ram, 320 Gb disk
  SharedIntel16Core32Ram320Disk,
  /// CPX51 - AMD 16 Cores, 32 Gb Ram, 360 Gb disk
  SharedAmd16Core32Ram360Disk,

  // Dedicated
  /// CCX13 - AMD 2 Cores, 8 Gb Ram, 80 Gb disk
  DedicatedAmd2Core8Ram80Disk,
  /// CCX23 - AMD 4 Cores, 16 Gb Ram, 160 Gb disk
  DedicatedAmd4Core16Ram160Disk,
  /// CCX33 - AMD 8 Cores, 32 Gb Ram, 240 Gb disk
  DedicatedAmd8Core32Ram240Disk,
  /// CCX43 - AMD 16 Cores, 64 Gb Ram, 360 Gb disk
  DedicatedAmd16Core64Ram360Disk,
  /// CCX53 - AMD 32 Cores, 128 Gb Ram, 600 Gb disk
  DedicatedAmd32Core128Ram600Disk,
  /// CCX63 - AMD 48 Cores, 192 Gb Ram, 960 Gb disk
  DedicatedAmd48Core192Ram960Disk,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Serialize,
  Deserialize,
  AsRefStr,
)]
pub enum HetznerDatacenter {
  #[default]
  Nuremberg1Dc3,
  Helsinki1Dc2,
  Falkenstein1Dc14,
  AshburnDc1,
  HillsboroDc1,
  SingaporeDc1,
}
