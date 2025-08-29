use std::{path::PathBuf, sync::OnceLock};

use anyhow::Context;
use colored::Colorize;
use config::ConfigLoader;
use environment_file::{
  maybe_read_item_from_file, maybe_read_list_from_file,
};
use komodo_client::entities::{
  config::{
    DatabaseConfig,
    core::{
      AwsCredentials, CoreConfig, Env, GithubWebhookAppConfig,
      GithubWebhookAppInstallationConfig, OauthCredentials,
    },
  },
  logger::LogConfig,
};

pub fn core_config() -> &'static CoreConfig {
  static CORE_CONFIG: OnceLock<CoreConfig> = OnceLock::new();
  CORE_CONFIG.get_or_init(|| {
    let env: Env = match envy::from_env()
      .context("Failed to parse Komodo Core environment") {
        Ok(env) => env,
        Err(e) => {
          panic!("{e:?}");
        }
      };
    let config = if env.komodo_config_paths.is_empty() {
      println!(
        "{}: No config paths found, using default config",
        "INFO".green(),
      );
      CoreConfig::default()
    } else {
      let config_keywords = env.komodo_config_keywords
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
      println!(
        "{}: {}: {config_keywords:?}",
        "INFO".green(),
        "Config File Keywords".dimmed(),
      );
      (ConfigLoader {
        paths: &env.komodo_config_paths
          .iter()
          .map(PathBuf::as_path)
          .collect::<Vec<_>>(),
        match_wildcards: &config_keywords,
        include_file_name: ".kcoreinclude",
        merge_nested: env.komodo_merge_nested_config,
        extend_array: env.komodo_extend_config_arrays,
        debug_print: env.komodo_config_debug,
      }).load::<CoreConfig>()
      .expect("Failed at parsing config from paths")
    };

    let installations = match (
      maybe_read_list_from_file(
        env.komodo_github_webhook_app_installations_ids_file,
        env.komodo_github_webhook_app_installations_ids
      ),
      env.komodo_github_webhook_app_installations_namespaces
    ) {
      (Some(ids), Some(namespaces)) => {
        if ids.len() != namespaces.len() {
          panic!("KOMODO_GITHUB_WEBHOOK_APP_INSTALLATIONS_IDS length and KOMODO_GITHUB_WEBHOOK_APP_INSTALLATIONS_NAMESPACES length mismatch. Got {ids:?} and {namespaces:?}")
        }
        ids
          .into_iter()
          .zip(namespaces)
          .map(|(id, namespace)| GithubWebhookAppInstallationConfig {
            id,
            namespace
          })
          .collect()
      },
      (Some(_), None) | (None, Some(_)) => {
        panic!("Got only one of KOMODO_GITHUB_WEBHOOK_APP_INSTALLATIONS_IDS or KOMODO_GITHUB_WEBHOOK_APP_INSTALLATIONS_NAMESPACES, both MUST be provided");
      }
      (None, None) => {
        config.github_webhook_app.installations
      }
    };

    // recreating CoreConfig here makes sure apply all env overrides applied.
    CoreConfig {
      // Secret things overridden with file
      jwt_secret: maybe_read_item_from_file(env.komodo_jwt_secret_file, env.komodo_jwt_secret).unwrap_or(config.jwt_secret),
      passkey: maybe_read_item_from_file(env.komodo_passkey_file, env.komodo_passkey)
        .unwrap_or(config.passkey),
      webhook_secret: maybe_read_item_from_file(env.komodo_webhook_secret_file, env.komodo_webhook_secret)
        .unwrap_or(config.webhook_secret),
      database: DatabaseConfig {
        uri: maybe_read_item_from_file(env.komodo_database_uri_file,env.komodo_database_uri).unwrap_or(config.database.uri),
        address: env.komodo_database_address.unwrap_or(config.database.address),
        username: maybe_read_item_from_file(env.komodo_database_username_file,env
          .komodo_database_username)
          .unwrap_or(config.database.username),
        password: maybe_read_item_from_file(env.komodo_database_password_file,env
          .komodo_database_password)
          .unwrap_or(config.database.password),
        app_name: env
          .komodo_database_app_name
          .unwrap_or(config.database.app_name),
        db_name: env
          .komodo_database_db_name
          .unwrap_or(config.database.db_name),
      },
      init_admin_username: maybe_read_item_from_file(
        env.komodo_init_admin_username_file,
        env.komodo_init_admin_username
      ).or(config.init_admin_username),
      init_admin_password: maybe_read_item_from_file(
        env.komodo_init_admin_password_file,
        env.komodo_init_admin_password
      ).unwrap_or(config.init_admin_password),
      oidc_enabled: env.komodo_oidc_enabled.unwrap_or(config.oidc_enabled),
      oidc_provider: env.komodo_oidc_provider.unwrap_or(config.oidc_provider),
      oidc_redirect_host: env.komodo_oidc_redirect_host.unwrap_or(config.oidc_redirect_host),
      oidc_client_id: maybe_read_item_from_file(env.komodo_oidc_client_id_file,env
        .komodo_oidc_client_id)
        .unwrap_or(config.oidc_client_id),
      oidc_client_secret: maybe_read_item_from_file(env.komodo_oidc_client_secret_file,env
        .komodo_oidc_client_secret)
        .unwrap_or(config.oidc_client_secret),
      oidc_use_full_email: env.komodo_oidc_use_full_email
        .unwrap_or(config.oidc_use_full_email),
      oidc_additional_audiences: maybe_read_list_from_file(env.komodo_oidc_additional_audiences_file,env
        .komodo_oidc_additional_audiences)
        .unwrap_or(config.oidc_additional_audiences),
      google_oauth: OauthCredentials {
        enabled: env
          .komodo_google_oauth_enabled
          .unwrap_or(config.google_oauth.enabled),
        id: maybe_read_item_from_file(env.komodo_google_oauth_id_file,env
          .komodo_google_oauth_id)
          .unwrap_or(config.google_oauth.id),
        secret: maybe_read_item_from_file(env.komodo_google_oauth_secret_file,env
          .komodo_google_oauth_secret)
          .unwrap_or(config.google_oauth.secret),
      },
      github_oauth: OauthCredentials {
        enabled: env
          .komodo_github_oauth_enabled
          .unwrap_or(config.github_oauth.enabled),
        id: maybe_read_item_from_file(env.komodo_github_oauth_id_file,env
          .komodo_github_oauth_id)
          .unwrap_or(config.github_oauth.id),
        secret: maybe_read_item_from_file(env.komodo_github_oauth_secret_file,env
          .komodo_github_oauth_secret)
          .unwrap_or(config.github_oauth.secret),
      },
      aws: AwsCredentials {
        access_key_id: maybe_read_item_from_file(env.komodo_aws_access_key_id_file, env
          .komodo_aws_access_key_id)
          .unwrap_or(config.aws.access_key_id),
        secret_access_key: maybe_read_item_from_file(env.komodo_aws_secret_access_key_file, env
          .komodo_aws_secret_access_key)
          .unwrap_or(config.aws.secret_access_key),
      },
      github_webhook_app: GithubWebhookAppConfig {
        app_id: maybe_read_item_from_file(env.komodo_github_webhook_app_app_id_file, env
          .komodo_github_webhook_app_app_id)
          .unwrap_or(config.github_webhook_app.app_id),
        pk_path: env
          .komodo_github_webhook_app_pk_path
          .unwrap_or(config.github_webhook_app.pk_path),
        installations,
      },

      // Non secrets
      title: env.komodo_title.unwrap_or(config.title),
      host: env.komodo_host.unwrap_or(config.host),
      port: env.komodo_port.unwrap_or(config.port),
      bind_ip: env.komodo_bind_ip.unwrap_or(config.bind_ip),
      timezone: env.komodo_timezone.unwrap_or(config.timezone),
      first_server: env.komodo_first_server.or(config.first_server),
      first_server_name: env.komodo_first_server_name.unwrap_or(config.first_server_name),
      frontend_path: env.komodo_frontend_path.unwrap_or(config.frontend_path),
      jwt_ttl: env
        .komodo_jwt_ttl
        .unwrap_or(config.jwt_ttl),
      sync_directory: env
        .komodo_sync_directory
        .unwrap_or(config.sync_directory),
      repo_directory: env
        .komodo_repo_directory
        .unwrap_or(config.repo_directory),
      action_directory: env
        .komodo_action_directory
        .unwrap_or(config.action_directory),
      resource_poll_interval: env
        .komodo_resource_poll_interval
        .unwrap_or(config.resource_poll_interval),
      monitoring_interval: env
        .komodo_monitoring_interval
        .unwrap_or(config.monitoring_interval),
      keep_stats_for_days: env
        .komodo_keep_stats_for_days
        .unwrap_or(config.keep_stats_for_days),
      keep_alerts_for_days: env
        .komodo_keep_alerts_for_days
        .unwrap_or(config.keep_alerts_for_days),
      webhook_base_url: env
        .komodo_webhook_base_url
        .unwrap_or(config.webhook_base_url),
      transparent_mode: env
        .komodo_transparent_mode
        .unwrap_or(config.transparent_mode),
      ui_write_disabled: env
        .komodo_ui_write_disabled
        .unwrap_or(config.ui_write_disabled),
      disable_confirm_dialog: env.komodo_disable_confirm_dialog
        .unwrap_or(config.disable_confirm_dialog),
      disable_websocket_reconnect: env.komodo_disable_websocket_reconnect
        .unwrap_or(config.disable_websocket_reconnect),
      enable_new_users: env.komodo_enable_new_users
        .unwrap_or(config.enable_new_users),
      disable_user_registration: env.komodo_disable_user_registration
        .unwrap_or(config.disable_user_registration),
      disable_non_admin_create: env.komodo_disable_non_admin_create
        .unwrap_or(config.disable_non_admin_create),
      disable_init_resources: env.komodo_disable_init_resources
        .unwrap_or(config.disable_init_resources),
      enable_fancy_toml: env.komodo_enable_fancy_toml
        .unwrap_or(config.enable_fancy_toml),
      lock_login_credentials_for: env.komodo_lock_login_credentials_for
        .unwrap_or(config.lock_login_credentials_for),
      local_auth: env.komodo_local_auth
        .unwrap_or(config.local_auth),
      logging: LogConfig {
        level: env
          .komodo_logging_level
          .unwrap_or(config.logging.level),
        stdio: env
          .komodo_logging_stdio
          .unwrap_or(config.logging.stdio),
        pretty: env.komodo_logging_pretty
          .unwrap_or(config.logging.pretty),
        location: env.komodo_logging_location
          .unwrap_or(config.logging.location),
        otlp_endpoint: env
          .komodo_logging_otlp_endpoint
          .unwrap_or(config.logging.otlp_endpoint),
        opentelemetry_service_name: env
          .komodo_logging_opentelemetry_service_name
          .unwrap_or(config.logging.opentelemetry_service_name),
      },
      pretty_startup_config: env.komodo_pretty_startup_config.unwrap_or(config.pretty_startup_config),
      unsafe_unsanitized_startup_config: env.komodo_unsafe_unsanitized_startup_config.unwrap_or(config.unsafe_unsanitized_startup_config),
      internet_interface: env.komodo_internet_interface.unwrap_or(config.internet_interface),
      ssl_enabled: env.komodo_ssl_enabled.unwrap_or(config.ssl_enabled),
      ssl_key_file: env.komodo_ssl_key_file.unwrap_or(config.ssl_key_file),
      ssl_cert_file: env.komodo_ssl_cert_file.unwrap_or(config.ssl_cert_file),

      // These can't be overridden on env
      secrets: config.secrets,
      git_providers: config.git_providers,
      docker_registries: config.docker_registries,
    }
  })
}
