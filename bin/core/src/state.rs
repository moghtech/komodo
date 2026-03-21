use std::sync::{Arc, OnceLock};

use anyhow::{Context, anyhow};
use arc_swap::ArcSwap;
use komodo_client::entities::{
  ImageDigest,
  action::ActionState,
  build::BuildState,
  deployment::DeploymentState,
  docker::{
    DockerLists, SwarmLists, container::ContainerListItem,
    service::SwarmServiceListItem, swarm::SwarmInspectInfo,
  },
  procedure::ProcedureState,
  repo::RepoState,
  server::{PeripheryInformation, ServerHealth, ServerState},
  stack::{StackService, StackState},
  stats::{SystemInformation, SystemStats},
  swarm::SwarmState,
};
use mogh_cache::CloneCache;

use crate::{
  config::core_config,
  connection::PeripheryConnections,
  helpers::{
    action_state::ActionStates, all_resources::AllResourcesById,
    image_digest::ImageDigestCache,
  },
};

static DB_CLIENT: OnceLock<database::Client> = OnceLock::new();

pub fn db_client() -> &'static database::Client {
  DB_CLIENT.get().unwrap_or_else(|| {
    error!(
      "FATAL: db_client accessed before initialized | Ensure init_db_client() is called during startup | Exiting..."
    );
    std::process::exit(1)
  })
}

/// Must be called in app startup sequence.
pub async fn init_db_client() {
  let init = async {
    let client = database::Client::new(&core_config().database)
      .await
      .context("failed to initialize database client")?;
    DB_CLIENT.set(client).map_err(|_| {
    anyhow!(
      "db_client initialized more than once - this should not happen"
    )
  })?;
    anyhow::Ok(())
  }
  .await;
  if let Err(e) = init {
    error!(
      "FATAL: Failed to initialize database::Client | {e:#} | Exiting..."
    );
    std::process::exit(1)
  }
}

/// server id => connection
pub fn periphery_connections() -> &'static PeripheryConnections {
  static CONNECTIONS: OnceLock<PeripheryConnections> =
    OnceLock::new();
  CONNECTIONS.get_or_init(Default::default)
}

pub fn action_states() -> &'static ActionStates {
  static ACTION_STATES: OnceLock<ActionStates> = OnceLock::new();
  ACTION_STATES.get_or_init(ActionStates::default)
}

#[derive(Default, Debug)]
pub struct History<Curr: Default, Prev> {
  pub curr: Curr,
  pub prev: Option<Prev>,
}

#[derive(Default, Clone, Debug)]
pub struct CachedSwarmStatus {
  pub id: String,
  pub state: SwarmState,
  pub inspect: Option<SwarmInspectInfo>,
  pub lists: Option<SwarmLists>,
  /// Store the error in communicating with Swarm
  pub err: Option<String>,
}

pub type SwarmStatusCache =
  CloneCache<String, Arc<CachedSwarmStatus>>;

pub fn swarm_status_cache() -> &'static SwarmStatusCache {
  static SWARM_STATUS_CACHE: OnceLock<SwarmStatusCache> =
    OnceLock::new();
  SWARM_STATUS_CACHE.get_or_init(Default::default)
}

#[derive(Default, Clone, Debug)]
pub struct CachedServerStatus {
  pub id: String,
  pub state: ServerState,
  pub health: Option<ServerHealth>,
  pub periphery_info: Option<PeripheryInformation>,
  pub system_info: Option<SystemInformation>,
  pub system_stats: Option<SystemStats>,
  pub docker: Option<DockerLists>,
  /// Store the error in reaching periphery
  pub err: Option<mogh_error::Serror>,
}

pub type ServerStatusCache =
  CloneCache<String, Arc<CachedServerStatus>>;

pub fn server_status_cache() -> &'static ServerStatusCache {
  static SERVER_STATUS_CACHE: OnceLock<ServerStatusCache> =
    OnceLock::new();
  SERVER_STATUS_CACHE.get_or_init(Default::default)
}

#[derive(Default, Clone, Debug)]
pub struct CachedStackStatus {
  /// The stack id
  pub id: String,
  /// The stack state
  pub state: StackState,
  /// The services connected to the stack
  pub services: Vec<StackService>,
}

pub type StackStatusCache =
  CloneCache<String, Arc<History<CachedStackStatus, StackState>>>;

pub fn stack_status_cache() -> &'static StackStatusCache {
  static STACK_STATUS_CACHE: OnceLock<StackStatusCache> =
    OnceLock::new();
  STACK_STATUS_CACHE.get_or_init(Default::default)
}

#[derive(Default, Clone, Debug)]
pub struct CachedDeploymentStatus {
  /// The deployment id
  pub id: String,
  pub state: DeploymentState,
  pub container: Option<ContainerListItem>,
  pub service: Option<SwarmServiceListItem>,
  pub image_digests: Option<Vec<ImageDigest>>,
}

/// Cache of ids to status
pub type DeploymentStatusCache = CloneCache<
  String,
  Arc<History<CachedDeploymentStatus, DeploymentState>>,
>;

/// Cache of ids to status
pub fn deployment_status_cache() -> &'static DeploymentStatusCache {
  static DEPLOYMENT_STATUS_CACHE: OnceLock<DeploymentStatusCache> =
    OnceLock::new();
  DEPLOYMENT_STATUS_CACHE.get_or_init(Default::default)
}

pub type BuildStateCache = CloneCache<String, BuildState>;

pub fn build_state_cache() -> &'static BuildStateCache {
  static BUILD_STATE_CACHE: OnceLock<BuildStateCache> =
    OnceLock::new();
  BUILD_STATE_CACHE.get_or_init(Default::default)
}

#[derive(Default, Clone, Debug)]
pub struct CachedRepoStatus {
  pub latest_hash: Option<String>,
  pub latest_message: Option<String>,
}

pub type RepoStatusCache = CloneCache<String, Arc<CachedRepoStatus>>;

pub fn repo_status_cache() -> &'static RepoStatusCache {
  static REPO_STATUS_CACHE: OnceLock<RepoStatusCache> =
    OnceLock::new();
  REPO_STATUS_CACHE.get_or_init(Default::default)
}

pub type RepoStateCache = CloneCache<String, RepoState>;

pub fn repo_state_cache() -> &'static RepoStateCache {
  static REPO_STATE_CACHE: OnceLock<RepoStateCache> = OnceLock::new();
  REPO_STATE_CACHE.get_or_init(Default::default)
}

pub type ProcedureStateCache = CloneCache<String, ProcedureState>;

pub fn procedure_state_cache() -> &'static ProcedureStateCache {
  static PROCEDURE_STATE_CACHE: OnceLock<ProcedureStateCache> =
    OnceLock::new();
  PROCEDURE_STATE_CACHE.get_or_init(Default::default)
}

pub type ActionStateCache = CloneCache<String, ActionState>;

pub fn action_state_cache() -> &'static ActionStateCache {
  static ACTION_STATE_CACHE: OnceLock<ActionStateCache> =
    OnceLock::new();
  ACTION_STATE_CACHE.get_or_init(Default::default)
}

pub fn all_resources_cache() -> &'static ArcSwap<AllResourcesById> {
  static ALL_RESOURCES: OnceLock<ArcSwap<AllResourcesById>> =
    OnceLock::new();
  ALL_RESOURCES.get_or_init(Default::default)
}

/// Maps Image name => (Digest, valid until ms).
/// Cache the latest queried image digests in order
/// to infer whether deployments / stacks have updates available.
pub fn image_digest_cache() -> &'static ImageDigestCache {
  static IMAGE_DIGEST_CACHE: OnceLock<Arc<ImageDigestCache>> =
    OnceLock::new();
  IMAGE_DIGEST_CACHE.get_or_init(ImageDigestCache::new)
}
