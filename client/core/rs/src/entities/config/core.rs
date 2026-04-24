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

use std::{collections::HashMap, path::PathBuf};

use mogh_auth_client::config::NamedOauthConfig;
use serde::Deserialize;

use crate::{
  deserializers::option_string_list_deserializer,
  entities::{
    Timelength,
    config::DatabaseConfig,
    logger::{LogConfig, LogLevel, StdioLogMode},
  },
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
  /// Override `private_key`
  pub komodo_private_key: Option<String>,
  /// Override `private_key` with file
  pub komodo_private_key_file: Option<PathBuf>,
  /// Override `periphery_public_keys`
  #[serde(alias = "komodo_periphery_public_key")]
  pub komodo_periphery_public_keys: Option<Vec<String>>,
  /// Override `passkey`
  pub komodo_passkey: Option<String>,
  /// Override `passkey` from file
  pub komodo_passkey_file: Option<PathBuf>,
  /// Override `timezone`
  #[serde(alias = "tz")]
  pub komodo_timezone: Option<String>,
  /// Override `first_server_name`
  pub komodo_first_server_name: Option<String>,
  /// Override `first_server_address`
  #[serde(alias = "komodo_first_server")]
  pub komodo_first_server_address: Option<String>,
  /// Override `jwt_secret`
  pub komodo_jwt_secret: Option<String>,
  /// Override `jwt_secret` from file
  pub komodo_jwt_secret_file: Option<PathBuf>,
  /// Override `jwt_ttl`
  pub komodo_jwt_ttl: Option<Timelength>,

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
  /// Override `min_password_length`
  pub komodo_min_password_length: Option<u16>,
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
  /// Override `oidc_auto_redirect`
  pub komodo_oidc_auto_redirect: Option<bool>,

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

  /// Override `auth_rate_limit_disabled`
  pub komodo_auth_rate_limit_disabled: Option<bool>,
  /// Override `auth_rate_limit_max_attempts`
  pub komodo_auth_rate_limit_max_attempts: Option<u16>,
  /// Override `auth_rate_limit_window_seconds`
  pub komodo_auth_rate_limit_window_seconds: Option<u64>,

  /// Override `cors_allowed_origins`
  pub komodo_cors_allowed_origins: Option<Vec<String>>,
  /// Override `cors_allow_credentials`
  pub komodo_cors_allow_credentials: Option<bool>,
  /// Override `session_allow_cross_site`
  pub komodo_session_allow_cross_site: Option<bool>,

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

  /// Override `logging.level`
  pub komodo_logging_level: Option<LogLevel>,
  /// Override `logging.stdio`
  pub komodo_logging_stdio: Option<StdioLogMode>,
  /// Override `logging.pretty`
  pub komodo_logging_pretty: Option<bool>,
  /// Override `logging.location`
  pub komodo_logging_location: Option<bool>,
  /// Override `logging.ansi`
  pub komodo_logging_ansi: Option<bool>,
  /// Override `logging.timestamps`
  pub komodo_logging_timestamps: Option<bool>,
  /// Override `logging.otlp_endpoint`
  pub komodo_logging_otlp_endpoint: Option<String>,
  /// Override `logging.opentelemetry_service_name`
  pub komodo_logging_opentelemetry_service_name: Option<String>,
  /// Override `logging.opentelemetry_scope_name`
  pub komodo_logging_opentelemetry_scope_name: Option<String>,
  /// Override `pretty_startup_config`
  pub komodo_pretty_startup_config: Option<bool>,
  /// Override `unsafe_unsanitized_startup_config`
  pub komodo_unsafe_unsanitized_startup_config: Option<bool>,

  /// Override `internet_interface`
  pub komodo_internet_interface: Option<String>,

  /// Override `ssl_enabled`.
  pub komodo_ssl_enabled: Option<bool>,
  /// Override `ssl_key_file`
  pub komodo_ssl_key_file: Option<String>,
  /// Override `ssl_cert_file`
  pub komodo_ssl_cert_file: Option<String>,

  /// Override `ui_path`
  pub komodo_ui_path: Option<String>,
  /// Override `ui_index_force_no_cache`
  pub komodo_ui_index_force_no_cache: Option<bool>,
  /// Override `sync_directory`
  pub komodo_sync_directory: Option<PathBuf>,
  /// Override `repo_directory`
  pub komodo_repo_directory: Option<PathBuf>,
  /// Override `action_directory`
  pub komodo_action_directory: Option<PathBuf>,
}

fn default_core_config_paths() -> Vec<PathBuf> {
  vec![PathBuf::from("/config")]
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

  /// Private key to use with Noise handshake to authenticate with Periphery agents.
  ///
  /// Supports openssl generated pem file, `openssl genpkey -algorithm X25519 -out private.key`.
  /// To load from file, use `private_key = "file:/path/to/private.key"`.
  ///
  /// If a file is specified and does not exist, will try to generate one at the path
  /// and use it going forward.
  ///
  /// Note. The private key used can be overridden for individual Servers / Builders.
  ///
  /// Default: file:/config/keys/core.key
  #[serde(default = "default_private_key")]
  pub private_key: String,

  /// Default accepted public keys to allow Periphery to connect.
  /// Core gains knowledge of the Periphery public key through the noise handshake.
  /// If not provided, Periphery -> Core connected Servers must
  /// configure accepted public key individually.
  ///
  /// Supports multiple public keys seperated by commas or newlines.
  ///
  /// Supports openssl generated pem file, `openssl pkey -in private.key -pubout -out public.key`.
  /// To load from file, include `file:/path/to/public.key` in the list.
  ///
  /// Note: If used, the accepted public key can still be overridden on individual Servers / Builders
  #[serde(
    default,
    alias = "periphery_public_key",
    deserialize_with = "option_string_list_deserializer",
    skip_serializing_if = "Option::is_none"
  )]
  pub periphery_public_keys: Option<Vec<String>>,

  /// Deprecated. Legacy v1 compatibility.
  /// Users should upgrade to private / public key authentication.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub passkey: Option<String>,

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

  /// If defined, ensure an enabled first server exists with the name.
  /// If None and "first_server_address" is defined, will default to "Local".
  /// Set this and not 'first_server_address' for Periphery -> Core Server.
  /// Default: None
  #[serde(skip_serializing_if = "Option::is_none")]
  pub first_server_name: Option<String>,

  /// If defined, ensure an enabled first server exists at this address.
  /// Example: `wss://periphery:8120`.
  /// In v1, was just 'first_server', maintains backward compatibility via alias.
  #[serde(
    alias = "first_server",
    skip_serializing_if = "Option::is_none"
  )]
  pub first_server_address: Option<String>,

  /// Configure database connection
  #[serde(default, alias = "mongo")]
  pub database: DatabaseConfig,

  // ================
  // = Auth / Login =
  // ================
  /// Enable login with local auth
  #[serde(default)]
  pub local_auth: bool,

  /// Configure a minimum password length.
  /// Default: 1
  #[serde(default = "default_min_password_length")]
  pub min_password_length: u16,

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

  /// Automatically redirect unauthenticated users to the OIDC provider
  /// instead of showing the login page.
  /// Users can bypass the redirect by appending `?disableAutoLogin` to the login URL.
  #[serde(default)]
  pub oidc_auto_redirect: bool,

  // =========
  // = Oauth =
  // =========
  /// Configure google oauth
  #[serde(default)]
  pub google_oauth: NamedOauthConfig,

  /// Configure github oauth
  #[serde(default)]
  pub github_oauth: NamedOauthConfig,

  // =================
  // = Rate Limiting =
  // =================
  /// Disable the auth rate limiter.
  #[serde(default)]
  pub auth_rate_limit_disabled: bool,

  /// Set the max allowed attempts per IP
  #[serde(default = "default_auth_rate_limit_max_attempts")]
  pub auth_rate_limit_max_attempts: u16,

  #[serde(default = "default_auth_rate_limit_window_seconds")]
  pub auth_rate_limit_window_seconds: u64,

  // =======
  // = CORS =
  // =======
  /// List of CORS allowed origins.
  /// If empty, allows all origins (`*`).
  /// Production setups should configure this explicitly.
  /// Example: `["https://komodo.example.com", "https://app.example.com"]`.
  #[serde(default)]
  pub cors_allowed_origins: Vec<String>,

  /// Tell CORS to allow credentials in requests.
  /// Used if needed for authentication proxy.
  #[serde(default)]
  pub cors_allow_credentials: bool,

  /// Use SameSite=None (actually allows samesite) instead of SameSite=Lax.
  /// The third option, SameSite=Strict, won't work with external login,
  /// as the session cookie will be lost on redirect with auth provider.
  #[serde(default)]
  pub session_allow_cross_site: bool,

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
  #[serde(default)]
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
  pub ssl_key_file: String,

  /// Path to the ssl cert.
  /// Default: `/config/ssl/cert.pem`.
  #[serde(default = "default_ssl_cert_file")]
  pub ssl_cert_file: String,

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

  /// The path to the built ui folder.
  #[serde(default = "default_ui_path")]
  pub ui_path: String,

  /// Force the `index.html` to served with
  /// 'Cache-Content: no-cache' header instead
  /// of using content hash as ETag.
  #[serde(default)]
  pub ui_index_force_no_cache: bool,
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

fn default_private_key() -> String {
  String::from("file:/config/keys/core.key")
}

fn default_ui_path() -> String {
  "/app/ui".to_string()
}

fn default_jwt_ttl() -> Timelength {
  Timelength::OneDay
}

fn default_min_password_length() -> u16 {
  1
}

fn default_init_admin_password() -> String {
  String::from("changeme")
}

fn default_auth_rate_limit_max_attempts() -> u16 {
  5
}

fn default_auth_rate_limit_window_seconds() -> u64 {
  15
}

fn default_sync_directory() -> PathBuf {
  PathBuf::from("/syncs")
}

fn default_repo_directory() -> PathBuf {
  PathBuf::from("/repo-cache")
}

fn default_action_directory() -> PathBuf {
  PathBuf::from("/action-cache")
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

fn default_ssl_key_file() -> String {
  "/config/ssl/key.pem".to_string()
}

fn default_ssl_cert_file() -> String {
  "/config/ssl/cert.pem".to_string()
}

impl Default for CoreConfig {
  fn default() -> Self {
    Self {
      title: default_title(),
      host: default_host(),
      port: default_core_port(),
      bind_ip: default_core_bind_ip(),
      internet_interface: Default::default(),
      private_key: Default::default(),
      periphery_public_keys: Default::default(),
      passkey: Default::default(),
      timezone: Default::default(),
      ui_write_disabled: Default::default(),
      disable_confirm_dialog: Default::default(),
      disable_websocket_reconnect: Default::default(),
      disable_init_resources: Default::default(),
      enable_fancy_toml: Default::default(),
      first_server_address: Default::default(),
      first_server_name: Default::default(),
      database: Default::default(),
      local_auth: Default::default(),
      min_password_length: default_min_password_length(),
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
      oidc_auto_redirect: Default::default(),
      google_oauth: Default::default(),
      github_oauth: Default::default(),
      auth_rate_limit_disabled: Default::default(),
      auth_rate_limit_max_attempts:
        default_auth_rate_limit_max_attempts(),
      auth_rate_limit_window_seconds:
        default_auth_rate_limit_window_seconds(),
      cors_allowed_origins: Default::default(),
      cors_allow_credentials: Default::default(),
      session_allow_cross_site: Default::default(),
      webhook_secret: Default::default(),
      webhook_base_url: Default::default(),
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
      ui_path: default_ui_path(),
      ui_index_force_no_cache: Default::default(),
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
      private_key: if self.private_key.starts_with("file:") {
        self.private_key.clone()
      } else {
        empty_or_redacted(&self.private_key)
      },
      periphery_public_keys: config.periphery_public_keys,
      passkey: config.passkey.as_deref().map(empty_or_redacted),
      timezone: config.timezone,
      first_server_address: config.first_server_address,
      first_server_name: config.first_server_name,
      jwt_secret: empty_or_redacted(&config.jwt_secret),
      jwt_ttl: config.jwt_ttl,
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
      min_password_length: config.min_password_length,
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
      oidc_auto_redirect: config.oidc_auto_redirect,
      google_oauth: NamedOauthConfig {
        enabled: config.google_oauth.enabled,
        client_id: empty_or_redacted(&config.google_oauth.client_id),
        client_secret: empty_or_redacted(
          &config.google_oauth.client_secret,
        ),
      },
      github_oauth: NamedOauthConfig {
        enabled: config.github_oauth.enabled,
        client_id: empty_or_redacted(&config.github_oauth.client_id),
        client_secret: empty_or_redacted(
          &config.github_oauth.client_secret,
        ),
      },
      auth_rate_limit_disabled: config.auth_rate_limit_disabled,
      auth_rate_limit_max_attempts: config
        .auth_rate_limit_max_attempts,
      auth_rate_limit_window_seconds: config
        .auth_rate_limit_window_seconds,
      cors_allowed_origins: config.cors_allowed_origins,
      cors_allow_credentials: config.cors_allow_credentials,
      session_allow_cross_site: config.session_allow_cross_site,
      webhook_secret: empty_or_redacted(&config.webhook_secret),
      webhook_base_url: config.webhook_base_url,
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
      ui_path: config.ui_path,
      ui_index_force_no_cache: config.ui_index_force_no_cache,
      repo_directory: config.repo_directory,
      action_directory: config.action_directory,
      sync_directory: config.sync_directory,
    }
  }

  pub fn oidc_enabled(&self) -> bool {
    self.oidc_enabled
      && !self.oidc_provider.is_empty()
      && !self.oidc_client_id.is_empty()
  }
}

/// Provide AWS credentials for Komodo to use.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AwsCredentials {
  /// The aws ACCESS_KEY_ID
  pub access_key_id: String,
  /// The aws SECRET_ACCESS_KEY
  pub secret_access_key: String,
}

impl mogh_server::ServerConfig for &CoreConfig {
  fn bind_ip(&self) -> &str {
    &self.bind_ip
  }
  fn port(&self) -> u16 {
    self.port
  }
  fn ssl_enabled(&self) -> bool {
    self.ssl_enabled
  }
  fn ssl_key_file(&self) -> &str {
    &self.ssl_key_file
  }
  fn ssl_cert_file(&self) -> &str {
    &self.ssl_cert_file
  }
}

impl mogh_server::cors::CorsConfig for &CoreConfig {
  fn allowed_origins(&self) -> &[String] {
    &self.cors_allowed_origins
  }
  fn allow_credentials(&self) -> bool {
    self.cors_allow_credentials
  }
}

impl mogh_server::session::SessionConfig for &CoreConfig {
  fn host(&self) -> &str {
    &self.host
  }
  fn host_env_field(&self) -> &str {
    "KOMODO_HOST"
  }
  fn expiry_seconds(&self) -> i64 {
    60 * 3
  }
  fn allow_cross_site(&self) -> bool {
    self.session_allow_cross_site
  }
}
