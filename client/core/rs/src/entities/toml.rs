use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::{
  ResourceTarget, ResourceTargetVariant,
  action::_PartialActionConfig, alerter::_PartialAlerterConfig,
  build::_PartialBuildConfig, builder::_PartialBuilderConfig,
  deployment::_PartialDeploymentConfig, permission::PermissionLevel,
  procedure::_PartialProcedureConfig, repo::_PartialRepoConfig,
  server::_PartialServerConfig, stack::_PartialStackConfig,
  sync::_PartialResourceSyncConfig, variable::Variable,
};

/// Specifies resources to sync on Komodo
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesToml {
  #[serde(
    default,
    alias = "server",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub servers: Vec<ResourceToml<_PartialServerConfig>>,

  #[serde(
    default,
    alias = "deployment",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub deployments: Vec<ResourceToml<_PartialDeploymentConfig>>,

  #[serde(
    default,
    alias = "stack",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub stacks: Vec<ResourceToml<_PartialStackConfig>>,

  #[serde(
    default,
    alias = "build",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub builds: Vec<ResourceToml<_PartialBuildConfig>>,

  #[serde(
    default,
    alias = "repo",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub repos: Vec<ResourceToml<_PartialRepoConfig>>,

  #[serde(
    default,
    alias = "procedure",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub procedures: Vec<ResourceToml<_PartialProcedureConfig>>,

  #[serde(
    default,
    alias = "action",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub actions: Vec<ResourceToml<_PartialActionConfig>>,

  #[serde(
    default,
    alias = "alerter",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub alerters: Vec<ResourceToml<_PartialAlerterConfig>>,

  #[serde(
    default,
    alias = "builder",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub builders: Vec<ResourceToml<_PartialBuilderConfig>>,

  #[serde(
    default,
    alias = "resource_sync",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub resource_syncs: Vec<ResourceToml<_PartialResourceSyncConfig>>,

  #[serde(
    default,
    alias = "user_group",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub user_groups: Vec<UserGroupToml>,

  #[serde(
    default,
    alias = "variable",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub variables: Vec<Variable>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceToml<PartialConfig: Default> {
  /// The resource name. Required
  pub name: String,

  /// The resource description. Optional.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub description: String,

  /// Tag ids or names. Optional
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub tags: Vec<String>,

  /// Optional. Only relevant for deployments / stacks.
  ///
  /// Will ensure deployment / stack is running with the latest configuration.
  /// Deploy actions to achieve this will be included in the sync.
  /// Default is false.
  #[serde(default, skip_serializing_if = "is_false")]
  pub deploy: bool,

  /// Optional. Only relevant for deployments / stacks using the 'deploy' sync feature.
  ///
  /// Specify other deployments / stacks by name as dependencies.
  /// The sync will ensure the deployment / stack will only be deployed 'after' its dependencies.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub after: Vec<String>,

  /// Resource specific configuration.
  #[serde(default)]
  pub config: PartialConfig,
}

fn is_false(b: &bool) -> bool {
  !b
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroupToml {
  /// User group name
  pub name: String,

  /// Users in the group
  #[serde(default)]
  pub users: Vec<String>,

  /// Give the user group elevated permissions on all resources of a certain type
  #[serde(default)]
  pub all: HashMap<ResourceTargetVariant, PermissionLevel>,

  /// Permissions given to the group
  #[serde(default, alias = "permission")]
  pub permissions: Vec<PermissionToml>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionToml {
  /// Id can be:
  ///   - resource name. `id = "abcd-build"`
  ///   - regex matching resource names. `id = "\^(.+)-build-([0-9]+)$\"`
  pub target: ResourceTarget,

  /// The permission level:
  ///   - None
  ///   - Read
  ///   - Execute
  ///   - Write
  pub level: PermissionLevel,
}
