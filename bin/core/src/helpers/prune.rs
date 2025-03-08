use anyhow::Context;
use async_timing_util::{
  ONE_DAY_MS, Timelength, unix_timestamp_ms, wait_until_timelength,
};
use futures::future::join_all;
use mungos::{find::find_collect, mongodb::bson::doc};
use periphery_client::api::image::PruneImages;

use crate::{config::core_config, state::db_client};

use super::periphery_client;

pub fn spawn_prune_loop() {
  tokio::spawn(async move {
    loop {
      wait_until_timelength(Timelength::OneDay, 5000).await;
      let (images_res, stats_res, alerts_res) =
        tokio::join!(prune_images(), prune_stats(), prune_alerts());
      if let Err(e) = images_res {
        error!("error in pruning images | {e:#}");
      }
      if let Err(e) = stats_res {
        error!("error in pruning stats | {e:#}");
      }
      if let Err(e) = alerts_res {
        error!("error in pruning alerts | {e:#}");
      }
    }
  });
}

async fn prune_images() -> anyhow::Result<()> {
  let futures = find_collect(&db_client().servers, None, None)
    .await
    .context("failed to get servers from db")?
    .into_iter()
    .filter(|server| {
      server.config.enabled && server.config.auto_prune
    })
    .map(|server| async move {
      (
        async {
          periphery_client(&server)?.request(PruneImages {}).await
        }
        .await,
        server,
      )
    });

  for (res, server) in join_all(futures).await {
    if let Err(e) = res {
      error!(
        "failed to prune images on server {} ({}) | {e:#}",
        server.name, server.id
      )
    }
  }

  Ok(())
}

async fn prune_stats() -> anyhow::Result<()> {
  if core_config().keep_stats_for_days == 0 {
    return Ok(());
  }
  let delete_before_ts = (unix_timestamp_ms()
    - core_config().keep_stats_for_days as u128 * ONE_DAY_MS)
    as i64;
  let res = db_client()
    .stats
    .delete_many(doc! {
      "ts": { "$lt": delete_before_ts }
    })
    .await?;
  if res.deleted_count > 0 {
    info!("deleted {} stats from db", res.deleted_count);
  }
  Ok(())
}

async fn prune_alerts() -> anyhow::Result<()> {
  if core_config().keep_alerts_for_days == 0 {
    return Ok(());
  }
  let delete_before_ts = (unix_timestamp_ms()
    - core_config().keep_alerts_for_days as u128 * ONE_DAY_MS)
    as i64;
  let res = db_client()
    .alerts
    .delete_many(doc! {
      "ts": { "$lt": delete_before_ts }
    })
    .await?;
  if res.deleted_count > 0 {
    info!("deleted {} alerts from db", res.deleted_count);
  }
  Ok(())
}
