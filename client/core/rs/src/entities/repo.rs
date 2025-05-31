use anyhow::Context;
use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    env_vars_deserializer, option_env_vars_deserializer,
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::I64,
};

use super::{
  EnvironmentVar, SystemCommand, environment_vars_from_str,
  resource::{Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type RepoListItem = ResourceListItem<RepoListItemInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RepoListItemInfo {
  /// The server that repo sits on.
  pub server_id: String,
  /// The builder that builds the repo.
  pub builder_id: String,
  /// Repo last cloned / pulled timestamp in ms.
  pub last_pulled_at: I64,
  /// Repo last built timestamp in ms.
  pub last_built_at: I64,
  /// The git provider domain
  pub git_provider: String,
  /// The configured repo
  pub repo: String,
  /// The configured branch
  pub branch: String,
  /// Full link to the repo.
  pub repo_link: String,
  /// The repo state
  pub state: RepoState,
  /// If the repo is cloned, will be the cloned short commit hash.
  pub cloned_hash: Option<String>,
  /// If the repo is cloned, will be the cloned commit message.
  pub cloned_message: Option<String>,
  /// If the repo is built, will be the latest built short commit hash.
  pub built_hash: Option<String>,
  /// Will be the latest remote short commit hash.
  pub latest_hash: Option<String>,
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, Display,
)]
pub enum RepoState {
  /// Unknown case
  #[default]
  Unknown,
  /// Last clone / pull successful (or never cloned)
  Ok,
  /// Last clone / pull failed
  Failed,
  /// Currently cloning
  Cloning,
  /// Currently pulling
  Pulling,
  /// Currently building
  Building,
}

#[typeshare]
pub type Repo = Resource<RepoConfig, RepoInfo>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RepoInfo {
  /// When repo was last pulled
  #[serde(default)]
  pub last_pulled_at: I64,
  /// When repo was last built
  #[serde(default)]
  pub last_built_at: I64,
  /// Latest built short commit hash, or null.
  pub built_hash: Option<String>,
  /// Latest built commit message, or null. Only for repo based stacks
  pub built_message: Option<String>,
  /// Latest remote short commit hash, or null.
  pub latest_hash: Option<String>,
  /// Latest remote commit message, or null
  pub latest_message: Option<String>,
}

#[typeshare(serialized_as = "Partial<RepoConfig>")]
pub type _PartialRepoConfig = PartialRepoConfig;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Partial)]
#[partial_derive(Serialize, Deserialize, Debug, Clone, Default)]
#[partial(skip_serializing_none, from, diff)]
pub struct RepoConfig {
  /// The server to clone the repo on.
  #[serde(default, alias = "server")]
  #[partial_attr(serde(alias = "server"))]
  #[builder(default)]
  pub server_id: String,

  /// Attach a builder to 'build' the repo.
  #[serde(default, alias = "builder")]
  #[partial_attr(serde(alias = "builder"))]
  #[builder(default)]
  pub builder_id: String,

  /// The git provider domain. Default: github.com
  #[serde(default = "default_git_provider")]
  #[builder(default = "default_git_provider()")]
  #[partial_default(default_git_provider())]
  pub git_provider: String,

  /// Whether to use https to clone the repo (versus http). Default: true
  ///
  /// Note. Komodo does not currently support cloning repos via ssh.
  #[serde(default = "default_git_https")]
  #[builder(default = "default_git_https()")]
  #[partial_default(default_git_https())]
  pub git_https: bool,

  /// The git account used to access private repos.
  /// Passing empty string can only clone public repos.
  ///
  /// Note. A token for the account must be available in the core config or the builder server's periphery config
  /// for the configured git provider.
  #[serde(default)]
  #[builder(default)]
  pub git_account: String,

  /// The github repo to clone.
  #[serde(default)]
  #[builder(default)]
  pub repo: String,

  /// The repo branch.
  #[serde(default = "default_branch")]
  #[builder(default = "default_branch()")]
  #[partial_default(default_branch())]
  pub branch: String,

  /// Optionally set a specific commit hash.
  #[serde(default)]
  #[builder(default)]
  pub commit: String,

  /// Explicitly specify the folder to clone the repo in.
  #[serde(default)]
  #[builder(default)]
  pub path: String,

  /// Whether incoming webhooks actually trigger action.
  #[serde(default = "default_webhook_enabled")]
  #[builder(default = "default_webhook_enabled()")]
  #[partial_default(default_webhook_enabled())]
  pub webhook_enabled: bool,

  /// Optionally provide an alternate webhook secret for this repo.
  /// If its an empty string, use the default secret from the config.
  #[serde(default)]
  #[builder(default)]
  pub webhook_secret: String,

  /// Command to be run after the repo is cloned.
  /// The path is relative to the root of the repo.
  #[serde(default)]
  #[builder(default)]
  pub on_clone: SystemCommand,

  /// Command to be run after the repo is pulled.
  /// The path is relative to the root of the repo.
  #[serde(default)]
  #[builder(default)]
  pub on_pull: SystemCommand,

  /// Configure quick links that are displayed in the resource header
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub links: Vec<String>,

  /// The environment variables passed to the compose file.
  /// They will be written to path defined in env_file_path,
  /// which is given relative to the run directory.
  ///
  /// If it is empty, no file will be written.
  #[serde(default, deserialize_with = "env_vars_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_env_vars_deserializer"
  ))]
  #[builder(default)]
  pub environment: String,

  /// The name of the written environment file before `docker compose up`.
  /// Relative to the repo root.
  /// Default: .env
  #[serde(default = "default_env_file_path")]
  #[builder(default = "default_env_file_path()")]
  #[partial_default(default_env_file_path())]
  pub env_file_path: String,

  /// Whether to skip secret interpolation into the repo environment variable file.
  #[serde(default)]
  #[builder(default)]
  pub skip_secret_interp: bool,
}

impl RepoConfig {
  pub fn builder() -> RepoConfigBuilder {
    RepoConfigBuilder::default()
  }

  pub fn env_vars(&self) -> anyhow::Result<Vec<EnvironmentVar>> {
    environment_vars_from_str(&self.environment)
      .context("Invalid environment")
  }
}

fn default_git_provider() -> String {
  String::from("github.com")
}

fn default_git_https() -> bool {
  true
}

fn default_branch() -> String {
  String::from("main")
}

fn default_env_file_path() -> String {
  String::from(".env")
}

fn default_webhook_enabled() -> bool {
  true
}

impl Default for RepoConfig {
  fn default() -> Self {
    Self {
      server_id: Default::default(),
      builder_id: Default::default(),
      git_provider: default_git_provider(),
      git_https: default_git_https(),
      repo: Default::default(),
      branch: default_branch(),
      commit: Default::default(),
      git_account: Default::default(),
      path: Default::default(),
      on_clone: Default::default(),
      on_pull: Default::default(),
      links: Default::default(),
      environment: Default::default(),
      env_file_path: default_env_file_path(),
      skip_secret_interp: Default::default(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
    }
  }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct RepoActionState {
  /// Whether Repo currently cloning on the attached Server
  pub cloning: bool,
  /// Whether Repo currently pulling on the attached Server
  pub pulling: bool,
  /// Whether Repo currently building using the attached Builder.
  pub building: bool,
  /// Whether Repo currently renaming.
  pub renaming: bool,
}

#[typeshare]
pub type RepoQuery = ResourceQuery<RepoQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct RepoQuerySpecifics {
  /// Filter repos by their repo.
  pub repos: Vec<String>,
}

impl super::resource::AddFilters for RepoQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if !self.repos.is_empty() {
      filters.insert("config.repo", doc! { "$in": &self.repos });
    }
  }
}
