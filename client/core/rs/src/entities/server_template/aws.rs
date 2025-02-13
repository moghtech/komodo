use derive_builder::Builder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use typeshare::typeshare;

use crate::{
  deserializers::{
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::builder::AwsBuilderConfig,
};

#[typeshare(serialized_as = "Partial<AwsServerTemplateConfig>")]
pub type _PartialAwsServerTemplateConfig =
  PartialAwsServerTemplateConfig;

/// Aws EC2 instance config.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct AwsServerTemplateConfig {
  /// The aws region to launch the server in, eg. us-east-1
  #[serde(default = "default_region")]
  #[builder(default = "default_region()")]
  #[partial_default(default_region())]
  pub region: String,
  /// The instance type to launch, eg. c5.2xlarge
  #[serde(default = "default_instance_type")]
  #[builder(default = "default_instance_type()")]
  #[partial_default(default_instance_type())]
  pub instance_type: String,
  /// Specify the ami id to use. Must be set up to start the periphery binary on startup.
  pub ami_id: String,
  /// The subnet to assign to the instance.
  pub subnet_id: String,
  /// The key pair name to give to the instance in case SSH access required.
  pub key_pair_name: String,
  /// Assign a public ip to the instance. Depending on how your network is
  /// setup, this may be required for the instance to reach the public internet.
  #[serde(default = "default_assign_public_ip")]
  #[builder(default = "default_assign_public_ip()")]
  #[partial_default(default_assign_public_ip())]
  pub assign_public_ip: bool,
  /// Use the instances public ip as the address for the server.
  /// Could be used when build instances are created in another non-interconnected network to the core api.
  #[serde(default = "default_use_public_ip")]
  #[builder(default = "default_use_public_ip()")]
  #[partial_default(default_use_public_ip())]
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
  /// The security groups to give to the instance.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub security_group_ids: Vec<String>,
  /// Specify the EBS volumes to attach.
  #[serde(default = "default_volumes")]
  #[builder(default = "default_volumes()")]
  #[partial_default(default_volumes())]
  pub volumes: Vec<AwsVolume>,
  /// The user data to deploy the instance with.
  #[serde(default = "default_user_data")]
  #[builder(default = "default_user_data()")]
  #[partial_default(default_user_data())]
  pub user_data: String,
}

impl AwsServerTemplateConfig {
  pub fn builder() -> AwsServerTemplateConfigBuilder {
    AwsServerTemplateConfigBuilder::default()
  }
}

fn default_region() -> String {
  String::from("us-east-1")
}

fn default_instance_type() -> String {
  String::from("t3.small")
}

fn default_assign_public_ip() -> bool {
  true
}

fn default_use_public_ip() -> bool {
  false
}

fn default_volumes() -> Vec<AwsVolume> {
  vec![AwsVolume {
    device_name: "/dev/sda1".to_string(),
    size_gb: 20,
    volume_type: AwsVolumeType::Gp2,
    iops: 0,
    throughput: 0,
  }]
}

fn default_port() -> i32 {
  8120
}

fn default_use_https() -> bool {
  true
}

fn default_user_data() -> String {
  String::from("#!/bin/bash
apt update
apt upgrade -y
curl -fsSL https://get.docker.com | sh
systemctl enable docker.service
systemctl enable containerd.service
curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | HOME=/root python3
systemctl enable periphery.service")
}

impl Default for AwsServerTemplateConfig {
  fn default() -> Self {
    Self {
      region: default_region(),
      instance_type: default_instance_type(),
      assign_public_ip: default_assign_public_ip(),
      use_public_ip: default_use_public_ip(),
      port: default_port(),
      use_https: default_use_https(),
      volumes: default_volumes(),
      ami_id: Default::default(),
      subnet_id: Default::default(),
      key_pair_name: Default::default(),
      user_data: default_user_data(),
      security_group_ids: Default::default(),
    }
  }
}

/// For information on AWS volumes, see
/// `<https://docs.aws.amazon.com/ebs/latest/userguide/ebs-volume-types.html>`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsVolume {
  /// The device name (for example, `/dev/sda1` or `xvdh`).
  pub device_name: String,
  /// The size of the volume in GB
  pub size_gb: i32,
  /// The type of volume. Options: gp2, gp3, io1, io2.
  pub volume_type: AwsVolumeType,
  /// The iops of the volume, or 0 for AWS default.
  pub iops: i32,
  /// The throughput of the volume, or 0 for AWS default.
  pub throughput: i32,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
  Display,
  AsRefStr,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AwsVolumeType {
  #[default]
  Gp2,
  Gp3,
  Io1,
  Io2,
}

impl AwsServerTemplateConfig {
  pub fn from_builder_config(value: &AwsBuilderConfig) -> Self {
    Self {
      region: value.region.clone(),
      instance_type: value.instance_type.clone(),
      volumes: vec![AwsVolume {
        device_name: "/dev/sda1".to_string(),
        size_gb: value.volume_gb,
        volume_type: AwsVolumeType::Gp2,
        iops: 0,
        throughput: 0,
      }],
      ami_id: value.ami_id.clone(),
      subnet_id: value.subnet_id.clone(),
      security_group_ids: value.security_group_ids.clone(),
      key_pair_name: value.key_pair_name.clone(),
      assign_public_ip: value.assign_public_ip,
      use_public_ip: value.use_public_ip,
      port: value.port,
      use_https: value.use_https,
      user_data: value.user_data.clone(),
    }
  }
}
