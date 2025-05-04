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

use std::{collections::HashMap, net::IpAddr, path::PathBuf};

use clap::Parser;
use serde::Deserialize;

use crate::entities::{
  Timelength,
  logger::{LogConfig, LogLevel, StdioLogMode},
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
  /// Sets the path of a config file or directory to use.
  /// Can use multiple times
  #[arg(short, long)]
  pub config_path: Option<Vec<String>>,

  /// Sets the keywords to match directory periphery config file names on.
  /// Can use multiple times.
  #[arg(long)]
  pub config_keyword: Option<Vec<String>>,

  /// Merges nested configs, eg. secrets, providers.
  /// Will override the equivalent env configuration.
  /// Default: false
  #[arg(long)]
  pub merge_nested_config: Option<bool>,

  /// Extends config arrays, eg. allowed_ips, passkeys.
  /// Will override the equivalent env configuration.
  /// Default: false
  #[arg(long)]
  pub extend_config_arrays: Option<bool>,

  /// Configure the logging level: error, warn, info, debug, trace.
  /// Default: info
  /// If passed, will override any other log_level set.
  #[arg(long)]
  pub log_level: Option<tracing::Level>,
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
  pub periphery_config_paths: Vec<String>,
  /// If specifying folders, use this to narrow down which
  /// files will be matched to parse into the final [PeripheryConfig].
  /// Only files inside the folders which have names containing all keywords
  /// provided to `config_keywords` will be included.
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default, alias = "periphery_config_keyword")]
  pub periphery_config_keywords: Vec<String>,

  /// Will merge nested config object (eg. secrets, providers) across multiple
  /// config files. Default: `false`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default)]
  pub periphery_merge_nested_config: bool,

  /// Will extend config arrays (eg. `allowed_ips`, `passkeys`) across multiple config files.
  /// Default: `false`
  ///
  /// Note. This is overridden if the equivalent arg is passed in [CliArgs].
  #[serde(default)]
  pub periphery_extend_config_arrays: bool,

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
  /// Override `disable_terminals`
  pub periphery_disable_terminals: Option<bool>,
  /// Override `stats_polling_rate`
  pub periphery_stats_polling_rate: Option<Timelength>,
  /// Override `legacy_compose_cli`
  pub periphery_legacy_compose_cli: Option<bool>,

  // LOGGING
  /// Override `logging.level`
  pub periphery_logging_level: Option<LogLevel>,
  /// Override `logging.stdio`
  pub periphery_logging_stdio: Option<StdioLogMode>,
  /// Override `logging.pretty`
  pub periphery_logging_pretty: Option<bool>,
  /// Override `logging.otlp_endpoint`
  pub periphery_logging_otlp_endpoint: Option<String>,
  /// Override `logging.opentelemetry_service_name`
  pub periphery_logging_opentelemetry_service_name: Option<String>,

  /// Override `allowed_ips`
  pub periphery_allowed_ips: Option<Vec<IpAddr>>,
  /// Override `passkeys`
  pub periphery_passkeys: Option<Vec<String>>,
  /// Override `passkeys` from file
  pub periphery_passkeys_file: Option<PathBuf>,
  /// Override `include_disk_mounts`
  pub periphery_include_disk_mounts: Option<Vec<PathBuf>>,
  /// Override `exclude_disk_mounts`
  pub periphery_exclude_disk_mounts: Option<Vec<PathBuf>>,

  /// Override `ssl_enabled`
  pub periphery_ssl_enabled: Option<bool>,
  /// Override `ssl_key_file`
  pub periphery_ssl_key_file: Option<PathBuf>,
  /// Override `ssl_cert_file`
  pub periphery_ssl_cert_file: Option<PathBuf>,
}

/// # Periphery Configuration File
///
/// Refer to the [example file](https://github.com/moghtech/komodo/blob/main/config/periphery.config.toml) for a full example.
#[derive(Debug, Clone, Deserialize)]
pub struct PeripheryConfig {
  /// The port periphery will run on.
  /// Default: `8120`
  #[serde(default = "default_periphery_port")]
  pub port: u16,

  /// IP address the periphery server binds to.
  /// Default: [::].
  #[serde(default = "default_periphery_bind_ip")]
  pub bind_ip: String,

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
  pub repo_dir: Option<PathBuf>,

  /// The system directory where stacks will managed.
  /// If not provided, will default to `${root_directory}/stacks`.
  /// Default: empty
  pub stack_dir: Option<PathBuf>,

  /// The system directory where builds will managed.
  /// If not provided, will default to `${root_directory}/builds`.
  /// Default: empty
  pub build_dir: Option<PathBuf>,

  /// Whether to disable the terminal APIs
  /// and disallow remote shell access.
  /// Default: false
  #[serde(default)]
  pub disable_terminals: bool,

  /// The rate at which the system stats will be polled to update the cache.
  /// Default: `5-sec`
  #[serde(default = "default_stats_polling_rate")]
  pub stats_polling_rate: Timelength,

  /// Whether stack actions should use `docker-compose ...`
  /// instead of `docker compose ...`.
  /// Default: false
  #[serde(default)]
  pub legacy_compose_cli: bool,

  /// Logging configuration
  #[serde(default)]
  pub logging: LogConfig,

  /// Limits which IPv4 addresses are allowed to call the api.
  /// Default: none
  ///
  /// Note: this should be configured to increase security.
  #[serde(default)]
  pub allowed_ips: Vec<IpAddr>,

  /// Limits the accepted passkeys.
  /// Default: none
  ///
  /// Note: this should be configured to increase security.
  #[serde(default)]
  pub passkeys: Vec<String>,

  /// If non-empty, only includes specific mount paths in the disk report.
  #[serde(default)]
  pub include_disk_mounts: Vec<PathBuf>,

  /// Exclude specific mount paths in the disk report.
  #[serde(default)]
  pub exclude_disk_mounts: Vec<PathBuf>,

  /// Mapping on local periphery secrets. These can be interpolated into eg. Deployment environment variables.
  /// Default: none
  #[serde(default)]
  pub secrets: HashMap<String, String>,

  /// Configure git credentials used to clone private repos.
  /// Supports any git provider.
  #[serde(default, alias = "git_provider")]
  pub git_providers: Vec<GitProvider>,

  /// Configure docker credentials used to push / pull images.
  /// Supports any docker image repository.
  #[serde(default, alias = "docker_registry")]
  pub docker_registries: Vec<DockerRegistry>,

  /// Whether to enable ssl.
  /// Default: true
  #[serde(default = "default_ssl_enabled")]
  pub ssl_enabled: bool,

  /// Path to the ssl key.
  /// Default: `${root_directory}/ssl/key.pem`.
  pub ssl_key_file: Option<PathBuf>,

  /// Path to the ssl cert.
  /// Default: `${root_directory}/ssl/cert.pem`.
  pub ssl_cert_file: Option<PathBuf>,
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

fn default_stats_polling_rate() -> Timelength {
  Timelength::FiveSeconds
}

fn default_ssl_enabled() -> bool {
  true
}

impl Default for PeripheryConfig {
  fn default() -> Self {
    Self {
      port: default_periphery_port(),
      bind_ip: default_periphery_bind_ip(),
      root_directory: default_root_directory(),
      repo_dir: None,
      stack_dir: None,
      build_dir: None,
      disable_terminals: Default::default(),
      stats_polling_rate: default_stats_polling_rate(),
      legacy_compose_cli: Default::default(),
      logging: Default::default(),
      allowed_ips: Default::default(),
      passkeys: Default::default(),
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
      port: self.port,
      bind_ip: self.bind_ip.clone(),
      root_directory: self.root_directory.clone(),
      repo_dir: self.repo_dir.clone(),
      stack_dir: self.stack_dir.clone(),
      build_dir: self.build_dir.clone(),
      disable_terminals: self.disable_terminals,
      stats_polling_rate: self.stats_polling_rate,
      legacy_compose_cli: self.legacy_compose_cli,
      logging: self.logging.clone(),
      allowed_ips: self.allowed_ips.clone(),
      passkeys: self
        .passkeys
        .iter()
        .map(|passkey| empty_or_redacted(passkey))
        .collect(),
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
      dir.to_owned()
    } else {
      self.root_directory.join("ssl/key.pem")
    }
  }

  pub fn ssl_cert_file(&self) -> PathBuf {
    if let Some(dir) = &self.ssl_cert_file {
      dir.to_owned()
    } else {
      self.root_directory.join("ssl/cert.pem")
    }
  }
}
