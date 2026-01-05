use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

pub mod cli;
pub mod core;
pub mod periphery;

fn default_config_keywords() -> Vec<String> {
  vec![String::from("*config.*")]
}

fn default_merge_nested_config() -> bool {
  true
}

fn default_extend_config_arrays() -> bool {
  true
}

/// Provide database connection information.
/// Komodo uses the MongoDB api driver for database communication,
/// and FerretDB to support Postgres and Sqlite storage options.
///
/// Must provide ONE of:
/// 1. `uri`
/// 2. `address` + `username` + `password`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
  /// Full mongo uri string, eg. `mongodb://username:password@your.mongo.int:27017`
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub uri: String,
  /// Just the address part of the mongo uri, eg `your.mongo.int:27017`
  #[serde(
    default = "default_database_address",
    skip_serializing_if = "String::is_empty"
  )]
  pub address: String,
  /// Mongo user username
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub username: String,
  /// Mongo user password
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub password: String,
  /// Mongo app name. default: `komodo_core`
  #[serde(default = "default_database_app_name")]
  pub app_name: String,
  /// Mongo db name. Which mongo database to create the collections in.
  /// Default: `komodo`.
  #[serde(default = "default_database_db_name")]
  pub db_name: String,
}

fn default_database_address() -> String {
  String::from("localhost:27017")
}

fn default_database_app_name() -> String {
  "komodo_core".to_string()
}

fn default_database_db_name() -> String {
  "komodo".to_string()
}

impl Default for DatabaseConfig {
  fn default() -> Self {
    Self {
      uri: Default::default(),
      address: default_database_address(),
      username: Default::default(),
      password: Default::default(),
      app_name: default_database_app_name(),
      db_name: default_database_db_name(),
    }
  }
}

fn default_database_config() -> &'static DatabaseConfig {
  static DEFAULT_DATABASE_CONFIG: OnceLock<DatabaseConfig> =
    OnceLock::new();
  DEFAULT_DATABASE_CONFIG.get_or_init(Default::default)
}

impl DatabaseConfig {
  pub fn sanitized(&self) -> DatabaseConfig {
    DatabaseConfig {
      uri: empty_or_redacted(&self.uri),
      address: self.address.clone(),
      username: empty_or_redacted(&self.username),
      password: empty_or_redacted(&self.password),
      app_name: self.app_name.clone(),
      db_name: self.db_name.clone(),
    }
  }

  pub fn is_default(&self) -> bool {
    self == default_database_config()
  }
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GitProvider {
  /// The git provider domain. Default: `github.com`.
  #[serde(default = "default_git_provider")]
  pub domain: String,
  /// Whether to use https. Default: true.
  #[serde(default = "default_git_https")]
  pub https: bool,
  /// The accounts on the git provider. Required.
  #[serde(alias = "account")]
  pub accounts: Vec<ProviderAccount>,
}

fn default_git_provider() -> String {
  String::from("github.com")
}

fn default_git_https() -> bool {
  true
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DockerRegistry {
  /// The docker provider domain. Default: `docker.io`.
  #[serde(default = "default_docker_provider")]
  pub domain: String,
  /// The accounts on the registry. Required.
  #[serde(alias = "account")]
  pub accounts: Vec<ProviderAccount>,
  /// Available organizations on the registry provider.
  /// Used to push an image under an organization's repo rather than an account's repo.
  #[serde(default, alias = "organization")]
  pub organizations: Vec<String>,
}

fn default_docker_provider() -> String {
  String::from("docker.io")
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  PartialOrd,
  Ord,
  Serialize,
  Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ProviderAccount {
  /// The account username. Required.
  #[serde(alias = "account")]
  pub username: String,
  /// The account access token. Required.
  #[serde(default, skip_serializing)]
  pub token: String,
}

/// Private-Public key utilities. (alias: `k`)
#[derive(Debug, Clone, clap::Subcommand)]
pub enum KeyCommand {
  /// Generate a new public / private key pair
  /// for use with Core - Periphery authentication.
  /// (aliases: `gen`, `g`)
  #[clap(alias = "gen", alias = "g")]
  Generate {
    /// Specify the format of the output.
    #[arg(long, short = 'f', default_value_t = KeyOutputFormat::Standard)]
    format: KeyOutputFormat,
  },

  /// Compute the public key for a given private key.
  /// (aliases: `comp`, `c`)
  #[clap(alias = "comp", alias = "c")]
  Compute {
    /// Pass the private key
    private_key: String,
    /// Specify the format of the output.
    #[arg(long, short = 'f', default_value_t = KeyOutputFormat::Standard)]
    format: KeyOutputFormat,
  },
}

#[derive(
  Debug, Clone, Copy, Default, strum::Display, clap::ValueEnum,
)]
#[strum(serialize_all = "lowercase")]
pub enum KeyOutputFormat {
  /// Readable output format. Default. (alias: `t`)
  #[default]
  #[clap(alias = "s")]
  Standard,
  /// Json (single line) output format. (alias: `j`)
  #[clap(alias = "j")]
  Json,
  /// Json "pretty" (multi line) output format. (alias: `jp`)
  #[clap(alias = "jp")]
  JsonPretty,
}

#[derive(Serialize)]
pub struct KeyPair<'a> {
  pub private_key: &'a str,
  pub public_key: &'a str,
}

pub fn empty_or_redacted(src: &str) -> String {
  if src.is_empty() {
    String::new()
  } else {
    String::from("##############")
  }
}
