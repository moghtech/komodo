use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, anyhow};
use async_timing_util::wait_until_timelength;
use bollard::{models, query_parameters::StatsOptionsBuilder};
use command::run_standard_command;
use futures_util::StreamExt;
use komodo_client::entities::docker::{
  container::ContainerStats,
  stats::{
    ContainerBlkioStatEntry, ContainerBlkioStats, ContainerCpuStats,
    ContainerCpuUsage, ContainerMemoryStats, ContainerNetworkStats,
    ContainerPidsStats, ContainerStorageStats,
    ContainerThrottlingData, FullContainerStats,
  },
};

use crate::{
  config::periphery_config, docker::DockerClient,
  state::container_stats,
};

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
  let format = "--format '{\"BlockIO\":\"{{ .BlockIO }}\", \
                           \"CPUPerc\":\"{{ .CPUPerc }}\", \
                           \"ID\":\"{{ .ID }}\", \
                           \"MemPerc\":\"{{ .MemPerc }}\", \
                           \"MemUsage\":\"{{ .MemUsage }}\", \
                           \"Name\":\"{{ .Name }}\", \
                           \"NetIO\":\"{{ .NetIO }}\",\
                           \"PIDs\":\"{{ .PIDs }}\"}'";
  let container_name = match container_name {
    Some(name) => format!(" {name}"),
    None => "".to_string(),
  };
  let command =
    format!("docker stats{container_name} --no-stream {format}");
  let output = run_standard_command(&command, None).await;
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

impl DockerClient {
  /// Calls for stats once, similar to --no-stream on the cli
  pub async fn full_container_stats(
    &self,
    container_name: &str,
  ) -> anyhow::Result<FullContainerStats> {
    let mut res = self.docker.stats(
      container_name,
      StatsOptionsBuilder::new().stream(false).build().into(),
    );
    let stats = res
      .next()
      .await
      .with_context(|| format!("Unable to get container stats for {container_name} (got None)"))?
      .with_context(|| format!("Unable to get container stats for {container_name}"))?;
    Ok(FullContainerStats {
      name: stats.name.unwrap_or(container_name.to_string()),
      id: stats.id,
      read: stats.read,
      preread: stats.preread,
      pids_stats: stats.pids_stats.map(convert_pids_stats),
      blkio_stats: stats.blkio_stats.map(convert_blkio_stats),
      num_procs: stats.num_procs,
      storage_stats: stats.storage_stats.map(convert_storage_stats),
      cpu_stats: stats.cpu_stats.map(convert_cpu_stats),
      precpu_stats: stats.precpu_stats.map(convert_cpu_stats),
      memory_stats: stats.memory_stats.map(convert_memory_stats),
      networks: stats.networks.map(convert_network_stats),
    })
  }
}

fn convert_pids_stats(
  pids_stats: models::ContainerPidsStats,
) -> ContainerPidsStats {
  ContainerPidsStats {
    current: pids_stats.current,
    limit: pids_stats.limit,
  }
}

fn convert_blkio_stats(
  blkio_stats: models::ContainerBlkioStats,
) -> ContainerBlkioStats {
  ContainerBlkioStats {
    io_service_bytes_recursive: blkio_stats
      .io_service_bytes_recursive
      .map(convert_blkio_stat_entries),
    io_serviced_recursive: blkio_stats
      .io_serviced_recursive
      .map(convert_blkio_stat_entries),
    io_queue_recursive: blkio_stats
      .io_queue_recursive
      .map(convert_blkio_stat_entries),
    io_service_time_recursive: blkio_stats
      .io_service_time_recursive
      .map(convert_blkio_stat_entries),
    io_wait_time_recursive: blkio_stats
      .io_wait_time_recursive
      .map(convert_blkio_stat_entries),
    io_merged_recursive: blkio_stats
      .io_merged_recursive
      .map(convert_blkio_stat_entries),
    io_time_recursive: blkio_stats
      .io_time_recursive
      .map(convert_blkio_stat_entries),
    sectors_recursive: blkio_stats
      .sectors_recursive
      .map(convert_blkio_stat_entries),
  }
}

fn convert_blkio_stat_entries(
  blkio_stat_entries: Vec<models::ContainerBlkioStatEntry>,
) -> Vec<ContainerBlkioStatEntry> {
  blkio_stat_entries
    .into_iter()
    .map(|blkio_stat_entry| ContainerBlkioStatEntry {
      major: blkio_stat_entry.major,
      minor: blkio_stat_entry.minor,
      op: blkio_stat_entry.op,
      value: blkio_stat_entry.value,
    })
    .collect()
}

fn convert_storage_stats(
  storage_stats: models::ContainerStorageStats,
) -> ContainerStorageStats {
  ContainerStorageStats {
    read_count_normalized: storage_stats.read_count_normalized,
    read_size_bytes: storage_stats.read_size_bytes,
    write_count_normalized: storage_stats.write_count_normalized,
    write_size_bytes: storage_stats.write_size_bytes,
  }
}

fn convert_cpu_stats(
  cpu_stats: models::ContainerCpuStats,
) -> ContainerCpuStats {
  ContainerCpuStats {
    cpu_usage: cpu_stats.cpu_usage.map(convert_cpu_usage),
    system_cpu_usage: cpu_stats.system_cpu_usage,
    online_cpus: cpu_stats.online_cpus,
    throttling_data: cpu_stats
      .throttling_data
      .map(convert_cpu_throttling_data),
  }
}

fn convert_cpu_usage(
  cpu_usage: models::ContainerCpuUsage,
) -> ContainerCpuUsage {
  ContainerCpuUsage {
    total_usage: cpu_usage.total_usage,
    percpu_usage: cpu_usage.percpu_usage,
    usage_in_kernelmode: cpu_usage.usage_in_kernelmode,
    usage_in_usermode: cpu_usage.usage_in_usermode,
  }
}

fn convert_cpu_throttling_data(
  cpu_throttling_data: models::ContainerThrottlingData,
) -> ContainerThrottlingData {
  ContainerThrottlingData {
    periods: cpu_throttling_data.periods,
    throttled_periods: cpu_throttling_data.throttled_periods,
    throttled_time: cpu_throttling_data.throttled_time,
  }
}

fn convert_memory_stats(
  memory_stats: models::ContainerMemoryStats,
) -> ContainerMemoryStats {
  ContainerMemoryStats {
    usage: memory_stats.usage,
    max_usage: memory_stats.max_usage,
    stats: memory_stats.stats,
    failcnt: memory_stats.failcnt,
    limit: memory_stats.limit,
    commitbytes: memory_stats.commitbytes,
    commitpeakbytes: memory_stats.commitpeakbytes,
    privateworkingset: memory_stats.privateworkingset,
  }
}

fn convert_network_stats(
  network_stats: HashMap<
    std::string::String,
    models::ContainerNetworkStats,
  >,
) -> HashMap<std::string::String, ContainerNetworkStats> {
  network_stats
    .into_iter()
    .map(|(name, network_stats)| {
      (
        name,
        ContainerNetworkStats {
          rx_bytes: network_stats.rx_bytes,
          rx_packets: network_stats.rx_packets,
          rx_errors: network_stats.rx_errors,
          rx_dropped: network_stats.rx_dropped,
          tx_bytes: network_stats.tx_bytes,
          tx_packets: network_stats.tx_packets,
          tx_errors: network_stats.tx_errors,
          tx_dropped: network_stats.tx_dropped,
          endpoint_id: network_stats.endpoint_id,
          instance_id: network_stats.instance_id,
        },
      )
    })
    .collect()
}
