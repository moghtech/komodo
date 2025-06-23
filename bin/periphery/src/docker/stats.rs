use std::{
  collections::HashMap,
  sync::{Arc, OnceLock},
};

use anyhow::{Context, anyhow};
use arc_swap::ArcSwap;
use async_timing_util::wait_until_timelength;
use komodo_client::entities::docker::container::ContainerStats;
use run_command::async_run_command;

use crate::config::periphery_config;

pub type ContainerStatsMap = HashMap<String, ContainerStats>;

pub fn container_stats() -> &'static ArcSwap<ContainerStatsMap> {
  static CONTAINER_STATS: OnceLock<ArcSwap<ContainerStatsMap>> =
    OnceLock::new();
  CONTAINER_STATS.get_or_init(Default::default)
}

pub fn spawn_polling_thread() {
  tokio::spawn(async move {
    let polling_rate = periphery_config()
      .container_stats_polling_rate
      .to_string()
      .parse()
      .expect("invalid stats polling rate");
    update_container_stats().await;
    loop {
      let _ts = wait_until_timelength(polling_rate, 200).await;
      update_container_stats().await;
    }
  });
}

async fn update_container_stats() {
  match get_container_stats(None).await {
    Ok(stats) => {
      container_stats().store(Arc::new(
        stats.into_iter().map(|s| (s.name.clone(), s)).collect(),
      ));
    }
    Err(e) => {
      error!("Failed to refresh container stats cache | {e:#}");
    }
  }
}

pub async fn get_container_stats(
  container_name: Option<String>,
) -> anyhow::Result<Vec<ContainerStats>> {
  let format = "--format \"{{ json . }}\"";
  let container_name = match container_name {
    Some(name) => format!(" {name}"),
    None => "".to_string(),
  };
  let command =
    format!("docker stats{container_name} --no-stream {format}");
  let output = async_run_command(&command).await;
  if output.success() {
    output
      .stdout
      .split('\n')
      .filter(|e| !e.is_empty())
      .map(|e| {
        let parsed = serde_json::from_str(e)
          .context(format!("failed at parsing entry {e}"))?;
        Ok(parsed)
      })
      .collect()
  } else {
    Err(anyhow!("{}", output.stderr.replace('\n', " | ")))
  }
}
