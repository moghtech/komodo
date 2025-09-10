use std::{
  collections::HashMap,
  sync::{Arc, OnceLock},
};

use anyhow::Context;
use arc_swap::ArcSwap;
use database::Client;
use komodo_client::entities::{
  action::ActionState,
  build::BuildState,
  config::core::{CoreConfig, GithubWebhookAppConfig},
  deployment::DeploymentState,
  procedure::ProcedureState,
  repo::RepoState,
  stack::StackState,
};
use octorust::auth::{
  Credentials, InstallationTokenGenerator, JWTCredentials,
};

use crate::{
  auth::jwt::JwtClient,
  config::core_config,
  helpers::{
    action_state::ActionStates, all_resources::AllResourcesById,
    cache::Cache,
  },
  monitor::{
    CachedDeploymentStatus, CachedRepoStatus, CachedServerStatus,
    CachedStackStatus, History,
  },
};

static DB_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn db_client() -> &'static Client {
  DB_CLIENT
    .get()
    .expect("db_client accessed before initialized")
}

/// Must be called in app startup sequence.
pub async fn init_db_client() {
  let client = Client::new(&core_config().database)
    .await
    .context("failed to initialize database client")
    .unwrap();
  DB_CLIENT.set(client).expect("db_clint");
}

pub fn jwt_client() -> &'static JwtClient {
  static JWT_CLIENT: OnceLock<JwtClient> = OnceLock::new();
  JWT_CLIENT.get_or_init(|| match JwtClient::new(core_config()) {
    Ok(client) => client,
    Err(e) => {
      error!("failed to initialialize JwtClient | {e:#}");
      panic!("Exiting");
    }
  })
}

pub fn github_client()
-> Option<&'static HashMap<String, octorust::Client>> {
  static GITHUB_CLIENT: OnceLock<
    Option<HashMap<String, octorust::Client>>,
  > = OnceLock::new();
  GITHUB_CLIENT
    .get_or_init(|| {
      let CoreConfig {
        github_webhook_app:
          GithubWebhookAppConfig {
            app_id,
            installations,
            pk_path,
            ..
          },
        ..
      } = core_config();
      if *app_id == 0 || installations.is_empty() {
        return None;
      }
      let private_key = match std::fs::read(pk_path).with_context(|| format!("github webhook app | failed to load private key at {pk_path}")) {
        Ok(key) => key,
        Err(e) => {
          error!("{e:#}");
          return None;
        }
      };

      let private_key = match nom_pem::decode_block(&private_key) {
        Ok(key) => key,
        Err(e) => {
          error!("github webhook app | failed to decode private key at {pk_path} | {e:?}");
          return None;
        }
      };

      let jwt = match JWTCredentials::new(*app_id, private_key.data).context("failed to initialize github JWTCredentials") {
        Ok(jwt) => jwt,
        Err(e) => {
          error!("github webhook app | failed to make github JWTCredentials | pk path: {pk_path} | {e:#}");
          return None
        }
      };

      let mut clients =
        HashMap::with_capacity(installations.capacity());

      for installation in installations {
        let token_generator = InstallationTokenGenerator::new(
          installation.id,
          jwt.clone(),
        );
        let client = match octorust::Client::new(
          "github-app",
          Credentials::InstallationToken(token_generator),
        ).with_context(|| format!("failed to initialize github webhook client for installation {}", installation.id)) {
          Ok(client) => client,
          Err(e) => {
            error!("{e:#}");
            continue;
          }
        };
        clients.insert(installation.namespace.to_string(), client);
      }

      Some(clients)
    })
    .as_ref()
}

pub fn action_states() -> &'static ActionStates {
  static ACTION_STATES: OnceLock<ActionStates> = OnceLock::new();
  ACTION_STATES.get_or_init(ActionStates::default)
}

/// Cache of ids to status
pub type DeploymentStatusCache = Cache<
  String,
  Arc<History<CachedDeploymentStatus, DeploymentState>>,
>;

/// Cache of ids to status
pub fn deployment_status_cache() -> &'static DeploymentStatusCache {
  static DEPLOYMENT_STATUS_CACHE: OnceLock<DeploymentStatusCache> =
    OnceLock::new();
  DEPLOYMENT_STATUS_CACHE.get_or_init(Default::default)
}

pub type StackStatusCache =
  Cache<String, Arc<History<CachedStackStatus, StackState>>>;

pub fn stack_status_cache() -> &'static StackStatusCache {
  static STACK_STATUS_CACHE: OnceLock<StackStatusCache> =
    OnceLock::new();
  STACK_STATUS_CACHE.get_or_init(Default::default)
}

pub type ServerStatusCache = Cache<String, Arc<CachedServerStatus>>;

pub fn server_status_cache() -> &'static ServerStatusCache {
  static SERVER_STATUS_CACHE: OnceLock<ServerStatusCache> =
    OnceLock::new();
  SERVER_STATUS_CACHE.get_or_init(Default::default)
}

pub type RepoStatusCache = Cache<String, Arc<CachedRepoStatus>>;

pub fn repo_status_cache() -> &'static RepoStatusCache {
  static REPO_STATUS_CACHE: OnceLock<RepoStatusCache> =
    OnceLock::new();
  REPO_STATUS_CACHE.get_or_init(Default::default)
}

pub type BuildStateCache = Cache<String, BuildState>;

pub fn build_state_cache() -> &'static BuildStateCache {
  static BUILD_STATE_CACHE: OnceLock<BuildStateCache> =
    OnceLock::new();
  BUILD_STATE_CACHE.get_or_init(Default::default)
}

pub type RepoStateCache = Cache<String, RepoState>;

pub fn repo_state_cache() -> &'static RepoStateCache {
  static REPO_STATE_CACHE: OnceLock<RepoStateCache> = OnceLock::new();
  REPO_STATE_CACHE.get_or_init(Default::default)
}

pub type ProcedureStateCache = Cache<String, ProcedureState>;

pub fn procedure_state_cache() -> &'static ProcedureStateCache {
  static PROCEDURE_STATE_CACHE: OnceLock<ProcedureStateCache> =
    OnceLock::new();
  PROCEDURE_STATE_CACHE.get_or_init(Default::default)
}

pub type ActionStateCache = Cache<String, ActionState>;

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
