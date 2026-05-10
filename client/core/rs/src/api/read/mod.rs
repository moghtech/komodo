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

#[cfg(feature = "utoipa")]
pub mod openapi;

pub trait KomodoReadRequest: HasResponse {}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetVersion",
  description = "Get the version of the Komodo Core API.",
  request_body(content = GetVersion),
  responses(
    (status = 200, description = "Komodo Core version", body = GetVersionResponse),
  ),
)]
pub fn get_version() {}

/// Get the version of the Komodo Core API.
/// Response: [GetVersionResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetVersionResponse)]
#[error(mogh_error::Error)]
pub struct GetVersion {}

/// Response for [GetVersion].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetVersionResponse {
  /// The version of the Komodo Core API.
  pub version: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/GetCoreInfo",
  description = "Get information about the Komodo Core API configuration.",
  request_body(content = GetCoreInfo),
  responses(
    (status = 200, description = "Komodo Core info", body = GetCoreInfoResponse),
  ),
)]
pub fn get_core_info() {}

/// Get information about the Komodo Core API configuration.
/// Response: [GetCoreInfoResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetCoreInfoResponse)]
#[error(mogh_error::Error)]
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
  /// Public key for Core / Periphery authentication.
  pub public_key: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListGitProvidersFromConfig",
  description = "List the git providers available in Core / Periphery config files.",
  request_body(content = ListGitProvidersFromConfig),
  responses(
    (status = 200, description = "The available git providers", body = ListGitProvidersFromConfigResponse),
    (status = 400, description = "Target must be `Server` or `Builder`", body = mogh_error::Serror),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn list_git_providers_from_config() {}

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
#[error(mogh_error::Error)]
pub struct ListGitProvidersFromConfig {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListGitProvidersFromConfigResponse = Vec<GitProvider>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListDockerRegistriesFromConfig",
  description = "List the docker registry providers available in Core / Periphery config files.",
  request_body(content = ListDockerRegistriesFromConfig),
  responses(
    (status = 200, description = "The available docker registries", body = ListDockerRegistriesFromConfigResponse),
    (status = 400, description = "Target must be `Server` or `Builder`", body = mogh_error::Serror),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn list_docker_registries_from_config() {}

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
#[error(mogh_error::Error)]
pub struct ListDockerRegistriesFromConfig {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListDockerRegistriesFromConfigResponse = Vec<DockerRegistry>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/ListSecrets",
  description = "List the secret keys (not values) in the core configuration file.",
  request_body(content = ListSecrets),
  responses(
    (status = 200, description = "The available secret keys", body = ListSecretsResponse),
    (status = 400, description = "Target must be `Server` or `Builder`", body = mogh_error::Serror),
    (status = 500, description = "Failed", body = mogh_error::Serror),
  ),
)]
pub fn list_secrets() {}

/// List the secret keys (not values) in the core configuration file.
/// Response: [ListSecretsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSecretsResponse)]
#[error(mogh_error::Error)]
pub struct ListSecrets {
  /// Accepts an optional Server or Builder target to expand the core list with
  /// providers available on that specific resource.
  pub target: Option<ResourceTarget>,
}

#[typeshare]
pub type ListSecretsResponse = Vec<String>;
