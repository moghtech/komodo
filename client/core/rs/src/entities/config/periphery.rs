//! # Configuring the Komodo Periphery Agent
//!
//! The periphery configuration is passed in three ways:
//! 1. Command line args ([CliArgs])
//! 2. Environment Variables ([Env])
//! 3. Configuration File ([PeripheryConfig])
//!
//! The final configuration is built by combining parameters
//! passed through the different methods. The priority of the args is
//! strictly hierarchical, meaning params passed with [CliArgs] have top priority,
//! followed by those passed in the environment, followed by those passed in
//! the configuration file.
//!

use clap::Parser;
use ipnetwork::IpNetwork;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

use crate::{
  deserializers::{
    ForgivingVec, option_string_list_deserializer,
    string_list_deserializer,
  },
  entities::{
    Timelength,
    logger::{LogConfig, LogLevel, StdioLogMode},
  },
};

use super::{
  DockerRegistry, GitProvider, ProviderAccount, empty_or_redacted,
};

/// # Periphery Command Line Arguments.
///
/// This structure represents the periphery command line arguments used to
/// configure the periphery agent. A help manual for the periphery binary
/// can be printed using `/path/to/periphery --help`.
///
/// Example command:
/// ```sh
/// periphery \
///   --config-path /path/to/periphery.config.base.toml \
///   --config-path /other_path/to/overide-periphery-config-directory \
///   --config-keyword periphery \
///   --config-keyword config \
///   --merge-nested-config true \
///   --extend-config-arrays false \
///   --log-level info
/// ```
#[derive(Parser)]
#[command(name = "periphery", author, about, version)]
pub struct CliArgs {
  /// Additional utilities.
  #[command(subcommand)]
  pub command: Option<Command>,

  /// Sets the path of a config file or directory to use.
  /// Can use multiple times
  #[arg(long, short = 'c')]
  pub config_path: Option<Vec<PathBuf>>,

  /// Sets the keywords to match directory periphery config file names on.
  /// Supports wildcard syntax.
  /// Can use multiple times to match multiple patterns independently.
  #[arg(long, short = 'm')]
  pub config_keyword: Option<Vec<String>>,

  /// Merges nested configs, eg. secrets, providers.
  /// Will override the equivalent env configuration.
  /// Default: true
  #[arg(long)]
  pub merge_nested_config: Option<bool>,

  /// Extends config arrays, eg. allowed_ips, passkeys.
  /// Will override the equivalent env configuration.
  /// Default: true
  #[arg(long)]
  pub extend_config_arrays: Option<bool>,

  /// Configure the logging level: error, warn, info, debug, trace.
  /// Default: info
  /// If passed, will override any other log_level set.
  #[arg(long)]
  pub log_level: Option<tracing::Level>,
}

#[cfg(feature = "cli")]
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
  /// Private-Public key utilities. (alias: `k`)
  #[clap(alias = "k")]
  Key {
    #[command(subcommand)]
    command: pki::cli::KeyCommand,
  },
}

/// # Periphery Environment Variables
///
/// The variables should be passed in the traditional `UPPER_SNAKE_CASE` format,
/// although the lower case format can still be parsed. If equivalent paramater is passed
/// in [CliArgs], the value passed to the environment will be ignored in favor of the cli arg.
#[derive(Deserialize)]
pub struct Env {
  /// Specify the config paths (files or folders) used to build up the
  /// final [PeripheryConfig].
  /// If not provided, will use Default config.
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default, alias = "periphery_config_path")]
  pub periphery_config_paths: Vec<PathBuf>,
  /// If specifying folders, use this to narrow down which
  /// files will be matched to parse into the final [PeripheryConfig].
  /// Only files inside the folders which have names containing a keywords
  /// provided to `config_keywords` will be included.
  /// Keywords support wildcard matching syntax.
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(
    default = "super::default_config_keywords",
    alias = "periphery_config_keyword"
  )]
  pub periphery_config_keywords: Vec<String>,

  /// Will merge nested config object (eg. secrets, providers) across multiple
  /// config files. Default: `true`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default = "super::default_merge_nested_config")]
  pub periphery_merge_nested_config: bool,

  /// Will extend config arrays (eg. `allowed_ips`, `passkeys`) across multiple config files.
  /// Default: `true`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default = "super::default_extend_config_arrays")]
  pub periphery_extend_config_arrays: bool,

  /// Override `private_key`
  pub periphery_private_key: Option<String>,
  /// Override `private_key` from file
  pub periphery_private_key_file: Option<PathBuf>,
  /// Override `onboarding_key`
  pub periphery_onboarding_key: Option<String>,
  /// Override `onboarding_key` from file
  pub periphery_onboarding_key_file: Option<PathBuf>,
  /// Override `core_public_keys`
  #[serde(alias = "periphery_core_public_key")]
  pub periphery_core_public_keys: Option<Vec<String>>,
  /// Override `passkeys`
  pub periphery_passkeys: Option<Vec<String>>,
  /// Override `passkeys` from file
  pub periphery_passkeys_file: Option<PathBuf>,
  /// Override `core_addresses`
  #[serde(alias = "periphery_core_address")]
  pub periphery_core_addresses: Option<Vec<String>>,
  /// Override `core_tls_insecure_skip_verify`
  pub periphery_core_tls_insecure_skip_verify: Option<bool>,
  /// Override `connect_as`
  pub periphery_connect_as: Option<String>,
  /// Override `server_enabled`
  pub periphery_server_enabled: Option<bool>,
  /// Override `port`
  pub periphery_port: Option<u16>,
  /// Override `bind_ip`
  pub periphery_bind_ip: Option<String>,
  /// Override `root_directory`
  pub periphery_root_directory: Option<PathBuf>,
  /// Override `repo_dir`
  pub periphery_repo_dir: Option<PathBuf>,
  /// Override `stack_dir`
  pub periphery_stack_dir: Option<PathBuf>,
  /// Override `build_dir`
  pub periphery_build_dir: Option<PathBuf>,
  /// Override `default_terminal_command`
  pub periphery_default_terminal_command: Option<String>,
  /// Override `disable_terminals`
  pub periphery_disable_terminals: Option<bool>,
  /// Override `disable_container_terminals`
  #[serde(alias = "periphery_disable_container_exec")]
  pub periphery_disable_container_terminals: Option<bool>,
  /// Override `stats_polling_rate`
  pub periphery_stats_polling_rate: Option<Timelength>,
  /// Override `container_stats_polling_rate`
  pub periphery_container_stats_polling_rate: Option<Timelength>,
  /// Override `legacy_compose_cli`
  pub periphery_legacy_compose_cli: Option<bool>,

  // LOGGING
  /// Override `logging.level`
  pub periphery_logging_level: Option<LogLevel>,
  /// Override `logging.stdio`
  pub periphery_logging_stdio: Option<StdioLogMode>,
  /// Override `logging.pretty`
  pub periphery_logging_pretty: Option<bool>,
  /// Override `logging.location`
  pub periphery_logging_location: Option<bool>,
  /// Override `logging.ansi`
  pub periphery_logging_ansi: Option<bool>,
  /// Override `logging.otlp_endpoint`
  pub periphery_logging_otlp_endpoint: Option<String>,
  /// Override `logging.opentelemetry_service_name`
  pub periphery_logging_opentelemetry_service_name: Option<String>,
  /// Override `logging.opentelemetry_scope_name`
  pub periphery_logging_opentelemetry_scope_name: Option<String>,
  /// Override `pretty_startup_config`
  pub periphery_pretty_startup_config: Option<bool>,

  /// Override `allowed_ips`
  pub periphery_allowed_ips: Option<ForgivingVec<IpNetwork>>,
  /// Override `include_disk_mounts`
  pub periphery_include_disk_mounts: Option<ForgivingVec<PathBuf>>,
  /// Override `exclude_disk_mounts`
  pub periphery_exclude_disk_mounts: Option<ForgivingVec<PathBuf>>,

  /// Override `ssl_enabled`
  pub periphery_ssl_enabled: Option<bool>,
  /// Override `ssl_key_file`
  pub periphery_ssl_key_file: Option<String>,
  /// Override `ssl_cert_file`
  pub periphery_ssl_cert_file: Option<String>,
}

/// # Periphery Configuration File
///
/// Refer to the [example file](https://github.com/moghtech/komodo/blob/main/config/periphery.config.toml) for a full example.
#[derive(Debug, Clone, Deserialize)]
pub struct PeripheryConfig {
  /// The private key used with noise handshake.
  ///
  /// Supports openssl generated pem file, `openssl genpkey -algorithm X25519 -out private.key`.
  /// To load from file, use `private_key = "file:/path/to/private.key"`
  ///
  /// If a file is specified and does not exist, will try to generate one at the path
  /// and use it going forward.
  ///
  /// Default: ${root_directory}/keys/periphery.key
  #[serde(skip_serializing_if = "Option::is_none")]
  pub private_key: Option<String>,

  /// Provide an onboarding key to use with the new Server
  /// creation flow.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub onboarding_key: Option<String>,

  /// Optionally pin a specific Core public key for additional trust.
  ///
  /// Supports openssl generated pem file, `openssl pkey -in private.key -pubout -out public.key`.
  /// To load from file, include `file:/path/to/public.key` in the list.
  ///
  /// If not provided and `core_addresses` are defined, defaults to ["file:${root_directory}/keys/core.pub"]
  #[serde(
    default,
    alias = "core_public_key",
    deserialize_with = "option_string_list_deserializer",
    skip_serializing_if = "Option::is_none"
  )]
  pub core_public_keys: Option<Vec<String>>,
  /// Deprecated. Legacy v1 compatibility.
  /// Users should upgrade to private / public key authentication.
  /// Can only be used with Core -> Periphery connection.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub passkeys: Option<Vec<String>>,

  // =======================
  // = OUTBOUND CONNECTION =
  // =======================
  /// Address of Komodo Core when connecting outbound
  #[serde(
    default,
    alias = "core_address",
    deserialize_with = "string_list_deserializer"
  )]
  pub core_addresses: Vec<String>,

  /// Allow Periphery to connect to Core
  /// without validating the Core certs
  #[serde(default)]
  pub core_tls_insecure_skip_verify: bool,

  /// Server name / id to connect as
  #[serde(default)]
  pub connect_as: String,

  // ======================
  // = INBOUND CONNECTION =
  // ======================
  /// Enable the inbound connection server.
  ///
  /// - If `core_addresses` set, defaults to `false`.
  /// - If `core_addresses` unset, defaults to `true`.
  pub server_enabled: Option<bool>,

  /// The port periphery will run on.
  /// Default: `8120`
  #[serde(default = "default_periphery_port")]
  pub port: u16,

  /// IP address the periphery server binds to.
  /// Default: [::].
  #[serde(default = "default_periphery_bind_ip")]
  pub bind_ip: String,

  /// Limits which IP addresses are allowed to call the api.
  /// Default: none
  ///
  /// Note: this should be configured to increase security.
  #[serde(default)]
  pub allowed_ips: ForgivingVec<IpNetwork>,

  /// Whether to enable ssl.
  /// Default: true
  #[serde(default = "default_ssl_enabled")]
  pub ssl_enabled: bool,

  /// Path to the ssl key.
  /// Default: `${root_directory}/ssl/key.pem`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ssl_key_file: Option<String>,

  /// Path to the ssl cert.
  /// Default: `${root_directory}/ssl/cert.pem`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ssl_cert_file: Option<String>,

  // ==================
  // = OTHER SETTINGS =
  // ==================
  /// The directory Komodo will use as the default root for the specific (repo, stack, build) directories.
  ///
  /// repo: ${root_directory}/repos
  /// stack: ${root_directory}/stacks
  /// build: ${root_directory}/builds
  ///
  /// Note. These can each be overridden with a specific directory
  /// by specifying `repo_dir`, `stack_dir`, or `build_dir` explicitly
  ///
  /// Default: `/etc/komodo`
  #[serde(default = "default_root_directory")]
  pub root_directory: PathBuf,

  /// The system directory where Komodo managed repos will be cloned.
  /// If not provided, will default to `${root_directory}/repos`.
  /// Default: empty
  #[serde(skip_serializing_if = "Option::is_none")]
  pub repo_dir: Option<PathBuf>,

  /// The system directory where stacks will managed.
  /// If not provided, will default to `${root_directory}/stacks`.
  /// Default: empty
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stack_dir: Option<PathBuf>,

  /// The system directory where builds will managed.
  /// If not provided, will default to `${root_directory}/builds`.
  /// Default: empty
  #[serde(skip_serializing_if = "Option::is_none")]
  pub build_dir: Option<PathBuf>,

  /// Configure the default terminal command
  /// when one isn't provided.
  /// Default: `bash`
  #[serde(default = "default_default_terminal_command")]
  pub default_terminal_command: String,

  /// Whether to disable the create terminal
  /// and disallow direct remote shell access.
  /// Default: false
  #[serde(default)]
  pub disable_terminals: bool,

  /// Whether to disable the container exec / attach api
  /// and disallow remote container shell access.
  /// Default: false
  #[serde(default, alias = "disable_container_exec")]
  pub disable_container_terminals: bool,

  /// The rate at which the system stats will be polled to update the cache.
  /// Options: https://docs.rs/komodo_client/latest/komodo_client/entities/enum.Timelength.html
  /// Default: `5-sec`
  #[serde(default = "default_stats_polling_rate")]
  pub stats_polling_rate: Timelength,

  /// The rate at which the container stats will be polled to update the cache.
  /// Options: https://docs.rs/komodo_client/latest/komodo_client/entities/enum.Timelength.html
  /// Default: `30-sec`
  #[serde(default = "default_container_stats_polling_rate")]
  pub container_stats_polling_rate: Timelength,

  /// Whether stack actions should use `docker-compose ...`
  /// instead of `docker compose ...`.
  /// Default: false
  #[serde(default)]
  pub legacy_compose_cli: bool,

  /// Logging configuration
  #[serde(default)]
  pub logging: LogConfig,

  /// Pretty-log (multi-line) the startup config
  /// for easier human readability.
  #[serde(default)]
  pub pretty_startup_config: bool,

  /// If non-empty, only includes specific mount paths in the disk report.
  #[serde(default)]
  pub include_disk_mounts: ForgivingVec<PathBuf>,

  /// Exclude specific mount paths in the disk report.
  #[serde(default)]
  pub exclude_disk_mounts: ForgivingVec<PathBuf>,

  /// Mapping on local periphery secrets. These can be interpolated into eg. Deployment environment variables.
  /// Default: none
  #[serde(default)]
  pub secrets: HashMap<String, String>,

  /// Configure git credentials used to clone private repos.
  /// Supports any git provider.
  #[serde(default, alias = "git_provider")]
  pub git_providers: ForgivingVec<GitProvider>,

  /// Configure docker credentials used to push / pull images.
  /// Supports any docker image repository.
  #[serde(default, alias = "docker_registry")]
  pub docker_registries: ForgivingVec<DockerRegistry>,
}

fn default_periphery_port() -> u16 {
  8120
}

fn default_periphery_bind_ip() -> String {
  "[::]".to_string()
}

fn default_root_directory() -> PathBuf {
  "/etc/komodo".parse().unwrap()
}

fn default_default_terminal_command() -> String {
  String::from("bash")
}

fn default_stats_polling_rate() -> Timelength {
  Timelength::FiveSeconds
}

fn default_container_stats_polling_rate() -> Timelength {
  Timelength::ThirtySeconds
}

fn default_ssl_enabled() -> bool {
  true
}

impl Default for PeripheryConfig {
  fn default() -> Self {
    Self {
      private_key: None,
      onboarding_key: None,
      core_public_keys: None,
      passkeys: None,
      core_addresses: Default::default(),
      core_tls_insecure_skip_verify: Default::default(),
      connect_as: Default::default(),
      server_enabled: Default::default(),
      port: default_periphery_port(),
      bind_ip: default_periphery_bind_ip(),
      root_directory: default_root_directory(),
      repo_dir: None,
      stack_dir: None,
      build_dir: None,
      default_terminal_command: default_default_terminal_command(),
      disable_terminals: Default::default(),
      disable_container_terminals: Default::default(),
      stats_polling_rate: default_stats_polling_rate(),
      container_stats_polling_rate:
        default_container_stats_polling_rate(),
      legacy_compose_cli: Default::default(),
      logging: Default::default(),
      pretty_startup_config: Default::default(),
      allowed_ips: Default::default(),
      include_disk_mounts: Default::default(),
      exclude_disk_mounts: Default::default(),
      secrets: Default::default(),
      git_providers: Default::default(),
      docker_registries: Default::default(),
      ssl_enabled: default_ssl_enabled(),
      ssl_key_file: None,
      ssl_cert_file: None,
    }
  }
}

impl PeripheryConfig {
  pub fn sanitized(&self) -> PeripheryConfig {
    PeripheryConfig {
      private_key: self.private_key.as_ref().map(|private_key| {
        if private_key.starts_with("file:") {
          private_key.clone()
        } else {
          empty_or_redacted(private_key)
        }
      }),
      onboarding_key: self
        .onboarding_key
        .as_ref()
        .map(|key| empty_or_redacted(key)),
      core_public_keys: self.core_public_keys.clone(),
      passkeys: self.passkeys.as_ref().map(|passkeys| {
        passkeys.iter().map(|p| empty_or_redacted(p)).collect()
      }),
      core_addresses: self.core_addresses.clone(),
      core_tls_insecure_skip_verify: self
        .core_tls_insecure_skip_verify,
      connect_as: self.connect_as.clone(),
      server_enabled: self.server_enabled,
      port: self.port,
      bind_ip: self.bind_ip.clone(),
      root_directory: self.root_directory.clone(),
      repo_dir: self.repo_dir.clone(),
      stack_dir: self.stack_dir.clone(),
      build_dir: self.build_dir.clone(),
      default_terminal_command: self.default_terminal_command.clone(),
      disable_terminals: self.disable_terminals,
      disable_container_terminals: self.disable_container_terminals,
      stats_polling_rate: self.stats_polling_rate,
      container_stats_polling_rate: self.container_stats_polling_rate,
      legacy_compose_cli: self.legacy_compose_cli,
      logging: self.logging.clone(),
      pretty_startup_config: self.pretty_startup_config,
      allowed_ips: self.allowed_ips.clone(),
      include_disk_mounts: self.include_disk_mounts.clone(),
      exclude_disk_mounts: self.exclude_disk_mounts.clone(),
      secrets: self
        .secrets
        .iter()
        .map(|(var, secret)| {
          (var.to_string(), empty_or_redacted(secret))
        })
        .collect(),
      git_providers: self
        .git_providers
        .iter()
        .map(|provider| GitProvider {
          domain: provider.domain.clone(),
          https: provider.https,
          accounts: provider
            .accounts
            .iter()
            .map(|account| ProviderAccount {
              username: account.username.clone(),
              token: empty_or_redacted(&account.token),
            })
            .collect(),
        })
        .collect(),
      docker_registries: self
        .docker_registries
        .iter()
        .map(|provider| DockerRegistry {
          domain: provider.domain.clone(),
          organizations: provider.organizations.clone(),
          accounts: provider
            .accounts
            .iter()
            .map(|account| ProviderAccount {
              username: account.username.clone(),
              token: empty_or_redacted(&account.token),
            })
            .collect(),
        })
        .collect(),
      ssl_enabled: self.ssl_enabled,
      ssl_key_file: self.ssl_key_file.clone(),
      ssl_cert_file: self.ssl_cert_file.clone(),
    }
  }

  /// If `server_enabled` is None, defaults based on
  /// whether there are any core_addresses defined.
  pub fn server_enabled(&self) -> bool {
    self
      .server_enabled
      .unwrap_or(self.core_addresses.is_empty())
  }

  pub fn core_public_keys_spec(&self) -> Option<Vec<String>> {
    // Return explicitly set public key spec.
    if let Some(public_keys) = self.core_public_keys.clone() {
      return Some(public_keys);
    };
    // If server enabled, pass through empty public keys exactly
    if self.server_enabled() {
      return None;
    }
    // Defaults to $root_directory/keys/core.pub for Periphery -> Core.
    // If it doesn't exist, will be auto written on first connection with Core.
    let path = format!(
      "file:{}",
      self.root_directory.join("keys/core.pub").display()
    );
    Some(vec![path])
  }

  pub fn repo_dir(&self) -> PathBuf {
    if let Some(dir) = &self.repo_dir {
      dir.to_owned()
    } else {
      self.root_directory.join("repos")
    }
  }

  pub fn stack_dir(&self) -> PathBuf {
    if let Some(dir) = &self.stack_dir {
      dir.to_owned()
    } else {
      self.root_directory.join("stacks")
    }
  }

  pub fn build_dir(&self) -> PathBuf {
    if let Some(dir) = &self.build_dir {
      dir.to_owned()
    } else {
      self.root_directory.join("builds")
    }
  }

  pub fn ssl_key_file(&self) -> PathBuf {
    if let Some(dir) = &self.ssl_key_file {
      dir.into()
    } else {
      self.root_directory.join("ssl/key.pem")
    }
  }

  pub fn ssl_cert_file(&self) -> PathBuf {
    if let Some(dir) = &self.ssl_cert_file {
      dir.into()
    } else {
      self.root_directory.join("ssl/cert.pem")
    }
  }
}

impl mogh_server::ServerConfig for &PeripheryConfig {
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
    static SSL_KEY_FILE: OnceLock<String> = OnceLock::new();
    SSL_KEY_FILE.get_or_init(|| {
      PeripheryConfig::ssl_key_file(self)
        .into_os_string()
        .into_string()
        .expect("Invalid ssl key file path.")
    })
  }
  fn ssl_cert_file(&self) -> &str {
    static SSL_CERT_FILE: OnceLock<String> = OnceLock::new();
    SSL_CERT_FILE.get_or_init(|| {
      PeripheryConfig::ssl_cert_file(self)
        .into_os_string()
        .into_string()
        .expect("Invalid ssl cert file path.")
    })
  }
}
