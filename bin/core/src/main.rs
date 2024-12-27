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
mod resource;
mod stack;
mod state;
mod sync;
mod ts_client;
mod ws;

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  let config = core_config();
  logger::init(&config.logging)?;

  info!("Komodo Core version: v{}", env!("CARGO_PKG_VERSION"));
  info!("{:?}", config.sanitized());

  tokio::join!(
    // Init db_client check to crash on db init failure
    state::init_db_client(),
    // Init default OIDC client (defined in config / env vars / compose secret file)
    auth::oidc::client::init_default_oidc_client()
  );
  tokio::join!(
    // Maybe initialize first server
    helpers::ensure_first_server_and_builder(),
    // Cleanup open updates / invalid alerts
    helpers::startup_cleanup(),
  );
  // init jwt client to crash on failure
  state::jwt_client();

  // Spawn tasks
  monitor::spawn_monitor_loop();
  resource::spawn_resource_refresh_loop();
  resource::spawn_build_state_refresh_loop();
  resource::spawn_repo_state_refresh_loop();
  resource::spawn_procedure_state_refresh_loop();
  resource::spawn_action_state_refresh_loop();
  resource::spawn_resource_sync_state_refresh_loop();
  helpers::prune::spawn_prune_loop();

  // Setup static frontend services
  let frontend_path = &config.frontend_path;
  let frontend_index =
    ServeFile::new(format!("{frontend_path}/index.html"));
  let serve_dir = ServeDir::new(frontend_path)
    .not_found_service(frontend_index.clone());

  let app = Router::new()
    .nest("/auth", api::auth::router())
    .nest("/user", api::user::router())
    .nest("/read", api::read::router())
    .nest("/write", api::write::router())
    .nest("/execute", api::execute::router())
    .nest("/listener", listener::router())
    .nest("/ws", ws::router())
    .nest("/client", ts_client::router())
    .nest_service("/", serve_dir)
    .fallback_service(frontend_index)
    .layer(cors()?)
    .into_make_service();

  let socket_addr =
    SocketAddr::from_str(&format!("0.0.0.0:{}", core_config().port))
      .context("failed to parse socket addr")?;

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

fn cors() -> anyhow::Result<CorsLayer> {
  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
  Ok(cors)
}
