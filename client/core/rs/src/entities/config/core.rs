//! # Configuring the Komodo Core API
//!
//! Komodo Core is configured by parsing base configuration file ([CoreConfig]), and overriding
//! any fields given in the file with ones provided on the environment ([Env]).
//!
//! The recommended method for running Komodo Core is via the docker image. This image has a default
//! configuration file provided in the image, meaning any custom configuration can be provided
//! on the environment alone. However, if a custom configuration file is prefered, it can be mounted
//! into the image at `/config/config.toml`.
//!

use std::{collections::HashMap, path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::entities::{
  Timelength,
  config::DatabaseConfig,
  logger::{LogConfig, LogLevel, StdioLogMode},
};

use super::{DockerRegistry, GitProvider, empty_or_redacted};

/// # Komodo Core Environment Variables
///
/// You can override any fields of the [CoreConfig] by passing the associated
/// environment variable. The variables should be passed in the traditional `UPPER_SNAKE_CASE` format,
/// although the lower case format can still be parsed.
///
/// *Note.* The Komodo Core docker image includes the default core configuration found at
/// [https://github.com/moghtech/komodo/blob/main/config/core.config.toml](https://github.com/moghtech/komodo/blob/main/config/core.config.toml).
/// To configure the core api, you can either mount your own custom configuration file to
/// `/config/config.toml` inside the container,
/// or simply override whichever fields you need using the environment.
#[derive(Debug, Clone, Deserialize)]
pub struct Env {
  /// Specify a custom config path for the core config toml.
  /// Default: `/config/config.toml`
  #[serde(
    default = "default_core_config_paths",
    alias = "komodo_config_path"
  )]
  pub komodo_config_paths: Vec<PathBuf>,
  /// If specifying folders, use this to narrow down which
  /// files will be matched to parse into the final [PeripheryConfig].
  /// Only files inside the folders which have names containing a keywords
  /// provided to `config_keywords` will be included.
  /// Keywords support wildcard matching syntax.
  #[serde(
    default = "super::default_config_keywords",
    alias = "komodo_config_keyword"
  )]
  pub komodo_config_keywords: Vec<String>,
  /// Will merge nested config object (eg. secrets, providers) across multiple
  /// config files. Default: `true`
  #[serde(default = "super::default_merge_nested_config")]
  pub komodo_merge_nested_config: bool,
  /// Will extend config arrays across multiple config files.
  /// Default: `true`
  #[serde(default = "super::default_extend_config_arrays")]
  pub komodo_extend_config_arrays: bool,
  /// Print some extra logs on startup to debug config loading issues.
  #[serde(default)]
  pub komodo_config_debug: bool,

  /// Override `title`
  pub komodo_title: Option<String>,
  /// Override `host`
  pub komodo_host: Option<String>,
  /// Override `port`
  pub komodo_port: Option<u16>,
  /// Override `bind_ip`
  pub komodo_bind_ip: Option<String>,
  /// Override `passkey`
  pub komodo_passkey: Option<String>,
  /// Override `passkey` with file
  pub komodo_passkey_file: Option<PathBuf>,
  /// Override `timezone`
  #[serde(alias = "tz", alias = "TZ")]
  pub komodo_timezone: Option<String>,
  /// Override `first_server`
  pub komodo_first_server: Option<String>,
  /// Override `first_server_name`
  pub komodo_first_server_name: Option<String>,
  /// Override `frontend_path`
  pub komodo_frontend_path: Option<String>,
  /// Override `jwt_secret`
  pub komodo_jwt_secret: Option<String>,
  /// Override `jwt_secret` from file
  pub komodo_jwt_secret_file: Option<PathBuf>,
  /// Override `jwt_ttl`
  pub komodo_jwt_ttl: Option<Timelength>,
  /// Override `sync_directory`
  pub komodo_sync_directory: Option<PathBuf>,
  /// Override `repo_directory`
  pub komodo_repo_directory: Option<PathBuf>,
  /// Override `action_directory`
  pub komodo_action_directory: Option<PathBuf>,
  /// Override `resource_poll_interval`
  pub komodo_resource_poll_interval: Option<Timelength>,
  /// Override `monitoring_interval`
  pub komodo_monitoring_interval: Option<Timelength>,
  /// Override `keep_stats_for_days`
  pub komodo_keep_stats_for_days: Option<u64>,
  /// Override `keep_alerts_for_days`
  pub komodo_keep_alerts_for_days: Option<u64>,
  /// Override `webhook_secret`
  pub komodo_webhook_secret: Option<String>,
  /// Override `webhook_secret` with file
  pub komodo_webhook_secret_file: Option<PathBuf>,
  /// Override `webhook_base_url`
  pub komodo_webhook_base_url: Option<String>,

  /// Override `logging.level`
  pub komodo_logging_level: Option<LogLevel>,
  /// Override `logging.stdio`
  pub komodo_logging_stdio: Option<StdioLogMode>,
  /// Override `logging.pretty`
  pub komodo_logging_pretty: Option<bool>,
  /// Override `logging.location`
  pub komodo_logging_location: Option<bool>,
  /// Override `logging.otlp_endpoint`
  pub komodo_logging_otlp_endpoint: Option<String>,
  /// Override `logging.opentelemetry_service_name`
  pub komodo_logging_opentelemetry_service_name: Option<String>,
  /// Override `pretty_startup_config`
  pub komodo_pretty_startup_config: Option<bool>,
  /// Override `unsafe_unsanitized_startup_config`
  pub komodo_unsafe_unsanitized_startup_config: Option<bool>,

  /// Override `transparent_mode`
  pub komodo_transparent_mode: Option<bool>,
  /// Override `ui_write_disabled`
  pub komodo_ui_write_disabled: Option<bool>,
  /// Override `enable_new_users`
  pub komodo_enable_new_users: Option<bool>,
  /// Override `disable_user_registration`
  pub komodo_disable_user_registration: Option<bool>,
  /// Override `lock_login_credentials_for`
  pub komodo_lock_login_credentials_for: Option<Vec<String>>,
  /// Override `disable_confirm_dialog`
  pub komodo_disable_confirm_dialog: Option<bool>,
  /// Override `disable_non_admin_create`
  pub komodo_disable_non_admin_create: Option<bool>,
  /// Override `disable_websocket_reconnect`
  pub komodo_disable_websocket_reconnect: Option<bool>,
  /// Override `disable_init_resources`
  pub komodo_disable_init_resources: Option<bool>,
  /// Override `enable_fancy_toml`
  pub komodo_enable_fancy_toml: Option<bool>,

  /// Override `local_auth`
  pub komodo_local_auth: Option<bool>,
  /// Override `init_admin_username`
  pub komodo_init_admin_username: Option<String>,
  /// Override `init_admin_username` from file
  pub komodo_init_admin_username_file: Option<PathBuf>,
  /// Override `init_admin_password`
  pub komodo_init_admin_password: Option<String>,
  /// Override `init_admin_password` from file
  pub komodo_init_admin_password_file: Option<PathBuf>,

  /// Override `oidc_enabled`
  pub komodo_oidc_enabled: Option<bool>,
  /// Override `oidc_provider`
  pub komodo_oidc_provider: Option<String>,
  /// Override `oidc_redirect_host`
  pub komodo_oidc_redirect_host: Option<String>,
  /// Override `oidc_client_id`
  pub komodo_oidc_client_id: Option<String>,
  /// Override `oidc_client_id` from file
  pub komodo_oidc_client_id_file: Option<PathBuf>,
  /// Override `oidc_client_secret`
  pub komodo_oidc_client_secret: Option<String>,
  /// Override `oidc_client_secret` from file
  pub komodo_oidc_client_secret_file: Option<PathBuf>,
  /// Override `oidc_use_full_email`
  pub komodo_oidc_use_full_email: Option<bool>,
  /// Override `oidc_additional_audiences`
  pub komodo_oidc_additional_audiences: Option<Vec<String>>,
  /// Override `oidc_additional_audiences` from file
  pub komodo_oidc_additional_audiences_file: Option<PathBuf>,

  /// Override `google_oauth.enabled`
  pub komodo_google_oauth_enabled: Option<bool>,
  /// Override `google_oauth.id`
  pub komodo_google_oauth_id: Option<String>,
  /// Override `google_oauth.id` from file
  pub komodo_google_oauth_id_file: Option<PathBuf>,
  /// Override `google_oauth.secret`
  pub komodo_google_oauth_secret: Option<String>,
  /// Override `google_oauth.secret` from file
  pub komodo_google_oauth_secret_file: Option<PathBuf>,

  /// Override `github_oauth.enabled`
  pub komodo_github_oauth_enabled: Option<bool>,
  /// Override `github_oauth.id`
  pub komodo_github_oauth_id: Option<String>,
  /// Override `github_oauth.id` from file
  pub komodo_github_oauth_id_file: Option<PathBuf>,
  /// Override `github_oauth.secret`
  pub komodo_github_oauth_secret: Option<String>,
  /// Override `github_oauth.secret` from file
  pub komodo_github_oauth_secret_file: Option<PathBuf>,

  /// Override `github_webhook_app.app_id`
  pub komodo_github_webhook_app_app_id: Option<i64>,
  /// Override `github_webhook_app.app_id` from file
  pub komodo_github_webhook_app_app_id_file: Option<PathBuf>,
  /// Override `github_webhook_app.installations[i].id`. Accepts comma seperated list.
  ///
  /// Note. Paired by index with values in `komodo_github_webhook_app_installations_namespaces`
  pub komodo_github_webhook_app_installations_ids: Option<Vec<i64>>,
  /// Override `github_webhook_app.installations[i].id` from file
  pub komodo_github_webhook_app_installations_ids_file:
    Option<PathBuf>,
  /// Override `github_webhook_app.installations[i].namespace`. Accepts comma seperated list.
  ///
  /// Note. Paired by index with values in `komodo_github_webhook_app_installations_ids`
  pub komodo_github_webhook_app_installations_namespaces:
    Option<Vec<String>>,
  /// Override `github_webhook_app.pk_path`
  pub komodo_github_webhook_app_pk_path: Option<String>,

  /// Override `database.uri`
  #[serde(alias = "komodo_mongo_uri")]
  pub komodo_database_uri: Option<String>,
  /// Override `database.uri` from file
  #[serde(alias = "komodo_mongo_uri_file")]
  pub komodo_database_uri_file: Option<PathBuf>,
  /// Override `database.address`
  #[serde(alias = "komodo_mongo_address")]
  pub komodo_database_address: Option<String>,
  /// Override `database.username`
  #[serde(alias = "komodo_mongo_username")]
  pub komodo_database_username: Option<String>,
  /// Override `database.username` with file
  #[serde(alias = "komodo_mongo_username_file")]
  pub komodo_database_username_file: Option<PathBuf>,
  /// Override `database.password`
  #[serde(alias = "komodo_mongo_password")]
  pub komodo_database_password: Option<String>,
  /// Override `database.password` with file
  #[serde(alias = "komodo_mongo_password_file")]
  pub komodo_database_password_file: Option<PathBuf>,
  /// Override `database.app_name`
  #[serde(alias = "komodo_mongo_app_name")]
  pub komodo_database_app_name: Option<String>,
  /// Override `database.db_name`
  #[serde(alias = "komodo_mongo_db_name")]
  pub komodo_database_db_name: Option<String>,

  /// Override `aws.access_key_id`
  pub komodo_aws_access_key_id: Option<String>,
  /// Override `aws.access_key_id` with file
  pub komodo_aws_access_key_id_file: Option<PathBuf>,
  /// Override `aws.secret_access_key`
  pub komodo_aws_secret_access_key: Option<String>,
  /// Override `aws.secret_access_key` with file
  pub komodo_aws_secret_access_key_file: Option<PathBuf>,

  /// Override `internet_interface`
  pub komodo_internet_interface: Option<String>,

  /// Override `ssl_enabled`.
  pub komodo_ssl_enabled: Option<bool>,
  /// Override `ssl_key_file`
  pub komodo_ssl_key_file: Option<PathBuf>,
  /// Override `ssl_cert_file`
  pub komodo_ssl_cert_file: Option<PathBuf>,
}

fn default_core_config_paths() -> Vec<PathBuf> {
  vec![PathBuf::from_str("/config").unwrap()]
}

/// # Core Configuration File
///
/// The Core API initializes it's configuration by reading the environment,
/// parsing the [CoreConfig] schema from the file path specified by `env.komodo_config_path`,
/// and then applying any config field overrides specified in the environment.
///
/// *Note.* The Komodo Core docker image includes the default core configuration found at
/// [https://github.com/moghtech/komodo/blob/main/config/core.config.toml](https://github.com/moghtech/komodo/blob/main/config/core.config.toml).
/// To configure the core api, you can either mount your own custom configuration file to
/// `/config/config.toml` inside the container,
/// or simply override whichever fields you need using the environment.
///
/// Refer to the [example file](https://github.com/moghtech/komodo/blob/main/config/core.config.toml) for a full example.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfig {
  // ===========
  // = General =
  // ===========
  /// The title of this Komodo Core deployment. Will be used in the browser page title.
  /// Default: 'Komodo'
  #[serde(default = "default_title")]
  pub title: String,

  /// The host to use with oauth redirect url, whatever host
  /// the user hits to access Komodo. eg `https://komodo.domain.com`.
  /// Only used if oauth used without user specifying redirect url themselves.
  #[serde(default = "default_host")]
  pub host: String,

  /// Port the core web server runs on.
  /// Default: 9120.
  #[serde(default = "default_core_port")]
  pub port: u16,

  /// IP address the core server binds to.
  /// Default: [::].
  #[serde(default = "default_core_bind_ip")]
  pub bind_ip: String,

  /// Interface to use as default route in multi-NIC environments.
  #[serde(default)]
  pub internet_interface: String,

  /// Sent in auth header with req to periphery.
  /// Should be some secure hash, maybe 20-40 chars.
  #[serde(default = "default_passkey")]
  pub passkey: String,

  /// A TZ Identifier. If not provided, will use Core local timezone.
  /// https://en.wikipedia.org/wiki/List_of_tz_database_time_zones.
  /// This will be populated by TZ env variable in addition to KOMODO_TIMEZONE.
  #[serde(default)]
  pub timezone: String,

  /// Disable user ability to use the UI to update resource configuration.
  #[serde(default)]
  pub ui_write_disabled: bool,

  /// Disable the popup confirm dialogs. All buttons will just be double click.
  #[serde(default)]
  pub disable_confirm_dialog: bool,

  /// Disable the UI websocket from automatically reconnecting.
  #[serde(default)]
  pub disable_websocket_reconnect: bool,

  /// Disable init system resource creation on fresh Komodo launch.
  /// These include the Backup Core Database and Global Auto Update procedures.
  #[serde(default)]
  pub disable_init_resources: bool,

  /// Enable the fancy TOML syntax highlighting
  #[serde(default)]
  pub enable_fancy_toml: bool,

  /// If defined, ensure an enabled first server exists at this address.
  /// Example: `http://periphery:8120`
  #[serde(skip_serializing_if = "Option::is_none")]
  pub first_server: Option<String>,

  /// Give the first server this name.
  /// Default: `Local`
  #[serde(default = "default_first_server_name")]
  pub first_server_name: String,

  /// The path to the built frontend folder.
  #[serde(default = "default_frontend_path")]
  pub frontend_path: String,

  /// Configure database connection
  #[serde(default, alias = "mongo")]
  pub database: DatabaseConfig,

  // ================
  // = Auth / Login =
  // ================
  /// enable login with local auth
  #[serde(default)]
  pub local_auth: bool,

  /// Upon fresh launch, initalize an Admin user with this username.
  /// If this is not provided, no initial user will be created.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub init_admin_username: Option<String>,
  /// Upon fresh launch, initalize an Admin user with this password.
  /// Default: `changeme`
  #[serde(default = "default_init_admin_password")]
  pub init_admin_password: String,

  /// Enable transparent mode, which gives all (enabled) users read access to all resources.
  #[serde(default)]
  pub transparent_mode: bool,

  /// New users will be automatically enabled.
  /// Combined with transparent mode, this is suitable for a demo instance.
  #[serde(default)]
  pub enable_new_users: bool,

  /// Normally new users will be registered, but not enabled until an Admin enables them.
  /// With `disable_user_registration = true`, only the first user to log in will registered as a user.
  #[serde(default)]
  pub disable_user_registration: bool,

  /// List of usernames for which the update username / password
  /// APIs are disabled. Used by demo to lock the 'demo' : 'demo' login.
  ///
  /// To lock the api for all users, use `lock_login_credentials_for = ["__ALL__"]`
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub lock_login_credentials_for: Vec<String>,

  /// Normally all users can create resources.
  /// If `disable_non_admin_create = true`, only admins will be able to create resources.
  #[serde(default)]
  pub disable_non_admin_create: bool,

  /// Optionally provide a specific jwt secret.
  /// Passing nothing or an empty string will cause one to be generated.
  /// Default: "" (empty string)
  #[serde(default)]
  pub jwt_secret: String,

  /// Control how long distributed JWT remain valid for.
  /// Default: `1-day`.
  #[serde(default = "default_jwt_ttl")]
  pub jwt_ttl: Timelength,

  // ========
  // = OIDC =
  // ========
  /// Enable login with configured OIDC provider.
  #[serde(default)]
  pub oidc_enabled: bool,

  /// Configure OIDC provider address for
  /// communcation directly with Komodo Core.
  ///
  /// Note. Needs to be reachable from Komodo Core.
  ///
  /// `https://accounts.example.internal/application/o/komodo`
  #[serde(default)]
  pub oidc_provider: String,

  /// Configure OIDC user redirect host.
  ///
  /// This is the host address users are redirected to in their browser,
  /// and may be different from `oidc_provider` host.
  /// DO NOT include the `path` part, this must be inferred.
  /// If not provided, the host will be the same as `oidc_provider`.
  /// Eg. `https://accounts.example.external`
  #[serde(default)]
  pub oidc_redirect_host: String,

  /// Set OIDC client id
  #[serde(default)]
  pub oidc_client_id: String,

  /// Set OIDC client secret
  #[serde(default)]
  pub oidc_client_secret: String,

  /// Use the full email for usernames.
  /// Otherwise, the @address will be stripped,
  /// making usernames more concise.
  #[serde(default)]
  pub oidc_use_full_email: bool,

  /// Your OIDC provider may set additional audiences other than `client_id`,
  /// they must be added here to make claims verification work.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub oidc_additional_audiences: Vec<String>,

  // =========
  // = Oauth =
  // =========
  /// Configure google oauth
  #[serde(default)]
  pub google_oauth: OauthCredentials,

  /// Configure github oauth
  #[serde(default)]
  pub github_oauth: OauthCredentials,

  // ============
  // = Webhooks =
  // ============
  /// Used to verify validity from webhooks.
  /// Should be some secure hash maybe 20-40 chars.
  /// It is given to git provider when configuring the webhook.
  #[serde(default)]
  pub webhook_secret: String,

  /// Override the webhook listener base url, if None will use the address defined as 'host'.
  /// Example: `https://webhooks.komo.do`
  ///
  /// This can be used if Komodo Core sits on an internal network which is
  /// unreachable directly from the open internet.
  /// A reverse proxy in a public network can forward webhooks to Komodo.
  #[serde(default)]
  pub webhook_base_url: String,

  /// Configure a Github Webhook app.
  /// Allows users to manage repo webhooks from within the Komodo UI.
  #[serde(default)]
  pub github_webhook_app: GithubWebhookAppConfig,

  // ===========
  // = Logging =
  // ===========
  /// Configure logging
  #[serde(default)]
  pub logging: LogConfig,

  /// Pretty-log (multi-line) the startup config
  /// for easier human readability.
  #[serde(default)]
  pub pretty_startup_config: bool,

  /// Unsafe: logs unsanitized config on startup,
  /// in order to verify everything is being
  /// passed correctly.
  pub unsafe_unsanitized_startup_config: bool,

  // ===========
  // = Pruning =
  // ===========
  /// Number of days to keep stats, or 0 to disable pruning.
  /// Stats older than this number of days are deleted on a daily cycle
  /// Default: 14
  #[serde(default = "default_prune_days")]
  pub keep_stats_for_days: u64,

  /// Number of days to keep alerts, or 0 to disable pruning.
  /// Alerts older than this number of days are deleted on a daily cycle
  /// Default: 14
  #[serde(default = "default_prune_days")]
  pub keep_alerts_for_days: u64,

  // ==================
  // = Poll Intervals =
  // ==================
  /// Interval at which to poll resources for any updates / automated actions.
  /// Options: `15-sec`, `1-min`, `5-min`, `15-min`, `1-hr`
  /// Default: `5-min`.  
  #[serde(default = "default_poll_interval")]
  pub resource_poll_interval: Timelength,

  /// Interval at which to collect server stats and send any alerts.
  /// Default: `15-sec`
  #[serde(default = "default_monitoring_interval")]
  pub monitoring_interval: Timelength,

  // ===================
  // = Cloud Providers =
  // ===================
  /// Configure AWS credentials to use with AWS builds / server launches.
  #[serde(default)]
  pub aws: AwsCredentials,

  // =================
  // = Git Providers =
  // =================
  /// Configure git credentials used to clone private repos.
  /// Supports any git provider.
  #[serde(
    default,
    alias = "git_provider",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub git_providers: Vec<GitProvider>,

  // ======================
  // = Registry Providers =
  // ======================
  /// Configure docker credentials used to push / pull images.
  /// Supports any docker image repository.
  #[serde(
    default,
    alias = "docker_registry",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub docker_registries: Vec<DockerRegistry>,

  // ===========
  // = Secrets =
  // ===========
  /// Configure core-based secrets. These will be preferentially interpolated into
  /// values if they contain a matching secret. Otherwise, the periphery will have to have the
  /// secret configured.
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub secrets: HashMap<String, String>,

  // =======
  // = SSL =
  // =======
  /// Whether to enable ssl.
  #[serde(default)]
  pub ssl_enabled: bool,

  /// Path to the ssl key.
  /// Default: `/config/ssl/key.pem`.
  #[serde(default = "default_ssl_key_file")]
  pub ssl_key_file: PathBuf,

  /// Path to the ssl cert.
  /// Default: `/config/ssl/cert.pem`.
  #[serde(default = "default_ssl_cert_file")]
  pub ssl_cert_file: PathBuf,

  // =========
  // = Other =
  // =========
  /// Configure directory to store sync files.
  /// Default: `/syncs`
  #[serde(default = "default_sync_directory")]
  pub sync_directory: PathBuf,

  /// Specify the directory used to clone stack / repo / build repos, for latest hash / contents.
  /// The default is fine when using a container.
  /// Default: `/repo-cache`
  #[serde(default = "default_repo_directory")]
  pub repo_directory: PathBuf,

  /// Specify the directory used to temporarily write typescript files used with actions.
  /// Default: `/action-cache`
  #[serde(default = "default_action_directory")]
  pub action_directory: PathBuf,
}

fn default_title() -> String {
  String::from("Komodo")
}

fn default_host() -> String {
  String::from("https://komodo.example.com")
}

fn default_core_port() -> u16 {
  9120
}

fn default_core_bind_ip() -> String {
  "[::]".to_string()
}

fn default_passkey() -> String {
  String::from("default-passkey-changeme")
}

fn default_frontend_path() -> String {
  "/app/frontend".to_string()
}

fn default_first_server_name() -> String {
  String::from("Local")
}

fn default_jwt_ttl() -> Timelength {
  Timelength::OneDay
}

fn default_init_admin_password() -> String {
  String::from("changeme")
}

fn default_sync_directory() -> PathBuf {
  // unwrap ok: `/syncs` will always be valid path
  PathBuf::from_str("/syncs").unwrap()
}

fn default_repo_directory() -> PathBuf {
  // unwrap ok: `/repo-cache` will always be valid path
  PathBuf::from_str("/repo-cache").unwrap()
}

fn default_action_directory() -> PathBuf {
  // unwrap ok: `/action-cache` will always be valid path
  PathBuf::from_str("/action-cache").unwrap()
}

fn default_prune_days() -> u64 {
  14
}

fn default_poll_interval() -> Timelength {
  Timelength::OneHour
}

fn default_monitoring_interval() -> Timelength {
  Timelength::FifteenSeconds
}

fn default_ssl_key_file() -> PathBuf {
  "/config/ssl/key.pem".parse().unwrap()
}

fn default_ssl_cert_file() -> PathBuf {
  "/config/ssl/cert.pem".parse().unwrap()
}

impl Default for CoreConfig {
  fn default() -> Self {
    Self {
      title: default_title(),
      host: default_host(),
      port: default_core_port(),
      bind_ip: default_core_bind_ip(),
      internet_interface: Default::default(),
      passkey: default_passkey(),
      timezone: Default::default(),
      ui_write_disabled: Default::default(),
      disable_confirm_dialog: Default::default(),
      disable_websocket_reconnect: Default::default(),
      disable_init_resources: Default::default(),
      enable_fancy_toml: Default::default(),
      first_server: Default::default(),
      first_server_name: default_first_server_name(),
      frontend_path: default_frontend_path(),
      database: Default::default(),
      local_auth: Default::default(),
      init_admin_username: Default::default(),
      init_admin_password: default_init_admin_password(),
      transparent_mode: Default::default(),
      enable_new_users: Default::default(),
      disable_user_registration: Default::default(),
      lock_login_credentials_for: Default::default(),
      disable_non_admin_create: Default::default(),
      jwt_secret: Default::default(),
      jwt_ttl: default_jwt_ttl(),
      oidc_enabled: Default::default(),
      oidc_provider: Default::default(),
      oidc_redirect_host: Default::default(),
      oidc_client_id: Default::default(),
      oidc_client_secret: Default::default(),
      oidc_use_full_email: Default::default(),
      oidc_additional_audiences: Default::default(),
      google_oauth: Default::default(),
      github_oauth: Default::default(),
      webhook_secret: Default::default(),
      webhook_base_url: Default::default(),
      github_webhook_app: Default::default(),
      logging: Default::default(),
      pretty_startup_config: Default::default(),
      unsafe_unsanitized_startup_config: Default::default(),
      keep_stats_for_days: default_prune_days(),
      keep_alerts_for_days: default_prune_days(),
      resource_poll_interval: default_poll_interval(),
      monitoring_interval: default_monitoring_interval(),
      aws: Default::default(),
      git_providers: Default::default(),
      docker_registries: Default::default(),
      secrets: Default::default(),
      ssl_enabled: Default::default(),
      ssl_key_file: default_ssl_key_file(),
      ssl_cert_file: default_ssl_cert_file(),
      sync_directory: default_sync_directory(),
      repo_directory: default_repo_directory(),
      action_directory: default_action_directory(),
    }
  }
}

impl CoreConfig {
  pub fn sanitized(&self) -> CoreConfig {
    let config = self.clone();
    CoreConfig {
      title: config.title,
      host: config.host,
      port: config.port,
      bind_ip: config.bind_ip,
      passkey: empty_or_redacted(&config.passkey),
      timezone: config.timezone,
      first_server: config.first_server,
      first_server_name: config.first_server_name,
      frontend_path: config.frontend_path,
      jwt_secret: empty_or_redacted(&config.jwt_secret),
      jwt_ttl: config.jwt_ttl,
      repo_directory: config.repo_directory,
      action_directory: config.action_directory,
      sync_directory: config.sync_directory,
      internet_interface: config.internet_interface,
      resource_poll_interval: config.resource_poll_interval,
      monitoring_interval: config.monitoring_interval,
      keep_stats_for_days: config.keep_stats_for_days,
      keep_alerts_for_days: config.keep_alerts_for_days,
      logging: config.logging,
      pretty_startup_config: config.pretty_startup_config,
      unsafe_unsanitized_startup_config: config
        .unsafe_unsanitized_startup_config,
      transparent_mode: config.transparent_mode,
      ui_write_disabled: config.ui_write_disabled,
      disable_confirm_dialog: config.disable_confirm_dialog,
      disable_websocket_reconnect: config.disable_websocket_reconnect,
      disable_init_resources: config.disable_init_resources,
      enable_fancy_toml: config.enable_fancy_toml,
      enable_new_users: config.enable_new_users,
      disable_user_registration: config.disable_user_registration,
      disable_non_admin_create: config.disable_non_admin_create,
      lock_login_credentials_for: config.lock_login_credentials_for,
      local_auth: config.local_auth,
      init_admin_username: config
        .init_admin_username
        .map(|u| empty_or_redacted(&u)),
      init_admin_password: empty_or_redacted(
        &config.init_admin_password,
      ),
      oidc_enabled: config.oidc_enabled,
      oidc_provider: config.oidc_provider,
      oidc_redirect_host: config.oidc_redirect_host,
      oidc_client_id: empty_or_redacted(&config.oidc_client_id),
      oidc_client_secret: empty_or_redacted(
        &config.oidc_client_secret,
      ),
      oidc_use_full_email: config.oidc_use_full_email,
      oidc_additional_audiences: config
        .oidc_additional_audiences
        .iter()
        .map(|aud| empty_or_redacted(aud))
        .collect(),
      google_oauth: OauthCredentials {
        enabled: config.google_oauth.enabled,
        id: empty_or_redacted(&config.google_oauth.id),
        secret: empty_or_redacted(&config.google_oauth.id),
      },
      github_oauth: OauthCredentials {
        enabled: config.github_oauth.enabled,
        id: empty_or_redacted(&config.github_oauth.id),
        secret: empty_or_redacted(&config.github_oauth.id),
      },
      webhook_secret: empty_or_redacted(&config.webhook_secret),
      webhook_base_url: config.webhook_base_url,
      github_webhook_app: config.github_webhook_app,
      database: config.database.sanitized(),
      aws: AwsCredentials {
        access_key_id: empty_or_redacted(&config.aws.access_key_id),
        secret_access_key: empty_or_redacted(
          &config.aws.secret_access_key,
        ),
      },
      secrets: config
        .secrets
        .into_iter()
        .map(|(id, secret)| (id, empty_or_redacted(&secret)))
        .collect(),
      git_providers: config
        .git_providers
        .into_iter()
        .map(|mut provider| {
          provider.accounts.iter_mut().for_each(|account| {
            account.token = empty_or_redacted(&account.token);
          });
          provider
        })
        .collect(),
      docker_registries: config
        .docker_registries
        .into_iter()
        .map(|mut provider| {
          provider.accounts.iter_mut().for_each(|account| {
            account.token = empty_or_redacted(&account.token);
          });
          provider
        })
        .collect(),

      ssl_enabled: config.ssl_enabled,
      ssl_key_file: config.ssl_key_file,
      ssl_cert_file: config.ssl_cert_file,
    }
  }
}

/// Generic Oauth credentials
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OauthCredentials {
  /// Whether this oauth method is available for usage.
  #[serde(default)]
  pub enabled: bool,
  /// The Oauth client id.
  #[serde(default)]
  pub id: String,
  /// The Oauth client secret.
  #[serde(default)]
  pub secret: String,
}

/// Provide AWS credentials for Komodo to use.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AwsCredentials {
  /// The aws ACCESS_KEY_ID
  pub access_key_id: String,
  /// The aws SECRET_ACCESS_KEY
  pub secret_access_key: String,
}

/// Provide configuration for a Github Webhook app.
#[derive(Debug, Clone, Deserialize)]
pub struct GithubWebhookAppConfig {
  /// Github app id
  pub app_id: i64,
  /// Configure the app installations on multiple accounts / organizations.
  pub installations: Vec<GithubWebhookAppInstallationConfig>,
  /// Private key path. Default: /github-private-key.pem.
  #[serde(default = "default_private_key_path")]
  pub pk_path: String,
}

fn default_private_key_path() -> String {
  String::from("/github/private-key.pem")
}

impl Default for GithubWebhookAppConfig {
  fn default() -> Self {
    GithubWebhookAppConfig {
      app_id: 0,
      installations: Default::default(),
      pk_path: default_private_key_path(),
    }
  }
}

/// Provide configuration for a Github Webhook app installation.
#[derive(Debug, Clone, Deserialize)]
pub struct GithubWebhookAppInstallationConfig {
  /// The installation ID
  pub id: i64,
  /// The user or organization name
  pub namespace: String,
}
