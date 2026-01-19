#[macro_use]
extern crate tracing;

use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;
use axum::{routing::post, Json, Router};
use komodo_client::entities::alert::{Alert, SeverityLevel};
use serde::Deserialize;

/// Entrypoint for handling each incoming alert.
async fn handle_incoming_alert(Json(alert): Json<Alert>) {
  if alert.resolved {
    info!("Alert Resolved!: {alert:?}");
    return;
  }
  match alert.level {
    SeverityLevel::Ok => info!("{alert:?}"),
    SeverityLevel::Warning => warn!("{alert:?}"),
    SeverityLevel::Critical => error!("{alert:?}"),
  }
}

/// ========================
/// Http server boilerplate.
/// ========================

#[derive(Deserialize)]
struct Env {
  #[serde(default = "default_port")]
  port: u16,
}

fn default_port() -> u16 {
  7000
}

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  mogh_logger::init(&Default::default())?;

  let Env { port } =
    envy::from_env().context("failed to parse env")?;

  let socket_addr = SocketAddr::from_str(&format!("0.0.0.0:{port}"))
    .context("invalid socket addr")?;

  info!("v {} | {socket_addr}", env!("CARGO_PKG_VERSION"));

  let app = Router::new().route("/", post(handle_incoming_alert));

  let listener = tokio::net::TcpListener::bind(socket_addr)
    .await
    .context("failed to bind tcp listener")?;

  axum::serve(listener, app).await.context("server crashed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut term_signal = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::terminate(),
  )?;

  let app = tokio::spawn(app());

  tokio::select! {
    res = app => return res?,
    _ = term_signal.recv() => {},
  }

  Ok(())
}
