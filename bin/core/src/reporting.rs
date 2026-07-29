use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use async_timing_util::{Timelength, wait_until_timelength};
use komodo_client::entities::{
  ResourceTargetVariant, core_report::CoreReport,
};
use mogh_pki::{
  PkiKind, RotatableKeyPair, SpkiPublicKey,
  one_way::OneWayNoiseHandshake,
};

use crate::{config::core_config, state::db_client};

pub fn spawn_reporting_loop() {
  let config = core_config();

  if !config.reporting_enabled {
    return;
  }

  let keys = match RotatableKeyPair::from_private_key_spec(
    PkiKind::OneWay,
    &config.reporting_private_key,
  ) {
    Ok(keys) => keys,
    Err(e) => {
      warn!(
        "Failed to initialize reporting key pair. Reporting is disabled. {e:#}"
      );
      return;
    }
  };

  let client = reqwest::Client::default();

  tokio::spawn(async move {
    loop {
      wait_until_timelength(Timelength::OneDay, 1100).await;
      if let Err(e) = report(&client, &keys).await {
        warn!("Reporting failed. {e:#}");
      }
    }
  });
}

async fn report(
  client: &reqwest::Client,
  keys: &RotatableKeyPair,
) -> anyhow::Result<()> {
  let db = db_client();

  let keys = keys.load();
  let private_key_bytes = keys
    .private
    .as_raw_bytes()
    .context("Invalid reporting private key.")?;

  let endpoint_public_key = client
    .get("https://mogh.tech/report/public_key")
    .send()
    .await
    .context("Failed to query for reporting endpoint public key for signature.")?
    .error_for_status()
    .context("Failed response for reporting endpoint public key for signature.")?
    .text()
    .await
    .context("Failed to get reporting endpoint public key for signature.")?;

  let endpoint_public_key =
    SpkiPublicKey::maybe_pem_to_raw_bytes(&endpoint_public_key)
      .context("Invalid reporting endpoint public key.")?;

  let users = db
    .users
    .count_documents(Default::default())
    .await
    .context("Failed to query database for users")?;
  let servers = db
    .servers
    .count_documents(Default::default())
    .await
    .context("Failed to query database for servers")?;
  let swarms = db
    .swarms
    .count_documents(Default::default())
    .await
    .context("Failed to query database for swarms")?;
  let stacks = db
    .stacks
    .count_documents(Default::default())
    .await
    .context("Failed to query database for stacks")?;
  let deployments = db
    .deployments
    .count_documents(Default::default())
    .await
    .context("Failed to query database for deployments")?;
  let builds = db
    .builds
    .count_documents(Default::default())
    .await
    .context("Failed to query database for builds")?;

  let count = [
    (ResourceTargetVariant::Server, servers),
    (ResourceTargetVariant::Swarm, swarms),
    (ResourceTargetVariant::Stack, stacks),
    (ResourceTargetVariant::Deployment, deployments),
    (ResourceTargetVariant::Build, builds),
  ]
  .into_iter()
  .collect();

  let report = CoreReport {
    report_public_key: keys.public().to_string(),
    users,
    count,
  };

  let serialized = serde_json::to_string(&report)
    .context("Failed to serialize report JSON")?;

  let timestamp =
    SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

  let prologue =
    format!("POST|/report/komodo|{serialized}|{timestamp}");

  let mut handshake = OneWayNoiseHandshake::new_initiator(
    &private_key_bytes,
    &endpoint_public_key,
    prologue.as_bytes(),
  )?;

  let signature = handshake
    .generate_signature()
    .context("Failed to generate report signature")?;

  client
    .post("https://mogh.tech/report/komodo")
    .header("x-api-signature", signature)
    .header("x-api-timestamp", timestamp)
    .header("content-type", "application/json")
    .body(serialized)
    .send()
    .await
    .context("Failed to post report.")?
    .error_for_status()
    .context("Failed response for report post.")?;

  Ok(())
}
