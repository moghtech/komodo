use mogh_resolver::{HasResponse, Resolve};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

mod action;
mod alert;
mod alerter;
mod build;
mod builder;
mod deployment;
mod docker;
mod onboarding_key;
mod permission;
mod procedure;
mod provider;
mod repo;
mod schedule;
mod server;
mod stack;
mod swarm;
mod sync;
mod tag;
mod terminal;
mod toml;
mod update;
mod user;
mod user_group;
mod variable;

pub use action::*;
pub use alert::*;
pub use alerter::*;
pub use build::*;
pub use builder::*;
pub use deployment::*;
pub use docker::*;
pub use onboarding_key::*;
pub use permission::*;
pub use procedure::*;
pub use provider::*;
pub use repo::*;
pub use schedule::*;
pub use server::*;
pub use stack::*;
pub use swarm::*;
pub use sync::*;
pub use tag::*;
pub use terminal::*;
pub use toml::*;
pub use update::*;
pub use user::*;
pub use user_group::*;
pub use variable::*;

use crate::entities::{
  ResourceTarget, Timelength,
  config::{DockerRegistry, GitProvider},
};

pub trait KomodoReadRequest: HasResponse {}

//

/// Get the version of the Komodo Core api.
/// Response: [GetVersionResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetVersionResponse)]
#[error(serror::Error)]
pub struct GetVersion {}

/// Response for [GetVersion].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetVersionResponse {
  /// The version of the core api.
  pub version: String,
}

//

/// Get info about the core api configuration.
/// Response: [GetCoreInfoResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetCoreInfoResponse)]
#[error(serror::Error)]
pub struct GetCoreInfo {}

/// Response for [GetCoreInfo].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetCoreInfoResponse {
  /// The title assigned to this core api.
  pub title: String,
  /// The monitoring interval of this core api.
  pub monitoring_interval: Timelength,
  /// The webhook base url.
  pub webhook_base_url: String,
  /// Whether transparent mode is enabled, which gives all users read access to all resources.
  pub transparent_mode: bool,
  /// Whether UI write access should be disabled
  pub ui_write_disabled: bool,
  /// Whether non admins can create resources
  pub disable_non_admin_create: bool,
  /// Whether confirm dialog should be disabled
  pub disable_confirm_dialog: bool,
  /// Whether to disable websocket automatic reconnect.
  pub disable_websocket_reconnect: bool,
  /// Whether to enable fancy toml highlighting.
  pub enable_fancy_toml: bool,
  /// TZ identifier Core is using, if manually set.
  pub timezone: String,
  /// Default public key allowing this Core to authenticate to Periphery agents.
  pub public_key: String,
}

//

/// List the git providers available in Core / Periphery config files.
/// Response: [ListGitProvidersFromConfigResponse].
///
/// Includes:
///   - providers in core config
///   - providers configured on builds, repos, syncs
///   - providers on the optional Server or Builder
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListGitProvidersFromConfigResponse)]
#[error(serror::Error)]
pub struct ListGitProvidersFromConfig {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListGitProvidersFromConfigResponse = Vec<GitProvider>;

//

/// List the docker registry providers available in Core / Periphery config files.
/// Response: [ListDockerRegistriesFromConfigResponse].
///
/// Includes:
///   - registries in core config
///   - registries configured on builds, deployments
///   - registries on the optional Server or Builder
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListDockerRegistriesFromConfigResponse)]
#[error(serror::Error)]
pub struct ListDockerRegistriesFromConfig {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListDockerRegistriesFromConfigResponse = Vec<DockerRegistry>;

//

/// List the available secrets from the core config.
/// Response: [ListSecretsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSecretsResponse)]
#[error(serror::Error)]
pub struct ListSecrets {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListSecretsResponse = Vec<String>;
