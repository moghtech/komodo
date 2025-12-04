use std::{collections::HashMap, sync::OnceLock};

use anyhow::Context;
use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use indexmap::IndexSet;
use partial_derive2::Partial;
use serde::{
  Deserialize, Serialize,
  de::{IntoDeserializer, Visitor, value::MapAccessDeserializer},
};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    env_vars_deserializer, file_contents_deserializer,
    option_env_vars_deserializer, option_file_contents_deserializer,
    option_maybe_string_i64_deserializer,
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::{EnvironmentVar, environment_vars_from_str},
};

use super::{
  FileContents, SystemCommand,
  docker::container::ContainerListItem,
  resource::{Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type Stack = Resource<StackConfig, StackInfo>;

impl Stack {
  /// If fresh is passed, it will bypass the deployed project name.
  /// and get the most up to date one from just project_name field falling back to stack name.
  pub fn project_name(&self, fresh: bool) -> String {
    if !fresh
      && let Some(project_name) = &self.info.deployed_project_name
    {
      return project_name.clone();
    }
    if self.config.project_name.is_empty() {
      self.name.clone()
    } else {
      self.config.project_name.clone()
    }
  }

  pub fn compose_file_paths(&self) -> &[String] {
    if self.config.file_paths.is_empty() {
      default_stack_file_paths()
    } else {
      &self.config.file_paths
    }
  }

  pub fn is_compose_file(&self, path: &str) -> bool {
    // First make sure its not a config file, which *could* also include
    // other compose files not directly treated as compose.
    if self.is_config_file(path) {
      return false;
    }
    for compose_path in self.compose_file_paths() {
      if path.ends_with(compose_path) {
        return true;
      }
    }
    false
  }

  pub fn is_config_file(&self, path: &str) -> bool {
    for file in &self.config.config_files {
      if path.ends_with(&file.path) {
        return true;
      }
    }
    false
  }

  /// Get tracked additional env files (those that Komodo should manage)
  fn tracked_env_files(&self) -> impl Iterator<Item = &str> {
    self
      .config
      .additional_env_files
      .iter()
      .filter(|f| f.track)
      .map(|f| f.path.as_str())
  }

  pub fn all_tracked_file_paths(&self) -> Vec<String> {
    let mut res = self
      .compose_file_paths()
      .iter()
      .cloned()
      // Makes sure to dedup them, while maintaining ordering
      .collect::<IndexSet<_>>();
    res.extend(self.tracked_env_files().map(str::to_string));
    res.extend(
      self.config.config_files.iter().map(|f| f.path.clone()),
    );
    res.into_iter().collect()
  }

  pub fn all_file_dependencies(&self) -> Vec<StackFileDependency> {
    let mut res = self
      .compose_file_paths()
      .iter()
      .cloned()
      .map(StackFileDependency::full_redeploy)
      // Makes sure to dedup them, while maintaining ordering
      .collect::<IndexSet<_>>();
    res.extend(
      self
        .tracked_env_files()
        .map(|p| StackFileDependency::full_redeploy(p.to_string())),
    );
    res.extend(self.config.config_files.clone());
    res.into_iter().collect()
  }
}

fn default_stack_file_paths() -> &'static [String] {
  static DEFAULT_FILE_PATHS: OnceLock<Vec<String>> = OnceLock::new();
  DEFAULT_FILE_PATHS
    .get_or_init(|| vec![String::from("compose.yaml")])
}

#[typeshare]
pub type StackListItem = ResourceListItem<StackListItemInfo>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackListItemInfo {
  /// The server that stack is deployed on.
  pub server_id: String,
  /// Whether stack is using files on host mode
  pub files_on_host: bool,
  /// Whether stack has file contents defined.
  pub file_contents: bool,
  /// Linked repo, if one is attached.
  pub linked_repo: String,
  /// The git provider domain
  pub git_provider: String,
  /// The configured repo
  pub repo: String,
  /// The configured branch
  pub branch: String,
  /// Full link to the repo.
  pub repo_link: String,
  /// The stack state
  pub state: StackState,
  /// A string given by docker conveying the status of the stack.
  pub status: Option<String>,
  /// The services that are part of the stack.
  /// If deployed, will be `deployed_services`.
  /// Otherwise, its `latest_services`
  pub services: Vec<StackServiceWithUpdate>,
  /// Whether the compose project is missing on the host.
  /// Ie, it does not show up in `docker compose ls`.
  /// If true, and the stack is not Down, this is an unhealthy state.
  pub project_missing: bool,
  /// If any compose files are missing in the repo, the path will be here.
  /// If there are paths here, this is an unhealthy state, and deploying will fail.
  pub missing_files: Vec<String>,
  /// Deployed short commit hash, or null. Only for repo based stacks.
  pub deployed_hash: Option<String>,
  /// Latest short commit hash, or null. Only for repo based stacks
  pub latest_hash: Option<String>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackServiceWithUpdate {
  pub service: String,
  /// The service's image
  pub image: String,
  /// Whether there is a newer image available for this service
  pub update_available: bool,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
  Display,
)]
// Do this one snake_case in line with DeploymentState.
// Also in line with docker terminology.
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StackState {
  /// The stack is currently re/deploying
  Deploying,
  /// All containers are running.
  Running,
  /// All containers are paused
  Paused,
  /// All contianers are stopped
  Stopped,
  /// All containers are created
  Created,
  /// All containers are restarting
  Restarting,
  /// All containers are dead
  Dead,
  /// All containers are removing
  Removing,
  /// The containers are in a mix of states
  Unhealthy,
  /// The stack is not deployed
  Down,
  /// Server not reachable for status
  #[default]
  Unknown,
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackInfo {
  /// If any of the expected compose / additional files are missing in the repo,
  /// they will be stored here.
  #[serde(default)]
  pub missing_files: Vec<String>,

  /// The deployed project name.
  /// This is updated whenever Komodo successfully deploys the stack.
  /// If it is present, Komodo will use it for actions over other options,
  /// to ensure control is maintained after changing the project name (there is no rename compose project api).
  pub deployed_project_name: Option<String>,

  /// Deployed short commit hash, or null. Only for repo based stacks.
  pub deployed_hash: Option<String>,
  /// Deployed commit message, or null. Only for repo based stacks
  pub deployed_message: Option<String>,
  /// The deployed compose / additional file contents.
  /// This is updated whenever Komodo successfully deploys the stack.
  pub deployed_contents: Option<Vec<FileContents>>,
  /// The deployed service names.
  /// This is updated whenever it is empty, or deployed contents is updated.
  pub deployed_services: Option<Vec<StackServiceNames>>,
  /// The output of `docker compose config`.
  /// This is updated whenever Komodo successfully deploys the stack.
  pub deployed_config: Option<String>,
  /// The latest service names.
  /// This is updated whenever the stack cache refreshes, using the latest file contents (either db defined or remote).
  #[serde(default)]
  pub latest_services: Vec<StackServiceNames>,

  /// The remote compose / additional file contents, whether on host or in repo.
  /// This is updated whenever Komodo refreshes the stack cache.
  /// It will be empty if the file is defined directly in the stack config.
  pub remote_contents: Option<Vec<StackRemoteFileContents>>,
  /// If there was an error in getting the remote contents, it will be here.
  pub remote_errors: Option<Vec<FileContents>>,

  /// Latest commit hash, or null
  pub latest_hash: Option<String>,
  /// Latest commit message, or null
  pub latest_message: Option<String>,
}

#[typeshare(serialized_as = "Partial<StackConfig>")]
pub type _PartialStackConfig = PartialStackConfig;

/// The compose file configuration.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct StackConfig {
  /// The Swarm to deploy the Stack on, setting the Stack into Swarm mode.
  ///
  /// Note. If both swarm_id and server_id are set,
  /// swarm_id overrides server_id and the Stack will be in Swarm mode.
  #[serde(default, alias = "swarm")]
  #[partial_attr(serde(alias = "swarm"))]
  #[builder(default)]
  pub swarm_id: String,

  /// The Server to deploy the Stack on, setting the Stack into Compose mode.
  ///
  /// Note. If both swarm_id and server_id are set,
  /// swarm_id overrides server_id and the Stack will be in Swarm mode.
  #[serde(default, alias = "server")]
  #[partial_attr(serde(alias = "server"))]
  #[builder(default)]
  pub server_id: String,

  /// Configure quick links that are displayed in the resource header
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub links: Vec<String>,

  /// Optionally specify a custom project name for the stack.
  /// If this is empty string, it will default to the stack name.
  /// Used with `docker compose -p {project_name}` / `docker stack deploy {project_name}`.
  ///
  /// Note. Can be used to import pre-existing stacks with names that do not match Stack name.
  #[serde(default)]
  #[builder(default)]
  pub project_name: String,

  /// Whether to automatically `compose pull` before redeploying stack.
  /// Ensured latest images are deployed.
  /// Will fail if the compose file specifies a locally build image.
  ///
  /// Note. Not used in Swarm mode.
  #[serde(default = "default_auto_pull")]
  #[builder(default = "default_auto_pull()")]
  #[partial_default(default_auto_pull())]
  pub auto_pull: bool,

  /// Whether to `docker compose build` before `compose down` / `compose up`.
  /// Combine with build_extra_args for custom behaviors.
  ///
  /// Note. Not used in Swarm mode.
  #[serde(default)]
  #[builder(default)]
  pub run_build: bool,

  /// Whether to poll for any updates to the images.
  #[serde(default)]
  #[builder(default)]
  pub poll_for_updates: bool,

  /// Whether to automatically redeploy when
  /// newer images are found. Will implicitly
  /// enable `poll_for_updates`, you don't need to
  /// enable both.
  #[serde(default)]
  #[builder(default)]
  pub auto_update: bool,

  /// If auto update is enabled, Komodo will
  /// by default only update the specific services
  /// with image updates. If this parameter is set to true,
  /// Komodo will redeploy the whole Stack (all services).
  #[serde(default)]
  #[builder(default)]
  pub auto_update_all_services: bool,

  /// Whether to run `docker compose down` before `compose up`.
  #[serde(default)]
  #[builder(default)]
  pub destroy_before_deploy: bool,

  /// Whether to skip secret interpolation into the stack environment variables.
  #[serde(default)]
  #[builder(default)]
  pub skip_secret_interp: bool,

  /// Choose a Komodo Repo (Resource) to source the compose files.
  #[serde(default)]
  #[builder(default)]
  pub linked_repo: String,

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

  /// The repo used as the source of the build.
  /// {namespace}/{repo_name}
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

  /// Optionally set a specific clone path
  #[serde(default)]
  #[builder(default)]
  pub clone_path: String,

  /// By default, the Stack will `git pull` the repo after it is first cloned.
  /// If this option is enabled, the repo folder will be deleted and recloned instead.
  #[serde(default)]
  #[builder(default)]
  pub reclone: bool,

  /// Whether incoming webhooks actually trigger action.
  #[serde(default = "default_webhook_enabled")]
  #[builder(default = "default_webhook_enabled()")]
  #[partial_default(default_webhook_enabled())]
  pub webhook_enabled: bool,

  /// Optionally provide an alternate webhook secret for this stack.
  /// If its an empty string, use the default secret from the config.
  #[serde(default)]
  #[builder(default)]
  pub webhook_secret: String,

  /// By default, the Stack will `DeployStackIfChanged`.
  /// If this option is enabled, will always run `DeployStack` without diffing.
  #[serde(default)]
  #[builder(default)]
  pub webhook_force_deploy: bool,

  /// If this is checked, the stack will source the files on the host.
  /// Use `run_directory` and `file_paths` to specify the path on the host.
  /// This is useful for those who wish to setup their files on the host,
  /// rather than defining the contents in UI or in a git repo.
  #[serde(default)]
  #[builder(default)]
  pub files_on_host: bool,

  /// Directory to change to (`cd`) before running `docker compose up -d`.
  #[serde(default)]
  #[builder(default)]
  pub run_directory: String,

  /// Add paths to compose files, relative to the run path.
  /// If this is empty, will use file `compose.yaml`.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub file_paths: Vec<String>,

  /// The name of the written environment file before `docker compose up`.
  /// Relative to the run directory root.
  /// Default: .env
  ///
  /// Note. Not used in Swarm mode.
  #[serde(default = "default_env_file_path")]
  #[builder(default = "default_env_file_path()")]
  #[partial_default(default_env_file_path())]
  pub env_file_path: String,

  /// Add additional env files to attach with `--env-file`.
  /// Relative to the run directory root.
  ///
  /// Note. It is already included as an `additional_file`.
  /// Don't add it again there.
  #[serde(default)]
  #[partial_attr(serde(default))]
  #[builder(default)]
  pub additional_env_files: Vec<AdditionalEnvFile>,

  /// Add additional config files either in repo or on host to track.
  /// Can add any files associated with the stack to enable editing them in the UI.
  /// Doing so will also include diffing these when deciding to deploy in `DeployStackIfChanged`.
  /// Relative to the run directory.
  ///
  /// Note. If the config file is .env and should be included in compose command
  /// using `--env-file`, add it to `additional_env_files` instead.
  #[serde(default)]
  #[partial_attr(serde(default))]
  #[builder(default)]
  pub config_files: Vec<StackFileDependency>,

  /// Whether to send StackStateChange alerts for this stack.
  #[serde(default = "default_send_alerts")]
  #[builder(default = "default_send_alerts()")]
  #[partial_default(default_send_alerts())]
  pub send_alerts: bool,

  /// Used with `registry_account` to login to a registry before docker compose up.
  #[serde(default)]
  #[builder(default)]
  pub registry_provider: String,

  /// Used with `registry_provider` to login to a registry before docker compose up.
  #[serde(default)]
  #[builder(default)]
  pub registry_account: String,

  /// The optional command to run before the Stack is deployed.
  #[serde(default)]
  #[builder(default)]
  pub pre_deploy: SystemCommand,

  /// The optional command to run after the Stack is deployed.
  #[serde(default)]
  #[builder(default)]
  pub post_deploy: SystemCommand,

  /// The extra arguments to pass to the deploy command.
  ///
  /// - For Compose stack, uses `docker compose up -d [EXTRA_ARGS]`.
  /// - For Swarm mode. `docker stack deploy [EXTRA_ARGS] STACK_NAME`
  ///
  /// If empty, no extra arguments will be passed.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub extra_args: Vec<String>,

  /// The extra arguments to pass after `docker compose build`.
  /// If empty, no extra build arguments will be passed.
  /// Only used if `run_build: true`
  ///
  /// Note. Not used in Swarm mode.
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub build_extra_args: Vec<String>,

  /// Optional command wrapper for secrets management tools.
  /// Wraps the docker compose up command with a prefix command.
  /// Use [[COMPOSE_COMMAND]] as placeholder for the full compose command.
  ///
  /// Examples:
  /// - "op run -- [[COMPOSE_COMMAND]]" (1password CLI)
  /// - "sops exec-file --no-fifo /path/to/secret.env '[[COMPOSE_COMMAND]]'" (sops)
  #[serde(default)]
  #[builder(default)]
  pub compose_cmd_wrapper: String,

  /// Ignore certain services declared in the compose file when checking
  /// the stack status. For example, an init service might be exited, but the
  /// stack should be healthy. This init service should be in `ignore_services`
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub ignore_services: Vec<String>,

  /// The contents of the file directly, for management in the UI.
  /// If this is empty, it will fall back to checking git config for
  /// repo based compose file.
  /// Supports variable / secret interpolation.
  #[serde(default, deserialize_with = "file_contents_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_file_contents_deserializer"
  ))]
  #[builder(default)]
  pub file_contents: String,

  /// The environment variables passed to the compose file.
  /// They will be written to path defined in env_file_path,
  /// which is given relative to the run directory.
  ///
  /// If it is empty, no file will be written.
  ///
  /// Note. Not used in Swarm mode.
  #[serde(default, deserialize_with = "env_vars_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_env_vars_deserializer"
  ))]
  #[builder(default)]
  pub environment: String,
}

impl StackConfig {
  pub fn builder() -> StackConfigBuilder {
    StackConfigBuilder::default()
  }

  pub fn env_vars(&self) -> anyhow::Result<Vec<EnvironmentVar>> {
    environment_vars_from_str(&self.environment)
      .context("Invalid environment")
  }
}

fn default_env_file_path() -> String {
  String::from(".env")
}

fn default_auto_pull() -> bool {
  true
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

fn default_send_alerts() -> bool {
  true
}

impl Default for StackConfig {
  fn default() -> Self {
    Self {
      swarm_id: Default::default(),
      server_id: Default::default(),
      project_name: Default::default(),
      run_directory: Default::default(),
      file_paths: Default::default(),
      files_on_host: Default::default(),
      registry_provider: Default::default(),
      registry_account: Default::default(),
      file_contents: Default::default(),
      auto_pull: default_auto_pull(),
      poll_for_updates: Default::default(),
      auto_update: Default::default(),
      auto_update_all_services: Default::default(),
      ignore_services: Default::default(),
      pre_deploy: Default::default(),
      post_deploy: Default::default(),
      extra_args: Default::default(),
      environment: Default::default(),
      env_file_path: default_env_file_path(),
      additional_env_files: Default::default(),
      config_files: Default::default(),
      run_build: Default::default(),
      destroy_before_deploy: Default::default(),
      build_extra_args: Default::default(),
      compose_cmd_wrapper: Default::default(),
      skip_secret_interp: Default::default(),
      linked_repo: Default::default(),
      git_provider: default_git_provider(),
      git_https: default_git_https(),
      repo: Default::default(),
      branch: default_branch(),
      commit: Default::default(),
      clone_path: Default::default(),
      reclone: Default::default(),
      git_account: Default::default(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
      webhook_force_deploy: Default::default(),
      send_alerts: default_send_alerts(),
      links: Default::default(),
    }
  }
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ComposeProject {
  /// The compose project name.
  pub name: String,
  /// The status of the project, as returned by docker.
  pub status: Option<String>,
  /// The compose files included in the project.
  pub compose_files: Vec<String>,
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackServiceNames {
  /// The name of the service
  pub service_name: String,
  /// Will either be the declared container_name in the compose file,
  /// or a pattern to match auto named containers.
  ///
  /// Auto named containers are composed of three parts:
  ///
  /// 1. The name of the compose project (top level name field of compose file).
  ///    This defaults to the name of the parent folder of the compose file.
  ///    Komodo will always set it to be the name of the stack, but imported stacks
  ///    will have a different name.
  /// 2. The service name
  /// 3. The replica number
  ///
  /// Example: stacko-mongo-1.
  ///
  /// This stores only 1. and 2., ie stacko-mongo.
  /// Containers will be matched via regex like `^container_name-?[0-9]*$``
  ///
  /// Note. Setting container_name is not supported by Swarm,
  /// so will always be 1. and 2. in Swarm mode.
  pub container_name: String,
  /// The services image.
  #[serde(default)]
  pub image: String,
}

#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackService {
  /// The service name
  pub service: String,
  /// The service image
  pub image: String,
  /// The container
  pub container: Option<ContainerListItem>,
  /// Whether there is an update available for this services image.
  pub update_available: bool,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct StackActionState {
  pub pulling: bool,
  pub deploying: bool,
  pub starting: bool,
  pub restarting: bool,
  pub pausing: bool,
  pub unpausing: bool,
  pub stopping: bool,
  pub destroying: bool,
}

#[typeshare]
pub type StackQuery = ResourceQuery<StackQuerySpecifics>;

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, DefaultBuilder,
)]
pub struct StackQuerySpecifics {
  /// Query only for Stacks on these Servers.
  /// If empty, does not filter by Server.
  /// Only accepts Server id (not name).
  #[serde(default)]
  pub server_ids: Vec<String>,
  /// Query only for Stacks with these linked repos.
  /// Only accepts Repo id (not name).
  #[serde(default)]
  pub linked_repos: Vec<String>,
  /// Filter syncs by their repo.
  #[serde(default)]
  pub repos: Vec<String>,
  /// Query only for Stack with available image updates.
  #[serde(default)]
  pub update_available: bool,
}

impl super::resource::AddFilters for StackQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if !self.server_ids.is_empty() {
      filters
        .insert("config.server_id", doc! { "$in": &self.server_ids });
    }
    if !self.linked_repos.is_empty() {
      filters.insert(
        "config.linked_repo",
        doc! { "$in": &self.linked_repos },
      );
    }
    if !self.repos.is_empty() {
      filters.insert("config.repo", doc! { "$in": &self.repos });
    }
  }
}

/// Keeping this minimal for now as its only needed to parse the service names / container names,
/// and replica count. Not a typeshared type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeFile {
  /// If not provided, will default to the parent folder holding the compose file.
  pub name: Option<String>,
  #[serde(default)]
  pub services: HashMap<String, ComposeService>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeService {
  pub image: Option<String>,
  pub container_name: Option<String>,
  pub deploy: Option<ComposeServiceDeploy>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeServiceDeploy {
  #[serde(
    default,
    deserialize_with = "option_maybe_string_i64_deserializer"
  )]
  pub replicas: Option<i64>,
}

// PRE-1.19.1 BACKWARD COMPAT NOTE
// This was split from general FileContents in 1.19.1,
// and must maintain 2 way de/ser backward compatibility
// with the mentioned struct.
/// Same as [FileContents] with some extra
/// info specific to Stacks.
#[typeshare]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackRemoteFileContents {
  /// The path to the file
  pub path: String,
  /// The contents of the file
  pub contents: String,
  /// The services depending on this file,
  /// or empty for global requirement (eg all compose files and env files).
  #[serde(default)]
  pub services: Vec<String>,
  /// Whether diff requires Redeploy / Restart / None
  #[serde(default)]
  pub requires: StackFileRequires,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Default,
  Serialize,
  Deserialize,
)]
pub enum StackFileRequires {
  /// Diff requires service redeploy.
  #[serde(alias = "redeploy")]
  Redeploy,
  /// Diff requires service restart
  #[serde(alias = "restart")]
  Restart,
  /// Diff requires no action. Default.
  #[default]
  #[serde(alias = "none")]
  None,
}

/// Additional env file configuration for Stack.
/// Supports backward compatibility with string-only format.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AdditionalEnvFile {
  /// File path relative to run directory
  pub path: String,
  /// Whether Komodo should track this file's contents.
  /// If true (default), Komodo will read, display, diff, and validate.
  /// If false, only passed to docker compose via --env-file.
  /// Useful for externally managed files (e.g., sops decrypted files).
  #[serde(default = "default_true")]
  pub track: bool,
}

fn default_true() -> bool {
  true
}

/// Used with custom de/serializer for [AdditionalEnvFile]
#[derive(Deserialize)]
struct __AdditionalEnvFile {
  path: String,
  #[serde(default = "default_true")]
  track: bool,
}

impl<'de> Deserialize<'de> for AdditionalEnvFile {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct AdditionalEnvFileVisitor;

    impl<'de> Visitor<'de> for AdditionalEnvFileVisitor {
      type Value = AdditionalEnvFile;

      fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
      ) -> std::fmt::Result {
        write!(formatter, "string or AdditionalEnvFile (object)")
      }

      fn visit_string<E>(self, path: String) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Ok(AdditionalEnvFile {
          path,
          track: default_true(),
        })
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Self::visit_string(self, v.to_string())
      }

      fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::MapAccess<'de>,
      {
        __AdditionalEnvFile::deserialize(
          MapAccessDeserializer::new(map).into_deserializer(),
        )
        .map(|v| AdditionalEnvFile {
          path: v.path,
          track: v.track,
        })
      }
    }

    deserializer.deserialize_any(AdditionalEnvFileVisitor)
  }
}

/// Configure additional file dependencies of the Stack.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct StackFileDependency {
  /// Specify the file
  pub path: String,
  /// Specify specific service/s
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub services: Vec<String>,
  /// Specify
  #[serde(default, skip_serializing_if = "is_none")]
  pub requires: StackFileRequires,
}

impl StackFileDependency {
  pub fn full_redeploy(path: String) -> StackFileDependency {
    StackFileDependency {
      path,
      services: Vec::new(),
      requires: StackFileRequires::Redeploy,
    }
  }
}

fn is_none(requires: &StackFileRequires) -> bool {
  matches!(requires, StackFileRequires::None)
}

/// Used with custom de/serializer for [StackFileDependency]
#[derive(Deserialize)]
struct __StackFileDependency {
  path: String,
  #[serde(
    default,
    alias = "service",
    deserialize_with = "string_list_deserializer"
  )]
  services: Vec<String>,
  #[serde(default, alias = "req")]
  requires: StackFileRequires,
}

impl<'de> Deserialize<'de> for StackFileDependency {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct StackFileDependencyVisitor;

    impl<'de> Visitor<'de> for StackFileDependencyVisitor {
      type Value = StackFileDependency;

      fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
      ) -> std::fmt::Result {
        write!(formatter, "string or StackFileDependency (object)")
      }

      fn visit_string<E>(self, path: String) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Ok(StackFileDependency {
          path,
          services: Vec::new(),
          requires: StackFileRequires::None,
        })
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Self::visit_string(self, v.to_string())
      }

      fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::MapAccess<'de>,
      {
        __StackFileDependency::deserialize(
          MapAccessDeserializer::new(map).into_deserializer(),
        )
        .map(|v| StackFileDependency {
          path: v.path,
          services: v.services,
          requires: v.requires,
        })
      }
    }

    deserializer.deserialize_any(StackFileDependencyVisitor)
  }
}

// // This one is nice for TOML, but annoying to use on frontend
// impl Serialize for StackFileDependency {
//   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//   where
//     S: serde::Serializer,
//   {
//     // Serialize to string in default case
//     if is_redeploy(&self.requires) && self.services.is_empty() {
//       return serializer.serialize_str(&self.path);
//     }
//     __StackFileDependency {
//       path: self.path.clone(),
//       services: self.services.clone(),
//       requires: self.requires,
//     }
//     .serialize(serializer)
//   }
// }
