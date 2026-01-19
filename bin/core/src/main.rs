#![recursion_limit = "256"]

#[macro_use]
extern crate tracing;

use mogh_server::axum_server::Handle;
use tracing::Instrument;

use crate::config::{core_config, core_keys};

mod alert;
mod api;
mod auth;
mod cloud;
mod config;
mod connection;
mod helpers;
mod monitor;
mod network;
mod periphery;
mod permission;
mod resource;
mod schedule;
mod stack;
mod startup;
mod state;
mod sync;
mod ts_client;

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  let config = core_config();
  mogh_logger::init(&config.logging)?;

  let startup_span = info_span!("CoreStartup");

  async {
    info!("Komodo Core version: v{}", env!("CARGO_PKG_VERSION"));

    match (
      config.pretty_startup_config,
      config.unsafe_unsanitized_startup_config,
    ) {
      (true, true) => info!("{:#?}", config),
      (true, false) => info!("{:#?}", config.sanitized()),
      (false, true) => info!("{:?}", config),
      (false, false) => info!("{:?}", config.sanitized()),
    }

    // Init + log public key. Will crash if invalid private key here.
    info!("Public Key: {}", core_keys().load().public);

    rustls::crypto::aws_lc_rs::default_provider()
      .install_default()
      .expect("Failed to install default crypto provider");

    // Init jwt provider to crash on failure
    let _ = &auth::JWT_PROVIDER;
    // Init db_client check to crash on db init failure
    state::init_db_client().await;
    // Run after db connection.
    startup::on_startup().await;

    // Spawn background tasks
    monitor::spawn_monitoring_loops();
    resource::spawn_resource_refresh_loop();
    resource::spawn_all_resources_cache_refresh_loop();
    resource::spawn_build_state_refresh_loop();
    resource::spawn_repo_state_refresh_loop();
    resource::spawn_procedure_state_refresh_loop();
    resource::spawn_action_state_refresh_loop();
    schedule::spawn_schedule_executor();
    helpers::prune::spawn_prune_loop();
  }
  .instrument(startup_span)
  .await;

  let handle = Handle::new();
  tokio::spawn({
    // Cannot run actions until the server is available.
    // We can use a handle for the server, and wait until
    // the handle is listening before running actions
    let handle = handle.clone();
    async move {
      handle.listening().await;
      startup::run_startup_actions().await;
    }
  });

  mogh_server::serve_app(api::app(), config, handle).await
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
