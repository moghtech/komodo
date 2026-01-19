use std::{path::PathBuf, sync::OnceLock};

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use komodo_client::entities::{
  config::{
    DatabaseConfig,
    cli::{
      CliConfig, Env,
      args::{CliArgs, Command, Execute, database::DatabaseCommand},
    },
  },
  logger::LogConfig,
};
use secret_file::maybe_read_item_from_file;

pub fn cli_args() -> &'static CliArgs {
  static CLI_ARGS: OnceLock<CliArgs> = OnceLock::new();
  CLI_ARGS.get_or_init(CliArgs::parse)
}

pub fn cli_env() -> &'static Env {
  static CLI_ARGS: OnceLock<Env> = OnceLock::new();
  CLI_ARGS.get_or_init(|| {
    match envy::from_env()
      .context("Failed to parse Komodo CLI environment")
    {
      Ok(env) => env,
      Err(e) => {
        panic!("{e:?}")
      }
    }
  })
}

pub fn cli_config() -> &'static CliConfig {
  static CLI_CONFIG: OnceLock<CliConfig> = OnceLock::new();
  CLI_CONFIG.get_or_init(|| {
    let args = cli_args();
    let env = cli_env().clone();
    let config_paths = args
      .config_path
      .clone()
      .unwrap_or(env.komodo_cli_config_paths);
    let debug_startup =
      args.debug_startup.unwrap_or(env.komodo_cli_debug_startup);

    if debug_startup {
      println!(
        "{}: Komodo CLI version: {}",
        "DEBUG".cyan(),
        env!("CARGO_PKG_VERSION").blue().bold()
      );
      println!(
        "{}: {}: {config_paths:?}",
        "DEBUG".cyan(),
        "Config Paths".dimmed(),
      );
    }

    let config_keywords = args
      .config_keyword
      .clone()
      .unwrap_or(env.komodo_cli_config_keywords);
    let config_keywords = config_keywords
      .iter()
      .map(String::as_str)
      .collect::<Vec<_>>();
    if debug_startup {
      println!(
        "{}: {}: {config_keywords:?}",
        "DEBUG".cyan(),
        "Config File Keywords".dimmed(),
      );
    }
    let mut unparsed_config = (config::ConfigLoader {
      paths: &config_paths
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>(),
      match_wildcards: &config_keywords,
      include_file_name: ".kminclude",
      merge_nested: env.komodo_cli_merge_nested_config,
      extend_array: env.komodo_cli_extend_config_arrays,
      debug_print: debug_startup,
    })
    .load::<serde_json::Map<String, serde_json::Value>>()
    .expect("failed at parsing config from paths");
    let init_parsed_config = serde_json::from_value::<CliConfig>(
      serde_json::Value::Object(unparsed_config.clone()),
    )
    .context("Failed to parse config")
    .unwrap();

    let (host, key, secret) = match &args.command {
      Command::Execute(Execute {
        host, key, secret, ..
      }) => (host.clone(), key.clone(), secret.clone()),
      _ => (None, None, None),
    };

    let backups_folder = match &args.command {
      Command::Database {
        command: DatabaseCommand::Backup { backups_folder, .. },
      } => backups_folder.clone(),
      Command::Database {
        command: DatabaseCommand::Restore { backups_folder, .. },
      } => backups_folder.clone(),
      _ => None,
    };
    let (uri, address, username, password, db_name) =
      match &args.command {
        Command::Database {
          command:
            DatabaseCommand::Copy {
              uri,
              address,
              username,
              password,
              db_name,
              ..
            },
        } => (
          uri.clone(),
          address.clone(),
          username.clone(),
          password.clone(),
          db_name.clone(),
        ),
        _ => (None, None, None, None, None),
      };

    let profile = args
      .profile
      .as_ref()
      .or(init_parsed_config.default_profile.as_ref());

    let unparsed_config = if let Some(profile) = profile
      && !profile.is_empty()
    {
      // Find the profile config,
      // then merge it with the Default config.
      let serde_json::Value::Array(profiles) = unparsed_config
        .remove("profile")
        .context("Config has no profiles, but a profile is required")
        .unwrap()
      else {
        panic!("`config.profile` is not array");
      };
      let Some(profile_config) = profiles.into_iter().find(|p| {
        let Ok(parsed) =
          serde_json::from_value::<CliConfig>(p.clone())
        else {
          return false;
        };
        &parsed.config_profile == profile
          || parsed
            .config_aliases
            .iter()
            .any(|alias| alias == profile)
      }) else {
        panic!("No profile matching '{profile}' was found.");
      };
      let serde_json::Value::Object(profile_config) = profile_config
      else {
        panic!("Profile config is not Object type.");
      };
      config::merge_config(
        unparsed_config,
        profile_config.clone(),
        env.komodo_cli_merge_nested_config,
        env.komodo_cli_extend_config_arrays,
      )
      .unwrap_or(profile_config)
    } else {
      unparsed_config
    };
    let config = serde_json::from_value::<CliConfig>(
      serde_json::Value::Object(unparsed_config),
    )
    .context("Failed to parse final config")
    .unwrap();
    let config_profile = if config.config_profile.is_empty() {
      String::from("None")
    } else {
      config.config_profile
    };

    CliConfig {
      config_profile,
      config_aliases: config.config_aliases,
      default_profile: config.default_profile,
      table_borders: env
        .komodo_cli_table_borders
        .or(config.table_borders),
      host: host
        .or(env.komodo_cli_host)
        .or(env.komodo_host)
        .unwrap_or(config.host),
      cli_key: key.or(env.komodo_cli_key).or(config.cli_key),
      cli_secret: secret
        .or(env.komodo_cli_secret)
        .or(config.cli_secret),
      backups_folder: backups_folder
        .or(env.komodo_cli_backups_folder)
        .unwrap_or(config.backups_folder),
      max_backups: env
        .komodo_cli_max_backups
        .unwrap_or(config.max_backups),
      database_target: DatabaseConfig {
        uri: uri
          .or(env.komodo_cli_database_target_uri)
          .unwrap_or(config.database_target.uri),
        address: address
          .or(env.komodo_cli_database_target_address)
          .unwrap_or(config.database_target.address),
        username: username
          .or(env.komodo_cli_database_target_username)
          .unwrap_or(config.database_target.username),
        password: password
          .or(env.komodo_cli_database_target_password)
          .unwrap_or(config.database_target.password),
        db_name: db_name
          .or(env.komodo_cli_database_target_db_name)
          .unwrap_or(config.database_target.db_name),
        app_name: config.database_target.app_name,
      },
      database: DatabaseConfig {
        uri: maybe_read_item_from_file(
          env.komodo_database_uri_file,
          env.komodo_database_uri,
        )
        .unwrap_or(config.database.uri),
        address: env
          .komodo_database_address
          .unwrap_or(config.database.address),
        username: maybe_read_item_from_file(
          env.komodo_database_username_file,
          env.komodo_database_username,
        )
        .unwrap_or(config.database.username),
        password: maybe_read_item_from_file(
          env.komodo_database_password_file,
          env.komodo_database_password,
        )
        .unwrap_or(config.database.password),
        db_name: env
          .komodo_database_db_name
          .unwrap_or(config.database.db_name),
        app_name: config.database.app_name,
      },
      cli_logging: LogConfig {
        level: env
          .komodo_cli_logging_level
          .unwrap_or(config.cli_logging.level),
        stdio: env
          .komodo_cli_logging_stdio
          .unwrap_or(config.cli_logging.stdio),
        pretty: env
          .komodo_cli_logging_pretty
          .unwrap_or(config.cli_logging.pretty),
        location: false,
        ansi: env
          .komodo_cli_logging_ansi
          .unwrap_or(config.cli_logging.ansi),
        otlp_endpoint: env
          .komodo_cli_logging_otlp_endpoint
          .unwrap_or(config.cli_logging.otlp_endpoint),
        opentelemetry_service_name: env
          .komodo_cli_logging_opentelemetry_service_name
          .unwrap_or(config.cli_logging.opentelemetry_service_name),
        opentelemetry_scope_name: env
          .komodo_cli_logging_opentelemetry_scope_name
          .unwrap_or(config.cli_logging.opentelemetry_scope_name),
      },
      profile: config.profile,
    }
  })
}
