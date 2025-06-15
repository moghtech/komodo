#[macro_use]
extern crate tracing;

use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
  cors::{Any, CorsLayer},
  services::{ServeDir, ServeFile},
};

use crate::config::core_config;

mod alert;
mod api;
mod auth;
mod cloud;
mod config;
mod db;
mod helpers;
mod listener;
mod monitor;
mod permission;
mod resource;
mod schedule;
mod stack;
mod startup;
mod state;
mod sync;
mod ts_client;
mod ws;

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  let config = core_config();
  logger::init(&config.logging)?;
  if let Err(e) =
    rustls::crypto::aws_lc_rs::default_provider().install_default()
  {
    error!("Failed to install default crypto provider | {e:?}");
    std::process::exit(1);
  };

  info!("Komodo Core version: v{}", env!("CARGO_PKG_VERSION"));

  if core_config().pretty_startup_config {
    info!("{:#?}", config.sanitized());
  } else {
    info!("{:?}", config.sanitized());
  }

  // Init jwt client to crash on failure
  state::jwt_client();
  tokio::join!(
    // Init db_client check to crash on db init failure
    state::init_db_client(),
    // Manage OIDC client (defined in config / env vars / compose secret file)
    auth::oidc::client::spawn_oidc_client_management()
  );
  // Run after db connection.
  startup::on_startup().await;

  // Spawn background tasks
  monitor::spawn_monitor_loop();
  resource::spawn_resource_refresh_loop();
  resource::spawn_all_resources_refresh_loop();
  resource::spawn_build_state_refresh_loop();
  resource::spawn_repo_state_refresh_loop();
  resource::spawn_procedure_state_refresh_loop();
  resource::spawn_action_state_refresh_loop();
  schedule::spawn_schedule_executor();
  helpers::prune::spawn_prune_loop();

  // Setup static frontend services
  let frontend_path = &config.frontend_path;
  let frontend_index =
    ServeFile::new(format!("{frontend_path}/index.html"));
  let serve_frontend = ServeDir::new(frontend_path)
    .not_found_service(frontend_index.clone());

  let app = Router::new()
    .nest("/auth", api::auth::router())
    .nest("/user", api::user::router())
    .nest("/read", api::read::router())
    .nest("/write", api::write::router())
    .nest("/execute", api::execute::router())
    .nest("/terminal", api::terminal::router())
    .nest("/listener", listener::router())
    .nest("/ws", ws::router())
    .nest("/client", ts_client::router())
    .fallback_service(serve_frontend)
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any),
    )
    .into_make_service();

  let addr =
    format!("{}:{}", core_config().bind_ip, core_config().port);
  let socket_addr = SocketAddr::from_str(&addr)
    .context("failed to parse listen address")?;

  if config.ssl_enabled {
    info!("🔒 Core SSL Enabled");
    rustls::crypto::ring::default_provider()
      .install_default()
      .expect("failed to install default rustls CryptoProvider");
    info!("Komodo Core starting on https://{socket_addr}");
    let ssl_config = RustlsConfig::from_pem_file(
      &config.ssl_cert_file,
      &config.ssl_key_file,
    )
    .await
    .context("Invalid ssl cert / key")?;
    axum_server::bind_rustls(socket_addr, ssl_config)
      .serve(app)
      .await
      .context("failed to start https server")
  } else {
    info!("🔓 Core SSL Disabled");
    info!("Komodo Core starting on http://{socket_addr}");
    axum_server::bind(socket_addr)
      .serve(app)
      .await
      .context("failed to start http server")
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut term_signal = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::terminate(),
  )?;
  tokio::select! {
    res = tokio::spawn(app()) => res?,
    _ = term_signal.recv() => Ok(()),
  }
}
