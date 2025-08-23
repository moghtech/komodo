use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
  deserializers::string_list_deserializer,
  entities::{
    config::{DatabaseConfig, empty_or_redacted},
    logger::{LogConfig, LogLevel, StdioLogMode},
  },
};

pub mod args;

/// # Komodo CLI Environment Variables
///
///
#[derive(Debug, Clone, Deserialize)]
pub struct Env {
  // ============
  // Cli specific
  // ============
  /// Specify the config paths (files or folders) used to build up the
  /// final [CliConfig].
  /// If not provided, will use "." (the current working directory).
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(
    default = "default_config_paths",
    alias = "komodo_cli_config_path"
  )]
  pub komodo_cli_config_paths: Vec<PathBuf>,
  /// If specifying folders, use this to narrow down which
  /// files will be matched to parse into the final [CliConfig].
  /// Only files inside the folders which have names containing all keywords
  /// provided to `config_keywords` will be included.
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(
    default = "default_config_keywords",
    alias = "komodo_cli_config_keyword"
  )]
  pub komodo_cli_config_keywords: Vec<String>,
  /// Will merge nested config object (eg. database) across multiple
  /// config files. Default: `true`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default = "super::default_merge_nested_config")]
  pub komodo_cli_merge_nested_config: bool,
  /// Will extend config arrays (eg profiles) across multiple config files.
  /// Default: `true`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default = "super::default_extend_config_arrays")]
  pub komodo_cli_extend_config_arrays: bool,
  /// Extra logs during cli config load.
  #[serde(default)]
  pub komodo_cli_debug_startup: bool,
  // Override `default_profile`.
  pub komodo_cli_default_profile: Option<String>,
  /// Override `host` and `KOMODO_HOST`.
  pub komodo_cli_host: Option<String>,
  /// Override `cli_key`
  pub komodo_cli_key: Option<String>,
  /// Override `cli_secret`
  pub komodo_cli_secret: Option<String>,
  /// Override `table_borders`
  pub komodo_cli_table_borders: Option<CliTableBorders>,
  /// Override `backups_folder`
  pub komodo_cli_backups_folder: Option<PathBuf>,
  /// Override `max_backups`
  pub komodo_cli_max_backups: Option<u16>,

  /// Override `database_target_uri`
  #[serde(alias = "komodo_cli_database_copy_uri")]
  pub komodo_cli_database_target_uri: Option<String>,
  /// Override `database_target_address`
  #[serde(alias = "komodo_cli_database_copy_address")]
  pub komodo_cli_database_target_address: Option<String>,
  /// Override `database_target_username`
  #[serde(alias = "komodo_cli_database_copy_username")]
  pub komodo_cli_database_target_username: Option<String>,
  /// Override `database_target_password`
  #[serde(alias = "komodo_cli_database_copy_password")]
  pub komodo_cli_database_target_password: Option<String>,
  /// Override `database_target_db_name`
  #[serde(alias = "komodo_cli_database_copy_db_name")]
  pub komodo_cli_database_target_db_name: Option<String>,

  // LOGGING
  /// Override `logging.level`
  pub komodo_cli_logging_level: Option<LogLevel>,
  /// Override `logging.stdio`
  pub komodo_cli_logging_stdio: Option<StdioLogMode>,
  /// Override `logging.pretty`
  pub komodo_cli_logging_pretty: Option<bool>,
  /// Override `logging.otlp_endpoint`
  pub komodo_cli_logging_otlp_endpoint: Option<String>,
  /// Override `logging.opentelemetry_service_name`
  pub komodo_cli_logging_opentelemetry_service_name: Option<String>,
  /// Override `pretty_startup_config`
  pub komodo_cli_pretty_startup_config: Option<bool>,

  // ================
  // Same as Core env
  // ================
  /// Override `host`
  pub komodo_host: Option<String>,

  // DATABASE
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
  /// Override `database.db_name`
  #[serde(alias = "komodo_mongo_db_name")]
  pub komodo_database_db_name: Option<String>,
}

fn default_config_paths() -> Vec<PathBuf> {
  if let Ok(home) = std::env::var("HOME") {
    vec![
      PathBuf::from_str(&home).unwrap().join(".config/komodo"),
      PathBuf::from_str(".").unwrap(),
    ]
  } else {
    vec![PathBuf::from_str(".").unwrap()]
  }
}

fn default_config_keywords() -> Vec<String> {
  vec![String::from("*komodo.cli*.*")]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
  /// Optional. Only relevant for top level CLI config.
  /// Set a default profile to be used when none is provided.
  /// This allows for quick switching between profiles while
  /// not having to explicitly pass `-p profile`.
  #[serde(
    alias = "default",
    skip_serializing_if = "Option::is_none"
  )]
  pub default_profile: Option<String>,
  /// Optional. The profile name. (alias: `name`)
  /// Configure profiles with name in the komodo.cli.toml,
  /// and select them using `km -p profile-name ...`.
  #[serde(
    default,
    alias = "name",
    skip_serializing_if = "String::is_empty"
  )]
  pub config_profile: String,
  /// Optional. The profile aliases. (aliases: `aliases`, `alias`)
  /// Configure profiles with alias in the komodo.cli.toml,
  /// and select them using `km -p alias ...`.
  #[serde(
    default,
    alias = "aliases",
    alias = "alias",
    deserialize_with = "string_list_deserializer",
    skip_serializing_if = "Vec::is_empty"
  )]
  pub config_aliases: Vec<String>,
  // Same as Core
  /// The host Komodo url.
  /// Eg. "https://demo.komo.do"
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub host: String,
  /// The api key for the CLI to use
  #[serde(alias = "key", skip_serializing_if = "Option::is_none")]
  pub cli_key: Option<String>,
  /// The api secret for the CLI to use
  #[serde(alias = "secret", skip_serializing_if = "Option::is_none")]
  pub cli_secret: Option<String>,
  /// The format for the tables.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub table_borders: Option<CliTableBorders>,
  /// The root backups folder.
  ///
  /// Default: `/backups`.
  ///
  /// Backups will be created in timestamped folders eg
  /// `/backups/2025-08-04_05_05_53`
  #[serde(default = "default_backups_folder")]
  pub backups_folder: PathBuf,

  /// Specify the maximum number of backups to keep,
  /// or 0 to disable backup pruning.
  /// Default: `14`
  ///
  /// After every backup, the CLI will prune the oldest backups
  /// if there are more backups than `max_backups`
  #[serde(default = "default_max_backups")]
  pub max_backups: u16,
  // Same as Core
  /// Configure database connection
  #[serde(
    default = "default_database_config",
    alias = "mongo",
    skip_serializing_if = "database_config_is_default"
  )]
  pub database: DatabaseConfig,
  /// Configure restore / copy database connection
  #[serde(
    default = "default_database_config",
    alias = "database_copy",
    skip_serializing_if = "database_config_is_default"
  )]
  pub database_target: DatabaseConfig,
  /// Logging configuration
  #[serde(
    default = "default_log_config",
    skip_serializing_if = "log_config_is_default"
  )]
  pub cli_logging: LogConfig,
  /// Configure additional profiles.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub profile: Vec<CliConfig>,
}

fn default_backups_folder() -> PathBuf {
  // SAFE: /backups is a valid path.
  PathBuf::from_str("/backups").unwrap()
}

fn default_max_backups() -> u16 {
  14
}

fn default_database_config() -> DatabaseConfig {
  DatabaseConfig {
    app_name: String::from("komodo_cli"),
    ..Default::default()
  }
}

fn database_config_is_default(db_config: &DatabaseConfig) -> bool {
  db_config == &default_database_config()
}

fn default_log_config() -> LogConfig {
  LogConfig {
    location: false,
    ..Default::default()
  }
}

fn log_config_is_default(log_config: &LogConfig) -> bool {
  log_config == &default_log_config()
}

impl Default for CliConfig {
  fn default() -> Self {
    Self {
      default_profile: Default::default(),
      config_profile: Default::default(),
      config_aliases: Default::default(),
      cli_key: Default::default(),
      cli_secret: Default::default(),
      cli_logging: default_log_config(),
      table_borders: Default::default(),
      backups_folder: default_backups_folder(),
      max_backups: default_max_backups(),
      database: default_database_config(),
      database_target: default_database_config(),
      host: Default::default(),
      profile: Default::default(),
    }
  }
}

impl CliConfig {
  pub fn sanitized(&self) -> CliConfig {
    CliConfig {
      default_profile: self.default_profile.clone(),
      config_profile: self.config_profile.clone(),
      config_aliases: self.config_aliases.clone(),
      cli_key: self
        .cli_key
        .as_ref()
        .map(|cli_key| empty_or_redacted(cli_key)),
      cli_secret: self
        .cli_secret
        .as_ref()
        .map(|cli_secret| empty_or_redacted(cli_secret)),
      cli_logging: self.cli_logging.clone(),
      table_borders: self.table_borders,
      backups_folder: self.backups_folder.clone(),
      max_backups: self.max_backups,
      database_target: self.database_target.sanitized(),
      host: self.host.clone(),
      database: self.database.sanitized(),
      profile: self
        .profile
        .iter()
        .map(CliConfig::sanitized)
        .collect(),
    }
  }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum CliTableBorders {
  /// Only horizontal borders. Default.
  #[default]
  Horizontal,
  /// Only vertical borders.
  Vertical,
  /// Only borders around the outside of the table.
  Outside,
  /// Only borders horizontally / vertically between the rows / columns.
  Inside,
  /// All borders
  All,
}
