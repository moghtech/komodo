use komodo_client::entities::{
  docker::DockerLists,
  server::PeripheryInformation,
  stats::{SystemInformation, SystemStats},
};
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};

/// This is the data Core uses to update all Server-related status caches.
#[derive(Debug, Clone, Serialize, Deserialize, Resolve)]
#[response(PollStatusResponse)]
#[error(anyhow::Error)]
pub struct PollStatus {
  /// Include system stats
  pub include_stats: bool,
  /// Include docker info
  pub include_docker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollStatusResponse {
  pub periphery_info: PeripheryInformation,
  /// Basic system information
  pub system_info: SystemInformation,
  /// Current System Stats (Cpu, Mem, Disk)
  pub system_stats: Option<SystemStats>,
  /// Docker lists
  pub docker: Option<DockerLists>,
}
