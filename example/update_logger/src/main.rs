#[macro_use]
extern crate tracing;

use komodo_client::{ws::UpdateWsMessage, KomodoClient};

/// Entrypoint for handling each incoming update.
async fn handle_incoming_update(update: UpdateWsMessage) {
  info!("{update:?}");
}

/// ========================
/// Ws Listener boilerplate.
/// ========================

async fn app() -> anyhow::Result<()> {
  logger::init(&Default::default())?;

  info!("v {}", env!("CARGO_PKG_VERSION"));

  let komodo =
    KomodoClient::new_from_env()?.with_healthcheck().await?;

  let (mut rx, _) = komodo.subscribe_to_updates()?;

  loop {
    let update = match rx.recv().await {
      Ok(msg) => msg,
      Err(e) => {
        error!("🚨 recv error | {e:?}");
        break;
      }
    };
    handle_incoming_update(update).await
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
    _ = term_signal.recv() => {},
  }

  Ok(())
}
