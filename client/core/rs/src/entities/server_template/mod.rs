use bson::{Document, doc};
use derive_default_builder::DefaultBuilder;
use derive_variants::EnumVariants;
use partial_derive2::{Diff, MaybeNone, PartialDiff};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use typeshare::typeshare;

use self::{
  aws::AwsServerTemplateConfig, hetzner::HetznerServerTemplateConfig,
};

use super::{
  MergePartial,
  resource::{AddFilters, Resource, ResourceListItem, ResourceQuery},
};

pub mod aws;
pub mod hetzner;

#[typeshare]
pub type ServerTemplate = Resource<ServerTemplateConfig, ()>;

#[typeshare]
pub type ServerTemplateListItem =
  ResourceListItem<ServerTemplateListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerTemplateListItemInfo {
  /// The cloud provider
  pub provider: String,
  /// The instance type, eg c5.2xlarge on for Aws templates
  pub instance_type: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, EnumVariants)]
#[variant_derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  Copy,
  Display,
  EnumString,
  AsRefStr
)]
#[serde(tag = "type", content = "params")]
pub enum ServerTemplateConfig {
  /// Template to launch an AWS EC2 instance
  Aws(aws::AwsServerTemplateConfig),
  /// Template to launch a Hetzner server
  Hetzner(hetzner::HetznerServerTemplateConfig),
}

impl Default for ServerTemplateConfig {
  fn default() -> Self {
    Self::Aws(Default::default())
  }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, EnumVariants)]
#[variant_derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  Copy,
  Display,
  EnumString,
  AsRefStr
)]
#[serde(tag = "type", content = "params")]
pub enum PartialServerTemplateConfig {
  Aws(#[serde(default)] aws::_PartialAwsServerTemplateConfig),
  Hetzner(
    #[serde(default)] hetzner::_PartialHetznerServerTemplateConfig,
  ),
}

impl Default for PartialServerTemplateConfig {
  fn default() -> Self {
    Self::Aws(Default::default())
  }
}

impl MaybeNone for PartialServerTemplateConfig {
  fn is_none(&self) -> bool {
    match self {
      PartialServerTemplateConfig::Aws(config) => config.is_none(),
      PartialServerTemplateConfig::Hetzner(config) => {
        config.is_none()
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerTemplateConfigDiff {
  Aws(aws::AwsServerTemplateConfigDiff),
  Hetzner(hetzner::HetznerServerTemplateConfigDiff),
}

impl From<ServerTemplateConfigDiff> for PartialServerTemplateConfig {
  fn from(value: ServerTemplateConfigDiff) -> Self {
    match value {
      ServerTemplateConfigDiff::Aws(diff) => {
        PartialServerTemplateConfig::Aws(diff.into())
      }
      ServerTemplateConfigDiff::Hetzner(diff) => {
        PartialServerTemplateConfig::Hetzner(diff.into())
      }
    }
  }
}

impl Diff for ServerTemplateConfigDiff {
  fn iter_field_diffs(
    &self,
  ) -> impl Iterator<Item = partial_derive2::FieldDiff> {
    match self {
      ServerTemplateConfigDiff::Aws(diff) => {
        diff.iter_field_diffs().collect::<Vec<_>>().into_iter()
      }
      ServerTemplateConfigDiff::Hetzner(diff) => {
        diff.iter_field_diffs().collect::<Vec<_>>().into_iter()
      }
    }
  }
}

impl
  PartialDiff<PartialServerTemplateConfig, ServerTemplateConfigDiff>
  for ServerTemplateConfig
{
  fn partial_diff(
    &self,
    partial: PartialServerTemplateConfig,
  ) -> ServerTemplateConfigDiff {
    match self {
      ServerTemplateConfig::Aws(original) => match partial {
        PartialServerTemplateConfig::Aws(partial) => {
          ServerTemplateConfigDiff::Aws(
            original.partial_diff(partial),
          )
        }
        PartialServerTemplateConfig::Hetzner(partial) => {
          let default = HetznerServerTemplateConfig::default();
          ServerTemplateConfigDiff::Hetzner(
            default.partial_diff(partial),
          )
        }
      },
      ServerTemplateConfig::Hetzner(original) => match partial {
        PartialServerTemplateConfig::Hetzner(partial) => {
          ServerTemplateConfigDiff::Hetzner(
            original.partial_diff(partial),
          )
        }
        PartialServerTemplateConfig::Aws(partial) => {
          let default = AwsServerTemplateConfig::default();
          ServerTemplateConfigDiff::Aws(default.partial_diff(partial))
        }
      },
    }
  }
}

impl MaybeNone for ServerTemplateConfigDiff {
  fn is_none(&self) -> bool {
    match self {
      ServerTemplateConfigDiff::Aws(config) => config.is_none(),
      ServerTemplateConfigDiff::Hetzner(config) => config.is_none(),
    }
  }
}

impl From<PartialServerTemplateConfig> for ServerTemplateConfig {
  fn from(
    value: PartialServerTemplateConfig,
  ) -> ServerTemplateConfig {
    match value {
      PartialServerTemplateConfig::Aws(config) => {
        ServerTemplateConfig::Aws(config.into())
      }
      PartialServerTemplateConfig::Hetzner(config) => {
        ServerTemplateConfig::Hetzner(config.into())
      }
    }
  }
}

impl From<ServerTemplateConfig> for PartialServerTemplateConfig {
  fn from(value: ServerTemplateConfig) -> Self {
    match value {
      ServerTemplateConfig::Aws(config) => {
        PartialServerTemplateConfig::Aws(config.into())
      }
      ServerTemplateConfig::Hetzner(config) => {
        PartialServerTemplateConfig::Hetzner(config.into())
      }
    }
  }
}

impl MergePartial for ServerTemplateConfig {
  type Partial = PartialServerTemplateConfig;
  fn merge_partial(
    self,
    partial: PartialServerTemplateConfig,
  ) -> ServerTemplateConfig {
    match partial {
      PartialServerTemplateConfig::Aws(partial) => match self {
        ServerTemplateConfig::Aws(config) => {
          let config = aws::AwsServerTemplateConfig {
            region: partial.region.unwrap_or(config.region),
            instance_type: partial
              .instance_type
              .unwrap_or(config.instance_type),
            volumes: partial.volumes.unwrap_or(config.volumes),
            ami_id: partial.ami_id.unwrap_or(config.ami_id),
            subnet_id: partial.subnet_id.unwrap_or(config.subnet_id),
            security_group_ids: partial
              .security_group_ids
              .unwrap_or(config.security_group_ids),
            key_pair_name: partial
              .key_pair_name
              .unwrap_or(config.key_pair_name),
            assign_public_ip: partial
              .assign_public_ip
              .unwrap_or(config.assign_public_ip),
            use_public_ip: partial
              .use_public_ip
              .unwrap_or(config.use_public_ip),
            port: partial.port.unwrap_or(config.port),
            use_https: partial.use_https.unwrap_or(config.use_https),
            user_data: partial.user_data.unwrap_or(config.user_data),
          };
          ServerTemplateConfig::Aws(config)
        }
        ServerTemplateConfig::Hetzner(_) => {
          ServerTemplateConfig::Aws(partial.into())
        }
      },
      PartialServerTemplateConfig::Hetzner(partial) => match self {
        ServerTemplateConfig::Hetzner(config) => {
          let config = hetzner::HetznerServerTemplateConfig {
            image: partial.image.unwrap_or(config.image),
            datacenter: partial
              .datacenter
              .unwrap_or(config.datacenter),
            private_network_ids: partial
              .private_network_ids
              .unwrap_or(config.private_network_ids),
            placement_group: partial
              .placement_group
              .unwrap_or(config.placement_group),
            enable_public_ipv4: partial
              .enable_public_ipv4
              .unwrap_or(config.enable_public_ipv4),
            enable_public_ipv6: partial
              .enable_public_ipv6
              .unwrap_or(config.enable_public_ipv6),
            firewall_ids: partial
              .firewall_ids
              .unwrap_or(config.firewall_ids),
            server_type: partial
              .server_type
              .unwrap_or(config.server_type),
            ssh_keys: partial.ssh_keys.unwrap_or(config.ssh_keys),
            user_data: partial.user_data.unwrap_or(config.user_data),
            use_public_ip: partial
              .use_public_ip
              .unwrap_or(config.use_public_ip),
            labels: partial.labels.unwrap_or(config.labels),
            volumes: partial.volumes.unwrap_or(config.volumes),
            port: partial.port.unwrap_or(config.port),
            use_https: partial.use_https.unwrap_or(config.use_https),
          };
          ServerTemplateConfig::Hetzner(config)
        }
        ServerTemplateConfig::Aws(_) => {
          ServerTemplateConfig::Hetzner(partial.into())
        }
      },
    }
  }
}

#[typeshare]
pub type ServerTemplateQuery =
  ResourceQuery<ServerTemplateQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct ServerTemplateQuerySpecifics {
  pub types: Vec<ServerTemplateConfigVariant>,
}

impl AddFilters for ServerTemplateQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    let types =
      self.types.iter().map(|t| t.as_ref()).collect::<Vec<_>>();
    if !self.types.is_empty() {
      filters.insert("config.type", doc! { "$in": types });
    }
  }
}
