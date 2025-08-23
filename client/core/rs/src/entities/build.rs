use std::sync::OnceLock;

use bson::{Document, doc};
use derive_builder::Builder;
use derive_default_builder::DefaultBuilder;
use partial_derive2::Partial;
use serde::{Deserialize, Serialize};
use strum::Display;
use typeshare::typeshare;

use crate::{
  deserializers::{
    env_vars_deserializer, item_or_vec_deserializer,
    labels_deserializer, option_env_vars_deserializer,
    option_item_or_vec_deserializer, option_labels_deserializer,
    option_string_list_deserializer, string_list_deserializer,
  },
  entities::I64,
};

use super::{
  SystemCommand, Version,
  resource::{Resource, ResourceListItem, ResourceQuery},
};

#[typeshare]
pub type Build = Resource<BuildConfig, BuildInfo>;

#[typeshare]
pub type BuildListItem = ResourceListItem<BuildListItemInfo>;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildListItemInfo {
  /// State of the build. Reflects whether most recent build successful.
  pub state: BuildState,
  /// Unix timestamp in milliseconds of last build
  pub last_built_at: I64,
  /// The current version of the build
  pub version: Version,
  /// The builder attached to build.
  pub builder_id: String,

  /// Whether build is in files on host mode.
  pub files_on_host: bool,
  /// Whether build has UI defined dockerfile contents
  pub dockerfile_contents: bool,

  /// Linked repo, if one is attached.
  pub linked_repo: String,
  /// The git provider domain
  pub git_provider: String,
  /// The repo used as the source of the build
  pub repo: String,
  /// The branch of the repo
  pub branch: String,
  /// Full link to the repo.
  pub repo_link: String,

  /// Latest built short commit hash, or null.
  pub built_hash: Option<String>,
  /// Latest short commit hash, or null. Only for repo based stacks
  pub latest_hash: Option<String>,

  /// The first listed image registry domain
  pub image_registry_domain: Option<String>,
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
pub enum BuildState {
  /// Currently building
  Building,
  /// Last build successful (or never built)
  Ok,
  /// Last build failed
  Failed,
  /// Other case
  #[default]
  Unknown,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BuildInfo {
  /// The timestamp build was last built.
  pub last_built_at: I64,

  /// Latest built short commit hash, or null.
  pub built_hash: Option<String>,
  /// Latest built commit message, or null. Only for repo based stacks
  pub built_message: Option<String>,
  /// The last built dockerfile contents.
  /// This is updated whenever Komodo successfully runs the build.
  pub built_contents: Option<String>,

  /// The absolute path to the file
  pub remote_path: Option<String>,
  /// The remote dockerfile contents, whether on host or in repo.
  /// This is updated whenever Komodo refreshes the build cache.
  /// It will be empty if the dockerfile is defined directly in the build config.
  pub remote_contents: Option<String>,
  /// If there was an error in getting the remote contents, it will be here.
  pub remote_error: Option<String>,

  /// Latest remote short commit hash, or null.
  pub latest_hash: Option<String>,
  /// Latest remote commit message, or null
  pub latest_message: Option<String>,
}

#[typeshare(serialized_as = "Partial<BuildConfig>")]
pub type _PartialBuildConfig = PartialBuildConfig;

/// The build configuration.
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Partial)]
#[partial_derive(Debug, Clone, Default, Serialize, Deserialize)]
#[partial(skip_serializing_none, from, diff)]
pub struct BuildConfig {
  /// Which builder is used to build the image.
  #[serde(default, alias = "builder")]
  #[partial_attr(serde(alias = "builder"))]
  #[builder(default)]
  pub builder_id: String,

  /// The current version of the build.
  #[serde(default)]
  #[builder(default)]
  pub version: Version,

  /// Whether to automatically increment the patch on every build.
  /// Default is `true`
  #[serde(default = "default_auto_increment_version")]
  #[builder(default = "default_auto_increment_version()")]
  #[partial_default(default_auto_increment_version())]
  pub auto_increment_version: bool,

  /// An alternate name for the image pushed to the repository.
  /// If this is empty, it will use the build name.
  ///
  /// Can be used in conjunction with `image_tag` to direct multiple builds
  /// with different configs to push to the same image registry, under different,
  /// independantly versioned tags.
  #[serde(default)]
  #[builder(default)]
  pub image_name: String,

  /// An extra tag put after the build version, for the image pushed to the repository.
  /// Eg. in image tag of `aarch64` would push to moghtech/komodo-core:1.13.2-aarch64.
  /// If this is empty, the image tag will just be the build version.
  ///
  /// Can be used in conjunction with `image_name` to direct multiple builds
  /// with different configs to push to the same image registry, under different,
  /// independantly versioned tags.
  #[serde(default)]
  #[builder(default)]
  pub image_tag: String,

  /// Configure quick links that are displayed in the resource header
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub links: Vec<String>,

  /// Choose a Komodo Repo (Resource) to source the build files.
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

  /// Whether incoming webhooks actually trigger action.
  #[serde(default = "default_webhook_enabled")]
  #[builder(default = "default_webhook_enabled()")]
  #[partial_default(default_webhook_enabled())]
  pub webhook_enabled: bool,

  /// Optionally provide an alternate webhook secret for this build.
  /// If its an empty string, use the default secret from the config.
  #[serde(default)]
  #[builder(default)]
  pub webhook_secret: String,

  /// If this is checked, the build will source the files on the host.
  /// Use `build_path` and `dockerfile_path` to specify the path on the host.
  /// This is useful for those who wish to setup their files on the host,
  /// rather than defining the contents in UI or in a git repo.
  #[serde(default)]
  #[builder(default)]
  pub files_on_host: bool,

  /// The path of the docker build context relative to the root of the repo.
  /// Default: "." (the root of the repo).
  #[serde(default = "default_build_path")]
  #[builder(default = "default_build_path()")]
  #[partial_default(default_build_path())]
  pub build_path: String,

  /// The path of the dockerfile relative to the build path.
  #[serde(default = "default_dockerfile_path")]
  #[builder(default = "default_dockerfile_path()")]
  #[partial_default(default_dockerfile_path())]
  pub dockerfile_path: String,

  /// Configuration for the registry/s to push the built image to.
  /// The first registry in this list will be used with attached Deployments.
  #[serde(default, deserialize_with = "item_or_vec_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_item_or_vec_deserializer"
  ))]
  #[builder(default)]
  pub image_registry: Vec<ImageRegistryConfig>,

  /// Whether to skip secret interpolation in the build_args.
  #[serde(default)]
  #[builder(default)]
  pub skip_secret_interp: bool,

  /// Whether to use buildx to build (eg `docker buildx build ...`)
  #[serde(default)]
  #[builder(default)]
  pub use_buildx: bool,

  /// Any extra docker cli arguments to be included in the build command
  #[serde(default, deserialize_with = "string_list_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_string_list_deserializer"
  ))]
  #[builder(default)]
  pub extra_args: Vec<String>,

  /// The optional command run after repo clone and before docker build.
  #[serde(default)]
  #[builder(default)]
  pub pre_build: SystemCommand,

  /// UI defined dockerfile contents.
  /// Supports variable / secret interpolation.
  #[serde(default)]
  #[builder(default)]
  pub dockerfile: String,

  /// Docker build arguments.
  ///
  /// These values are visible in the final image by running `docker inspect`.
  #[serde(default, deserialize_with = "env_vars_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_env_vars_deserializer"
  ))]
  #[builder(default)]
  pub build_args: String,

  /// Secret arguments.
  ///
  /// These values remain hidden in the final image by using
  /// docker secret mounts. See <https://docs.docker.com/build/building/secrets>.
  ///
  /// The values can be used in RUN commands:
  /// ```sh
  /// RUN --mount=type=secret,id=SECRET_KEY \
  ///   SECRET_KEY=$(cat /run/secrets/SECRET_KEY) ...
  /// ```
  #[serde(default, deserialize_with = "env_vars_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_env_vars_deserializer"
  ))]
  #[builder(default)]
  pub secret_args: String,

  /// Docker labels
  #[serde(default, deserialize_with = "labels_deserializer")]
  #[partial_attr(serde(
    default,
    deserialize_with = "option_labels_deserializer"
  ))]
  #[builder(default)]
  pub labels: String,
}

impl BuildConfig {
  pub fn builder() -> BuildConfigBuilder {
    BuildConfigBuilder::default()
  }
}

fn default_auto_increment_version() -> bool {
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

fn default_build_path() -> String {
  String::from(".")
}

fn default_dockerfile_path() -> String {
  String::from("Dockerfile")
}

fn default_webhook_enabled() -> bool {
  true
}

impl Default for BuildConfig {
  fn default() -> Self {
    Self {
      builder_id: Default::default(),
      skip_secret_interp: Default::default(),
      version: Default::default(),
      auto_increment_version: default_auto_increment_version(),
      image_name: Default::default(),
      image_tag: Default::default(),
      links: Default::default(),
      linked_repo: Default::default(),
      git_provider: default_git_provider(),
      git_https: default_git_https(),
      repo: Default::default(),
      branch: default_branch(),
      commit: Default::default(),
      git_account: Default::default(),
      pre_build: Default::default(),
      build_path: default_build_path(),
      dockerfile_path: default_dockerfile_path(),
      build_args: Default::default(),
      secret_args: Default::default(),
      labels: Default::default(),
      extra_args: Default::default(),
      use_buildx: Default::default(),
      image_registry: Default::default(),
      webhook_enabled: default_webhook_enabled(),
      webhook_secret: Default::default(),
      dockerfile: Default::default(),
      files_on_host: Default::default(),
    }
  }
}

/// Configuration for an image registry
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize,
)]
pub struct ImageRegistryConfig {
  /// Specify the registry provider domain, eg `docker.io`.
  /// If not provided, will not push to any registry.
  #[serde(default)]
  pub domain: String,

  /// Specify an account to use with the registry.
  #[serde(default)]
  pub account: String,

  /// Optional. Specify an organization to push the image under.
  /// Empty string means no organization.
  #[serde(default)]
  pub organization: String,
}

impl ImageRegistryConfig {
  pub fn static_default() -> &'static ImageRegistryConfig {
    static DEFAULT: OnceLock<ImageRegistryConfig> = OnceLock::new();
    DEFAULT.get_or_init(Default::default)
  }
}

#[typeshare]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BuildActionState {
  pub building: bool,
}

#[typeshare]
pub type BuildQuery = ResourceQuery<BuildQuerySpecifics>;

#[typeshare]
#[derive(
  Debug, Clone, Default, Serialize, Deserialize, DefaultBuilder,
)]
pub struct BuildQuerySpecifics {
  #[serde(default)]
  pub builder_ids: Vec<String>,

  #[serde(default)]
  pub repos: Vec<String>,

  /// query for builds last built more recently than this timestamp
  /// defaults to 0 which is a no op
  #[serde(default)]
  pub built_since: I64,
}

impl super::resource::AddFilters for BuildQuerySpecifics {
  fn add_filters(&self, filters: &mut Document) {
    if !self.builder_ids.is_empty() {
      filters.insert(
        "config.builder_id",
        doc! { "$in": &self.builder_ids },
      );
    }
    if !self.repos.is_empty() {
      filters.insert("config.repo", doc! { "$in": &self.repos });
    }
    if self.built_since > 0 {
      filters.insert(
        "info.last_built_at",
        doc! { "$gte": self.built_since },
      );
    }
  }
}
