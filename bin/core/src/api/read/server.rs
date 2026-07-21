use std::{
  collections::HashMap,
  sync::{Arc, OnceLock},
};

use anyhow::{Context, anyhow};
use async_timing_util::{
  FIFTEEN_SECONDS_MS, get_timelength_in_ms, unix_timestamp_ms,
};
use database::mungos::{
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    server::{
      Server, ServerActionState, ServerListItem, ServerSortBy,
      ServerState,
    },
    stats::{SystemInformation, SystemProcess},
  },
};
use mogh_error::AddStatusCode;
use mogh_resolver::Resolve;
use periphery_client::api as periphery;
use reqwest::StatusCode;
use tokio::sync::Mutex;

use crate::{
  helpers::{
    periphery_client,
    query::{get_all_tags, get_cached_server_state},
  },
  permission::get_check_permissions,
  resource,
  state::{action_states, db_client, server_status_cache},
};

use super::{ReadArgs, list_limit};

impl Resolve<ReadArgs> for GetServersSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetServersSummaryResponse> {
    let servers = resource::list_for_user::<Server>(
      Default::default(),
      None,
      None,
      user,
      PermissionLevel::Read.into(),
      &[],
    )
    .await?;

    let core_version = env!("CARGO_PKG_VERSION");
    let mut res = GetServersSummaryResponse::default();

    for server in servers {
      res.total += 1;
      match server.info.state {
        ServerState::Ok => {
          // Check for version mismatch
          if matches!(&server.info.version, Some(version) if version != core_version)
          {
            res.warning += 1;
          } else {
            res.healthy += 1;
          }
        }
        ServerState::NotOk => {
          res.unhealthy += 1;
        }
        ServerState::Disabled => {
          if !server.template {
            res.disabled += 1;
          }
        }
      }
    }
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetServer {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Server> {
    Ok(
      get_check_permissions::<Server>(
        &self.server,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListServers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<ServerListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = list_limit(self.limit);
    let sort_by: resource::ListItemSort<ServerListItem> =
      match self.sort_by {
        ServerSortBy::Name => resource::ListItemSort::Name,
        ServerSortBy::Region => {
          resource::ListItemSort::DbField("config.region")
        }
        ServerSortBy::Version => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info
              .version
              .cmp(&b.info.version)
              .then_with(|| a.name.cmp(&b.name))
          }))
        }
        ServerSortBy::State => {
          resource::ListItemSort::InMemory(Box::new(|a, b| {
            a.info
              .state
              .cmp(&b.info.state)
              .then_with(|| a.name.cmp(&b.name))
          }))
        }
      };
    let servers = resource::list_items_for_user::<Server>(
      self.query,
      resource::ListItemsQueryOptions {
        limit,
        page: self.page,
        sort_desc: self.sort_desc,
        sort_by,
      },
      user,
      PermissionLevel::Read.into(),
      &all_tags,
      |server| {
        states.is_empty() || states.contains(&server.info.state)
      },
    )
    .await?;
    Ok(servers)
  }
}

impl Resolve<ReadArgs> for ListFullServers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullServersResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let states = self.query.specific.states.clone();
    let limit = list_limit(self.limit);
    Ok(
      resource::list_full_for_user_filtered::<Server, _>(
        self.query,
        limit,
        self.page,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
        |server| {
          let states = states.clone();
          async move {
            if states.is_empty()
              || states
                .contains(&get_cached_server_state(&server.id).await)
            {
              Some(server)
            } else {
              None
            }
          }
        },
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetServerState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetServerStateResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let status = server_status_cache()
      .get(&server.id)
      .await
      .ok_or(anyhow!("did not find cached status for server"))?;
    let response = GetServerStateResponse {
      status: status.state,
    };
    Ok(response)
  }
}

impl Resolve<ReadArgs> for GetServerActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ServerActionState> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .server
      .get(&server.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetPeripheryInformation {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetPeripheryInformationResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .periphery_info
      .as_ref()
      .cloned()
      .context("Server status missing Periphery Info. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

impl Resolve<ReadArgs> for GetSystemInformation {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<SystemInformation> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await
    .status_code(StatusCode::BAD_REQUEST)?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .system_info
      .as_ref()
      .cloned()
      .context("Server status missing system Info. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

impl Resolve<ReadArgs> for GetSystemStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetSystemStatsResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    server_status_cache()
      .get(&server.id)
      .await
      .context("Missing server status")?
      .system_stats
      .as_ref()
      .cloned()
      .context("Server status missing system stats. The Server may be disconnected.")
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

// This protects the peripheries from spam requests
const PROCESSES_EXPIRY: u128 = FIFTEEN_SECONDS_MS;
type ProcessesCache =
  Mutex<HashMap<String, Arc<(Vec<SystemProcess>, u128)>>>;
fn processes_cache() -> &'static ProcessesCache {
  static PROCESSES_CACHE: OnceLock<ProcessesCache> = OnceLock::new();
  PROCESSES_CACHE.get_or_init(Default::default)
}

impl Resolve<ReadArgs> for ListSystemProcesses {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListSystemProcessesResponse> {
    let server = get_check_permissions::<Server>(
      &self.server,
      user,
      PermissionLevel::Read.processes(),
    )
    .await?;
    let mut lock = processes_cache().lock().await;
    let res = match lock.get(&server.id) {
      Some(cached) if cached.1 > unix_timestamp_ms() => {
        cached.0.clone()
      }
      _ => {
        let stats = periphery_client(&server)
          .await?
          .request(periphery::stats::GetSystemProcesses {})
          .await?;
        lock.insert(
          server.id,
          (stats.clone(), unix_timestamp_ms() + PROCESSES_EXPIRY)
            .into(),
        );
        stats
      }
    };
    Ok(res)
  }
}

const STATS_PER_PAGE: i64 = 200;

impl Resolve<ReadArgs> for GetHistoricalServerStats {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetHistoricalServerStatsResponse> {
    let GetHistoricalServerStats {
      server,
      granularity,
      page,
    } = self;
    let server = get_check_permissions::<Server>(
      &server,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let granularity =
      get_timelength_in_ms(granularity.to_string().parse().unwrap())
        as i64;
    let mut ts_vec = Vec::<i64>::new();
    let curr_ts = unix_timestamp_ms() as i64;
    let mut curr_ts = curr_ts
      - curr_ts % granularity
      - granularity * (page as i64).saturating_mul(STATS_PER_PAGE);
    for _ in 0..STATS_PER_PAGE {
      ts_vec.push(curr_ts);
      curr_ts -= granularity;
    }

    let stats = find_collect(
      &db_client().stats,
      doc! {
        "sid": server.id,
        "ts": { "$in": ts_vec },
      },
      FindOptions::builder()
        .sort(doc! { "ts": -1 })
        .limit(STATS_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to pull stats from db")?;
    let next_page = if stats.len() == STATS_PER_PAGE as usize {
      Some(page + 1)
    } else {
      None
    };
    let res = GetHistoricalServerStatsResponse { stats, next_page };
    Ok(res)
  }
}
