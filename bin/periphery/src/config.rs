use std::sync::OnceLock;

use clap::Parser;
use environment_file::maybe_read_list_from_file;
use komodo_client::entities::{
  config::periphery::{CliArgs, Env, PeripheryConfig},
  logger::{LogConfig, LogLevel},
};
use merge_config_files::parse_config_paths;

pub fn periphery_config() -> &'static PeripheryConfig {
  static PERIPHERY_CONFIG: OnceLock<PeripheryConfig> =
    OnceLock::new();
  PERIPHERY_CONFIG.get_or_init(|| {
    let env: Env = envy::from_env()
      .expect("failed to parse periphery environment");
    let args = CliArgs::parse();
    let config_paths =
      args.config_path.unwrap_or(env.periphery_config_paths);
    let config = if config_paths.is_empty() {
      PeripheryConfig::default()
    } else {
      parse_config_paths::<PeripheryConfig>(
        config_paths,
        args.config_keyword.unwrap_or(env.periphery_config_keywords),
        args
          .merge_nested_config
          .unwrap_or(env.periphery_merge_nested_config),
        args
          .extend_config_arrays
          .unwrap_or(env.periphery_extend_config_arrays),
      )
      .expect("failed at parsing config from paths")
    };

    PeripheryConfig {
      port: env.periphery_port.unwrap_or(config.port),
      bind_ip: env.periphery_bind_ip.unwrap_or(config.bind_ip),
      root_directory: env
        .periphery_root_directory
        .unwrap_or(config.root_directory),
      repo_dir: env.periphery_repo_dir.or(config.repo_dir),
      stack_dir: env.periphery_stack_dir.or(config.stack_dir),
      build_dir: env.periphery_build_dir.or(config.build_dir),
      stats_polling_rate: env
        .periphery_stats_polling_rate
        .unwrap_or(config.stats_polling_rate),
      legacy_compose_cli: env
        .periphery_legacy_compose_cli
        .unwrap_or(config.legacy_compose_cli),
      logging: LogConfig {
        level: args
          .log_level
          .map(LogLevel::from)
          .or(env.periphery_logging_level)
          .unwrap_or(config.logging.level),
        stdio: env
          .periphery_logging_stdio
          .unwrap_or(config.logging.stdio),
        otlp_endpoint: env
          .periphery_logging_otlp_endpoint
          .unwrap_or(config.logging.otlp_endpoint),
        opentelemetry_service_name: env
          .periphery_logging_opentelemetry_service_name
          .unwrap_or(config.logging.opentelemetry_service_name),
      },
      allowed_ips: env
        .periphery_allowed_ips
        .unwrap_or(config.allowed_ips),
      passkeys: maybe_read_list_from_file(
        env.periphery_passkeys_file,
        env.periphery_passkeys,
      )
      .unwrap_or(config.passkeys),
      include_disk_mounts: env
        .periphery_include_disk_mounts
        .unwrap_or(config.include_disk_mounts),
      exclude_disk_mounts: env
        .periphery_exclude_disk_mounts
        .unwrap_or(config.exclude_disk_mounts),
      ssl_enabled: env
        .periphery_ssl_enabled
        .unwrap_or(config.ssl_enabled),
      ssl_key_file: env
        .periphery_ssl_key_file
        .or(config.ssl_key_file),
      ssl_cert_file: env
        .periphery_ssl_cert_file
        .or(config.ssl_cert_file),
      secrets: config.secrets,
      git_providers: config.git_providers,
      docker_registries: config.docker_registries,
    }
  })
}
