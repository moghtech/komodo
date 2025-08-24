use std::{cmp::Ordering, sync::OnceLock};

use async_timing_util::wait_until_timelength;
use komodo_client::entities::stats::{
  SingleDiskUsage, SystemInformation, SystemLoadAverage, SystemProcess, SystemStats,
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::RwLock;

use crate::config::periphery_config;

pub fn stats_client() -> &'static RwLock<StatsClient> {
  static STATS_CLIENT: OnceLock<RwLock<StatsClient>> =
    OnceLock::new();
  STATS_CLIENT.get_or_init(|| RwLock::new(StatsClient::default()))
}

/// This should be called before starting the server in main.rs.
/// Keeps the cached stats up to date
pub fn spawn_polling_thread() {
  tokio::spawn(async move {
    let polling_rate = periphery_config()
      .stats_polling_rate
      .to_string()
      .parse()
      .expect("invalid stats polling rate");
    let client = stats_client();
    loop {
      let ts = wait_until_timelength(polling_rate, 1).await;
      let mut client = client.write().await;
      client.refresh();
      client.stats = client.get_system_stats();
      client.stats.refresh_ts = ts as i64;
    }
  });
}

pub struct StatsClient {
  /// Cached system stats
  pub stats: SystemStats,
  /// Cached system information
  pub info: SystemInformation,

  // the handles used to get the stats
  system: sysinfo::System,
  disks: sysinfo::Disks,
  networks: sysinfo::Networks,
}

const BYTES_PER_GB: f64 = 1073741824.0;
const BYTES_PER_MB: f64 = 1048576.0;
const BYTES_PER_KB: f64 = 1024.0;

impl Default for StatsClient {
  fn default() -> Self {
    let system = sysinfo::System::new_all();
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let stats = SystemStats {
      polling_rate: periphery_config().stats_polling_rate,
      ..Default::default()
    };
    StatsClient {
      info: get_system_information(&system),
      system,
      disks,
      networks,
      stats,
    }
  }
}

impl StatsClient {
  fn refresh(&mut self) {
    self.system.refresh_cpu_all();
    self.system.refresh_memory();
    self.system.refresh_processes_specifics(
      ProcessesToUpdate::All,
      true,
      ProcessRefreshKind::everything().without_tasks(),
    );
    self.disks.refresh(true);
    self.networks.refresh(true);
  }

  pub fn get_system_stats(&self) -> SystemStats {
    let total_mem = self.system.total_memory();
    let available_mem = self.system.available_memory();

    let mut network_ingress_bytes: u64 = 0;
    let mut network_egress_bytes: u64 = 0;

    for (_, network) in self.networks.iter() {
      network_ingress_bytes += network.received();
      network_egress_bytes += network.transmitted();
    }

    let load_avg = System::load_average();

    SystemStats {
      cpu_perc: self.system.global_cpu_usage(),
      load_average: SystemLoadAverage {
        one: load_avg.one,
        five: load_avg.five,
        fifteen: load_avg.fifteen,
      },
      mem_free_gb: self.system.free_memory() as f64 / BYTES_PER_GB,
      mem_used_gb: (total_mem - available_mem) as f64 / BYTES_PER_GB,
      mem_total_gb: total_mem as f64 / BYTES_PER_GB,
      network_ingress_bytes: network_ingress_bytes as f64,
      network_egress_bytes: network_egress_bytes as f64,
      disks: self.get_disks(),
      polling_rate: self.stats.polling_rate,
      refresh_ts: self.stats.refresh_ts,
      refresh_list_ts: self.stats.refresh_list_ts,
    }
  }

  fn get_disks(&self) -> Vec<SingleDiskUsage> {
    let config = periphery_config();
    self
      .disks
      .list()
      .iter()
      .filter(|d| {
        if d.file_system() == "overlay" {
          return false;
        }
        let path = d.mount_point();
        for mount in config.exclude_disk_mounts.iter() {
          if path == mount {
            return false;
          }
        }
        if config.include_disk_mounts.is_empty() {
          return true;
        }
        for mount in config.include_disk_mounts.iter() {
          if path == mount {
            return true;
          }
        }
        false
      })
      .map(|disk| {
        let file_system =
          disk.file_system().to_string_lossy().to_string();
        let disk_total = disk.total_space() as f64 / BYTES_PER_GB;
        let disk_free = disk.available_space() as f64 / BYTES_PER_GB;
        SingleDiskUsage {
          mount: disk.mount_point().to_owned(),
          used_gb: disk_total - disk_free,
          total_gb: disk_total,
          file_system,
        }
      })
      .collect()
  }

  pub fn get_processes(&self) -> Vec<SystemProcess> {
    let mut procs: Vec<_> = self
      .system
      .processes()
      .iter()
      .map(|(pid, p)| {
        let disk_usage = p.disk_usage();
        SystemProcess {
          pid: pid.as_u32(),
          name: p.name().to_string_lossy().to_string(),
          exe: p
            .exe()
            .map(|exe| exe.to_str().unwrap_or_default())
            .unwrap_or_default()
            .to_string(),
          cmd: p
            .cmd()
            .iter()
            .map(|cmd| cmd.to_string_lossy().to_string())
            .collect(),
          start_time: (p.start_time() * 1000) as f64,
          cpu_perc: p.cpu_usage(),
          mem_mb: p.memory() as f64 / BYTES_PER_MB,
          disk_read_kb: disk_usage.read_bytes as f64 / BYTES_PER_KB,
          disk_write_kb: disk_usage.written_bytes as f64
            / BYTES_PER_KB,
        }
      })
      .collect();
    procs.sort_by(|a, b| {
      if a.cpu_perc > b.cpu_perc {
        Ordering::Less
      } else {
        Ordering::Greater
      }
    });
    procs
  }
}

fn get_system_information(
  sys: &sysinfo::System,
) -> SystemInformation {
  let config = periphery_config();
  SystemInformation {
    name: System::name(),
    os: System::long_os_version(),
    kernel: System::kernel_version(),
    host_name: System::host_name(),
    core_count: System::physical_core_count().map(|c| c as u32),
    cpu_brand: sys
      .cpus()
      .iter()
      .next()
      .map(|cpu| cpu.brand().to_string())
      .unwrap_or_default(),
    terminals_disabled: config.disable_terminals,
    container_exec_disabled: config.disable_container_exec,
  }
}
