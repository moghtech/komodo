use bson::{doc, Document};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::deserializers::{
  file_contents_deserializer, option_file_contents_deserializer,
  option_string_list_deserializer, string_list_deserializer,
};

use super::{
  resource::{Resource, ResourceListItem, ResourceQuery},
  ResourceTarget, I64,
};

#[typeshare]
pub type ResourceSyncListItem =
  ResourceListItem<ResourceSyncListItemInfo>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSyncListItemInfo {
  /// Unix timestamp of last sync, or 0
  pub last_sync_ts: I64,
  /// Whether sync is `files_on_host` mode.
  pub files_on_host: bool,
  /// Whether sync has file contents defined.
  pub file_contents: bool,
  /// Whether sync has `managed` mode enabled.
  pub managed: bool,
  /// Resource paths to the files.
  pub resource_path: Vec<String>,
  /// The git provider domain.
  pub git_provider: String,
  /// The Github repo used as the source of the sync resources
  pub repo: String,
  /// The branch of the repo
  pub branch: String,
  /// Short commit hash of last sync, or empty string
  pub last_sync_hash: Option<String>,
  /// Commit message of last sync, or empty string
  pub last_sync_message: Option<String>,
  /// State of the sync. Reflects whether most recent sync successful.
  pub state: ResourceSyncState,
}

#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, Display,
)]
pub enum ResourceSyncState {
  /// Last sync successful (or never synced). No Changes pending
  Ok,
  /// Last sync failed
  Failed,
  /// Currently syncing
  Syncing,
  /// Updates pending
  Pending,
  /// Other case
  #[default]
  Unknown,
}

#[typeshare]
pub type ResourceSync =
  Resource<ResourceSyncConfig, ResourceSyncInfo>;

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSyncInfo {
  /// Unix timestamp of last applied sync
  #[serde(default)]
  pub last_sync_ts: I64,
  /// Short commit hash of last applied sync
  pub last_sync_hash: Option<String>,
  /// Commit message of last applied sync
  pub last_sync_message: Option<String>,

  /// The list of pending updates to resources
  #[serde(default)]
  pub resource_updates: Vec<ResourceDiff>,
  /// The list of pending updates to variables
  #[serde(default)]
  pub variable_updates: Vec<DiffData>,
  /// The list of pending updates to user groups
  #[serde(default)]
  pub user_group_updates: Vec<DiffData>,
  /// The list of pending deploys to resources.
  #[serde(default)]
  pub pending_deploy: SyncDeployUpdate,
  /// If there is an error, it will be stored here
  pub pending_error: Option<String>,
  /// The commit hash which produced these pending updates.
  pub pending_hash: Option<String>,
  /// The commit message which produced these pending updates.
  pub pending_message: Option<String>,

  /// The current sync files
  #[serde(default)]
  pub remote_contents: Vec<SyncFileContents>,
  /// Any read errors in files by path
  #[serde(default)]
  pub remote_errors: Vec<SyncFileContents>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDiff {
  /// The resource target.
  /// The target id will be empty if "Create" ResourceDiffType.
  pub target: ResourceTarget,
  /// The data associated with the diff.
  pub data: DiffData,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DiffData {
  /// Resource will be created
  Create {
    /// The name of resource to create
    #[serde(default)]
    name: String,
    /// The proposed resource to create in TOML
    proposed: String,
  },
  Update {
    /// The proposed TOML
    proposed: String,
    /// The current TOML
    current: String,
  },
  Delete {
    /// The current TOML of the resource to delete
    current: String,
  },
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncDeployUpdate {
  /// Resources to deploy
  pub to_deploy: i32,
  /// A readable log of all the changes to be applied
  pub log: String,
}

#[typeshare(serialized_as = "Partial<ResourceSyncConfig>")]
pub type _PartialResourceSyncConfig = PartialResourceSyncConfig;

/// The sync configuration.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct ResourceSyncConfig {
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

  /// The Github repo used as the source of the build.
  #[serde(default)]
  #[builder(default)]
  pub repo: String,

  /// The branch of the repo.
  #[serde(default = "default_branch")]
  #[builder(default = "default_branch()")]
  #[partial_default(default_branch())]
  pub branch: String,

  /// Optionally set a specific commit hash.
  #[serde(default)]
  #[builder(default)]
  pub commit: String,

  /// The git account used to access private repos.
  /// Passing empty string can only clone public repos.
  ///
  /// Note. A token for the account must be available in the core config or the builder server's periphery config
  /// for the configured git provider.
  #[serde(default)]
  #[builder(default)]
  pub git_account: String,

  /// Whether incoming webhooks actually trigger action.
  #[serde(default = "default_webhook_enabled")]
  #[builder(default = "default_webhook_enabled()")]
  #[partial_default(default_webhook_enabled())]
  pub webhook_enabled: bool,

  /// Optionally provide an alternate webhook secret for this sync.
  /// If its an empty string, use the default secret from the config.
  #[serde(default)]
  #[builder(default)]
  pub webhook_secret: String,

  /// Files are available on the Komodo Core host.
  /// Specify the file / folder with [ResourceSyncConfig::resource_path].
  #[serde(default)]
  #[builder(default)]
  pub files_on_host: bool,

  /// The path of the resource file(s) to sync.
  ///  - If Files on Host, this is relative to the configured `sync_directory` in core config.
  ///  - If Git Repo based, this is relative to the root of the repo.
  /// Can be a specific file, or a directory containing multiple files / folders.
  /// See [https://komo.do/docs/sync-resources](https://komo.do/docs/sync-resources) for more information.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub resource_path: Vec<String>,

  /// Excluse Komodo Resources (Servers / Stacks / Builds)
  /// from the sync. Will be variable / user group only sync.
  #[serde(default)]
  #[builder(default)]
  pub exclude_resources: bool,

  /// Enable "pushes" to the file,
  /// which exports resources matching tags to single file.
  ///  - If using `files_on_host`, it is stored in the file_contents, which must point to a .toml file path (it will be created if it doesn't exist).
  ///  - If using `file_contents`, it is stored in the database.
  /// When using this, "delete" mode is always enabled.
  #[serde(default)]
  #[builder(default)]
  pub managed: bool,

  /// Whether sync should delete resources
  /// not declared in the resource files
  #[serde(default)]
  #[builder(default)]
  pub delete: bool,

  /// When using `managed` resource sync, will only export resources
  /// matching all of the given tags. If none, will match all resources.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub match_tags: Vec<String>,

  /// Manage the file contents in the UI.
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_file_contents_deserializer"
  ))]
  #[builder(default)]
  pub file_contents: String,
}

impl ResourceSyncConfig {
  pub fn builder() -> ResourceSyncConfigBuilder {
    ResourceSyncConfigBuilder::default()
  }

  /// Checks for empty file contents, ignoring whitespace / comments.
  pub fn file_contents_empty(&self) -> bool {
    self
      .file_contents
      .split('\n')
      .map(str::trim)
      .filter(|line| !line.is_empty() && !line.starts_with('#'))
      .count()
      == 0
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

fn default_webhook_enabled() -> bool {
  true
}

impl Default for ResourceSyncConfig {
  fn default() -> Self {
    Self {
      git_provider: default_git_provider(),
      git_https: default_git_https(),
      repo: Default::default(),
      branch: default_branch(),
      commit: Default::default(),
      git_account: Default::default(),
      resource_path: Default::default(),
      files_on_host: Default::default(),
      file_contents: Default::default(),
      exclude_resources: Default::default(),
      managed: Default::default(),
      match_tags: Default::default(),
      delete: Default::default(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
    }
  }
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncFileContents {
  /// The base resource path.
  #[serde(default)]
  pub resource_path: String,
  /// The path of the file / error path relative to the resource path.
  pub path: String,
  /// The contents of the file
  pub contents: String,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ResourceSyncActionState {
  /// Whether sync currently syncing
  pub syncing: bool,
}

#[typeshare]
pub type ResourceSyncQuery =
  ResourceQuery<ResourceSyncQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct ResourceSyncQuerySpecifics {
  /// Filter syncs by their repo.
  pub repos: Vec<String>,
}

impl super::resource::AddFilters for ResourceSyncQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if !self.repos.is_empty() {
      filters.insert("config.repo", doc! { "$in": &self.repos });
    }
  }
}
