use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{
  I64, Timelength,
  server::{
    PeripheryInformation, Server, ServerActionState, ServerListItem,
    ServerQuery, ServerState,
  },
  stats::{
    SystemInformation, SystemProcess, SystemStats, SystemStatsRecord,
  },
};

use super::KomodoReadRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetServer",
  description = "Get a specific server.",
  request_body(content = GetServer),
  responses(
    (status = 200, description = "The server", body = GetServerResponse),
  ),
)]
pub fn get_server() {}

/// Get a specific server. Response: [Server].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(Server)]
#[error(mogh_error::Error)]
pub struct GetServer {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type GetServerResponse = Server;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListServers",
  description = "List servers matching optional query.",
  request_body(content = ListServers),
  responses(
    (status = 200, description = "The list of servers", body = ListServersResponse),
  ),
)]
pub fn list_servers() {}

/// List servers matching optional query. Response: [ListServersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListServersResponse)]
#[error(mogh_error::Error)]
pub struct ListServers {
  /// optional structured query to filter servers.
  #[serde(default)]
  pub query: ServerQuery,
}

#[typeshare]
pub type ListServersResponse = Vec<ServerListItem>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListFullServers",
  description = "List servers matching optional query.",
  request_body(content = ListFullServers),
  responses(
    (status = 200, description = "The list of servers", body = ListFullServersResponse),
  ),
)]
pub fn list_full_servers() {}

/// List servers matching optional query. Response: [ListFullServersResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListFullServersResponse)]
#[error(mogh_error::Error)]
pub struct ListFullServers {
  /// optional structured query to filter servers.
  #[serde(default)]
  pub query: ServerQuery,
}

#[typeshare]
pub type ListFullServersResponse = Vec<Server>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetServerState",
  description = "Get the state of the target server.",
  request_body(content = GetServerState),
  responses(
    (status = 200, description = "The server state", body = GetServerStateResponse),
  ),
)]
pub fn get_server_state() {}

/// Get the state of the target server. Response: [GetServerStateResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetServerStateResponse)]
#[error(mogh_error::Error)]
pub struct GetServerState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

/// The response for [GetServerState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetServerStateResponse {
  /// The server status.
  pub status: ServerState,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetServerActionState",
  description = "Get current action state for the servers.",
  request_body(content = GetServerActionState),
  responses(
    (status = 200, description = "The server action state", body = GetServerActionStateResponse),
  ),
)]
pub fn get_server_action_state() {}

/// Get current action state for the servers. Response: [ServerActionState].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ServerActionState)]
#[error(mogh_error::Error)]
pub struct GetServerActionState {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type GetServerActionStateResponse = ServerActionState;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetPeripheryInformation",
  description = "Get the Periphery information of the target server, including the Periphery version and public key.",
  request_body(content = GetPeripheryInformation),
  responses(
    (status = 200, description = "The periphery information", body = GetPeripheryInformationResponse),
  ),
)]
pub fn get_periphery_information() {}

/// Get the Periphery information of the target server,
/// including the Periphery version and public key.
/// Response: [PeripheryInformation].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetPeripheryInformationResponse)]
#[error(mogh_error::Error)]
pub struct GetPeripheryInformation {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type GetPeripheryInformationResponse = PeripheryInformation;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetSystemInformation",
  description = "Get the system information of the target server.",
  request_body(content = GetSystemInformation),
  responses(
    (status = 200, description = "The system information", body = GetSystemInformationResponse),
  ),
)]
pub fn get_system_information() {}

/// Get the system information of the target server.
/// Response: [SystemInformation].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSystemInformationResponse)]
#[error(mogh_error::Error)]
pub struct GetSystemInformation {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type GetSystemInformationResponse = SystemInformation;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetSystemStats",
  description = "Get the system stats on the target server.",
  request_body(content = GetSystemStats),
  responses(
    (status = 200, description = "The system stats", body = GetSystemStatsResponse),
  ),
)]
pub fn get_system_stats() {}

/// Get the system stats on the target server. Response: [SystemStats].
///
/// Note. This does not hit the server directly. The stats come from an
/// in memory cache on the core, which hits the server periodically
/// to keep it up to date.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetSystemStatsResponse)]
#[error(mogh_error::Error)]
pub struct GetSystemStats {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type GetSystemStatsResponse = SystemStats;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/ListSystemProcesses",
  description = "List the processes running on the target server.",
  request_body(content = ListSystemProcesses),
  responses(
    (status = 200, description = "The list of processes", body = ListSystemProcessesResponse),
  ),
)]
pub fn list_system_processes() {}

/// List the processes running on the target server.
/// Response: [ListSystemProcessesResponse].
///
/// Note. This does not hit the server directly. The procedures come from an
/// in memory cache on the core, which hits the server periodically
/// to keep it up to date.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(ListSystemProcessesResponse)]
#[error(mogh_error::Error)]
pub struct ListSystemProcesses {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
}

#[typeshare]
pub type ListSystemProcessesResponse = Vec<SystemProcess>;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetHistoricalServerStats",
  description = "Paginated endpoint serving historical (timeseries) server stats for graphing.",
  request_body(content = GetHistoricalServerStats),
  responses(
    (status = 200, description = "The historical server stats", body = GetHistoricalServerStatsResponse),
  ),
)]
pub fn get_historical_server_stats() {}

/// Paginated endpoint serving historical (timeseries) server stats for graphing.
/// Response: [GetHistoricalServerStatsResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetHistoricalServerStatsResponse)]
#[error(mogh_error::Error)]
pub struct GetHistoricalServerStats {
  /// Id or name
  #[serde(alias = "id", alias = "name")]
  pub server: String,
  /// The granularity of the data.
  pub granularity: Timelength,
  /// Page of historical data. Default is 0, which is the most recent data.
  /// Use with the `next_page` field of the response.
  #[serde(default)]
  pub page: u32,
}

/// Response to [GetHistoricalServerStats].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetHistoricalServerStatsResponse {
  /// The timeseries page of data.
  pub stats: Vec<SystemStatsRecord>,
  /// If there is a next page of data, pass this to `page` to get it.
  pub next_page: Option<u32>,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/read/GetServersSummary",
  description = "Gets a summary of data relating to all servers.",
  request_body(content = GetServersSummary),
  responses(
    (status = 200, description = "The servers summary", body = GetServersSummaryResponse),
  ),
)]
pub fn get_servers_summary() {}

/// Gets a summary of data relating to all servers.
/// Response: [GetServersSummaryResponse].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoReadRequest)]
#[response(GetServersSummaryResponse)]
#[error(mogh_error::Error)]
pub struct GetServersSummary {}

/// Response for [GetServersSummary].
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetServersSummaryResponse {
  /// The total number of servers.
  pub total: I64,
  /// The number of healthy (`status: OK`) servers.
  pub healthy: I64,
  /// The number of servers with warnings (e.g., version mismatch).
  pub warning: I64,
  /// The number of unhealthy servers.
  pub unhealthy: I64,
  /// The number of disabled servers.
  pub disabled: I64,
}
