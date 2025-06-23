#[macro_use]
extern crate tracing;

//
use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;
use axum_server::tls_rustls::RustlsConfig;
use config::periphery_config;

mod api;
mod build;
mod compose;
mod config;
mod docker;
mod git;
mod helpers;
mod ssl;
mod stats;
mod terminal;

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  let config = config::periphery_config();
  logger::init(&config.logging)?;

  info!("Komodo Periphery version: v{}", env!("CARGO_PKG_VERSION"));

  if periphery_config().pretty_startup_config {
    info!("{:#?}", config.sanitized());
  } else {
    info!("{:?}", config.sanitized());
  }

  stats::spawn_polling_thread();
  docker::stats::spawn_polling_thread();

  let addr = format!(
    "{}:{}",
    config::periphery_config().bind_ip,
    config::periphery_config().port
  );

  let socket_addr = SocketAddr::from_str(&addr)
    .context("failed to parse listen address")?;

  let app =
    api::router().into_make_service_with_connect_info::<SocketAddr>();

  if config.ssl_enabled {
    info!("🔒 Periphery SSL Enabled");
    rustls::crypto::ring::default_provider()
      .install_default()
      .expect("failed to install default rustls CryptoProvider");
    ssl::ensure_certs().await;
    info!("Komodo Periphery starting on https://{}", socket_addr);
    let ssl_config = RustlsConfig::from_pem_file(
      config.ssl_cert_file(),
      config.ssl_key_file(),
    )
    .await
    .context("Invalid ssl cert / key")?;
    axum_server::bind_rustls(socket_addr, ssl_config)
      .serve(app)
      .await?
  } else {
    info!("🔓 Periphery SSL Disabled");
    info!("Komodo Periphery starting on http://{}", socket_addr);
    axum_server::bind(socket_addr).serve(app).await?
  }

  Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut term_signal = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::terminate(),
  )?;

  let app = tokio::spawn(app());

  tokio::select! {
    res = app => return res?,
    _ = term_signal.recv() => {
      info!("Exiting all active Terminals for shutdown");
      terminal::delete_all_terminals().await;
    },
  }

  Ok(())
}
