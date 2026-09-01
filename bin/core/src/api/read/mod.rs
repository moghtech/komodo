use std::collections::HashSet;

use anyhow::{Context, anyhow};
use axum::{
  Extension, Router, extract::Path, middleware, routing::post,
};
use komodo_client::{
  api::read::*,
  entities::{
    ResourceTarget,
    build::Build,
    builder::{Builder, BuilderConfig},
    config::{GitProvider, ImageRegistry},
    permission::PermissionLevel,
    repo::Repo,
    server::Server,
    sync::ResourceSync,
    user::User,
  },
};
use mogh_auth_server::middleware::authenticate_request;
use mogh_error::Response;
use mogh_error::{AddStatusCodeError, Json};
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::{Display, EnumDiscriminants};
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::KomodoAuthImpl,
  config::{core_config, core_keys},
  helpers::periphery_client,
  resource,
};

use super::Variant;

mod action;
mod alert;
mod alerter;
mod build;
mod builder;
mod deployment;
mod docker;
mod onboarding_key;
mod permission;
mod procedure;
mod provider;
mod repo;
mod schedule;
mod server;
mod stack;
mod swarm;
mod sync;
mod tag;
mod terminal;
mod toml;
mod update;
mod user;
mod user_group;
mod variable;

pub struct ReadArgs {
  pub user: User,
}

/// Resolve the page limit for List apis, falling back to the
/// configured `default_pagination_limit` when not provided.
fn list_limit(limit: Option<u64>) -> u64 {
  limit.unwrap_or(core_config().default_pagination_limit)
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumDiscriminants,
)]
#[strum_discriminants(name(ReadRequestMethod), derive(Display))]
#[args(ReadArgs)]
#[response(Response)]
#[error(mogh_error::Error)]
#[serde(tag = "type", content = "params")]
enum ReadRequest {
  GetVersion(GetVersion),
  GetCoreInfo(GetCoreInfo),
  ListSecrets(ListSecrets),
  ListGitProvidersFromConfig(ListGitProvidersFromConfig),
  #[serde(alias = "ListDockerRegistriesFromConfig")]
  ListImageRegistriesFromConfig(ListImageRegistriesFromConfig),

  // ==== SWARM ====
  GetSwarmsSummary(GetSwarmsSummary),
  GetSwarm(GetSwarm),
  GetSwarmActionState(GetSwarmActionState),
  ListSwarms(ListSwarms),
  InspectSwarm(InspectSwarm),
  ListFullSwarms(ListFullSwarms),
  ListSwarmNodes(ListSwarmNodes),
  InspectSwarmNode(InspectSwarmNode),
  ListSwarmConfigs(ListSwarmConfigs),
  InspectSwarmConfig(InspectSwarmConfig),
  ListSwarmSecrets(ListSwarmSecrets),
  InspectSwarmSecret(InspectSwarmSecret),
  ListSwarmStacks(ListSwarmStacks),
  InspectSwarmStack(InspectSwarmStack),
  ListSwarmTasks(ListSwarmTasks),
  InspectSwarmTask(InspectSwarmTask),
  ListSwarmServices(ListSwarmServices),
  InspectSwarmService(InspectSwarmService),
  GetSwarmServiceLog(GetSwarmServiceLog),
  SearchSwarmServiceLog(SearchSwarmServiceLog),
  ListSwarmNetworks(ListSwarmNetworks),

  // ==== SERVER ====
  GetServersSummary(GetServersSummary),
  GetServer(GetServer),
  GetServerState(GetServerState),
  GetPeripheryInformation(GetPeripheryInformation),
  GetServerActionState(GetServerActionState),
  ListServers(ListServers),
  ListFullServers(ListFullServers),

  // ==== TERMINAL ====
  ListTerminals(ListTerminals),

  // ==== CONTAINER ====
  #[serde(alias = "GetDockerContainersSummary")]
  GetContainersSummary(GetContainersSummary),
  #[serde(alias = "ListAllDockerContainers")]
  ListAllContainers(ListAllContainers),
  #[serde(alias = "ListDockerContainers")]
  ListContainers(ListContainers),
  #[serde(alias = "InspectDockerContainer")]
  InspectContainer(InspectContainer),
  GetResourceMatchingContainer(GetResourceMatchingContainer),
  GetContainerLog(GetContainerLog),
  SearchContainerLog(SearchContainerLog),
  ListComposeProjects(ListComposeProjects),
  #[serde(alias = "ListDockerNetworks")]
  ListNetworks(ListNetworks),
  #[serde(alias = "InspectDockerNetwork")]
  InspectNetwork(InspectNetwork),
  #[serde(alias = "ListDockerImages")]
  ListImages(ListImages),
  #[serde(alias = "InspectDockerImage")]
  InspectImage(InspectImage),
  #[serde(alias = "ListDockerImageHistory")]
  ListImageHistory(ListImageHistory),
  #[serde(alias = "ListDockerVolumes")]
  ListVolumes(ListVolumes),
  #[serde(alias = "InspectDockerVolume")]
  InspectVolume(InspectVolume),

  // ==== SERVER STATS ====
  GetSystemInformation(GetSystemInformation),
  GetSystemStats(GetSystemStats),
  GetHistoricalServerStats(GetHistoricalServerStats),
  ListSystemProcesses(ListSystemProcesses),

  // ==== STACK ====
  GetStacksSummary(GetStacksSummary),
  GetStack(GetStack),
  GetStackActionState(GetStackActionState),
  GetStackLog(GetStackLog),
  SearchStackLog(SearchStackLog),
  InspectStackContainer(InspectStackContainer),
  InspectStackSwarmService(InspectStackSwarmService),
  ListStacks(ListStacks),
  ListFullStacks(ListFullStacks),
  ListStackServices(ListStackServices),
  ListAllStackServices(ListAllStackServices),
  ListCommonStackExtraArgs(ListCommonStackExtraArgs),
  ListCommonStackBuildExtraArgs(ListCommonStackBuildExtraArgs),

  // ==== DEPLOYMENT ====
  GetDeploymentsSummary(GetDeploymentsSummary),
  GetDeployment(GetDeployment),
  GetDeploymentContainer(GetDeploymentContainer),
  GetDeploymentActionState(GetDeploymentActionState),
  GetDeploymentStats(GetDeploymentStats),
  GetDeploymentLog(GetDeploymentLog),
  SearchDeploymentLog(SearchDeploymentLog),
  InspectDeploymentContainer(InspectDeploymentContainer),
  InspectDeploymentSwarmService(InspectDeploymentSwarmService),
  ListDeployments(ListDeployments),
  ListFullDeployments(ListFullDeployments),
  ListCommonDeploymentExtraArgs(ListCommonDeploymentExtraArgs),

  // ==== BUILD ====
  GetBuildsSummary(GetBuildsSummary),
  GetBuild(GetBuild),
  GetBuildActionState(GetBuildActionState),
  GetBuildMonthlyStats(GetBuildMonthlyStats),
  ListBuildVersions(ListBuildVersions),
  ListBuilds(ListBuilds),
  ListFullBuilds(ListFullBuilds),
  ListCommonBuildExtraArgs(ListCommonBuildExtraArgs),

  // ==== REPO ====
  GetReposSummary(GetReposSummary),
  GetRepo(GetRepo),
  GetRepoActionState(GetRepoActionState),
  ListRepos(ListRepos),
  ListFullRepos(ListFullRepos),

  // ==== PROCEDURE ====
  GetProceduresSummary(GetProceduresSummary),
  GetProcedure(GetProcedure),
  GetProcedureActionState(GetProcedureActionState),
  ListProcedures(ListProcedures),
  ListFullProcedures(ListFullProcedures),

  // ==== ACTION ====
  GetActionsSummary(GetActionsSummary),
  GetAction(GetAction),
  GetActionActionState(GetActionActionState),
  ListActions(ListActions),
  ListFullActions(ListFullActions),

  // ==== SCHEDULE ====
  ListSchedules(ListSchedules),

  // ==== SYNC ====
  GetResourceSyncsSummary(GetResourceSyncsSummary),
  GetResourceSync(GetResourceSync),
  GetResourceSyncActionState(GetResourceSyncActionState),
  ListResourceSyncs(ListResourceSyncs),
  ListFullResourceSyncs(ListFullResourceSyncs),

  // ==== BUILDER ====
  GetBuildersSummary(GetBuildersSummary),
  GetBuilder(GetBuilder),
  ListBuilders(ListBuilders),
  ListFullBuilders(ListFullBuilders),

  // ==== ALERTER ====
  GetAlertersSummary(GetAlertersSummary),
  GetAlerter(GetAlerter),
  ListAlerters(ListAlerters),
  ListFullAlerters(ListFullAlerters),

  // ==== TOML ====
  ExportAllResourcesToToml(ExportAllResourcesToToml),
  ExportResourcesToToml(ExportResourcesToToml),

  // ==== TAG ====
  GetTag(GetTag),
  ListTags(ListTags),

  // ==== USER ====
  GetUsername(GetUsername),
  GetPermission(GetPermission),
  FindUser(FindUser),
  ListUsers(ListUsers),
  ListApiKeys(ListApiKeys),
  ListApiKeysForServiceUser(ListApiKeysForServiceUser),
  ListPermissions(ListPermissions),
  ListUserTargetPermissions(ListUserTargetPermissions),

  // ==== USER GROUP ====
  GetUserGroup(GetUserGroup),
  ListUserGroups(ListUserGroups),

  // ==== UPDATE ====
  GetUpdate(GetUpdate),
  ListUpdates(ListUpdates),

  // ==== ALERT ====
  ListAlerts(ListAlerts),
  GetAlert(GetAlert),

  // ==== VARIABLE ====
  GetVariable(GetVariable),
  ListVariables(ListVariables),

  // ==== PROVIDER ====
  GetGitProviderAccount(GetGitProviderAccount),
  ListGitProviderAccounts(ListGitProviderAccounts),
  #[serde(alias = "GetDockerRegistryAccount")]
  GetImageRegistryAccount(GetImageRegistryAccount),
  #[serde(alias = "ListDockerRegistryAccounts")]
  ListImageRegistryAccounts(ListImageRegistryAccounts),

  // ==== ONBOARDING KEY ====
  ListOnboardingKeys(ListOnboardingKeys),
}

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .route("/{variant}", post(variant_handler))
    .layer(middleware::from_fn(
      authenticate_request::<KomodoAuthImpl, true>,
    ))
}

async fn variant_handler(
  user: Extension<User>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> mogh_error::Result<axum::response::Response> {
  let req: ReadRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(user, Json(req)).await
}

async fn handler(
  Extension(user): Extension<User>,
  Json(request): Json<ReadRequest>,
) -> mogh_error::Result<axum::response::Response> {
  let req_id = Uuid::new_v4();
  let method: ReadRequestMethod = (&request).into();

  let user_id = user.id.clone();
  let username = user.username.clone();

  trace!(
    req_id = req_id.to_string(),
    method = method.to_string(),
    user_id,
    username,
    "READ REQUEST",
  );

  let res = request.resolve(&ReadArgs { user }).await;

  if let Err(e) = &res {
    trace!(
      req_id = req_id.to_string(),
      method = method.to_string(),
      user_id,
      username,
      "READ REQUEST | ERROR: {:#}",
      e.error
    );
  }

  res.map(|res| res.0)
}

impl Resolve<ReadArgs> for GetVersion {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> mogh_error::Result<GetVersionResponse> {
    Ok(GetVersionResponse {
      version: env!("CARGO_PKG_VERSION").to_string(),
    })
  }
}

//

impl Resolve<ReadArgs> for GetCoreInfo {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> mogh_error::Result<GetCoreInfoResponse> {
    let config = core_config();
    let info = GetCoreInfoResponse {
      title: config.title.clone(),
      monitoring_interval: config.monitoring_interval,
      webhook_base_url: if config.webhook_base_url.is_empty() {
        config.host.clone()
      } else {
        config.webhook_base_url.clone()
      },
      transparent_mode: config.transparent_mode,
      ui_write_disabled: config.ui_write_disabled,
      disable_confirm_dialog: config.disable_confirm_dialog,
      disable_non_admin_create: config.disable_non_admin_create,
      disable_websocket_reconnect: config.disable_websocket_reconnect,
      enable_fancy_toml: config.enable_fancy_toml,
      timezone: config.timezone.clone(),
      default_pagination_limit: config.default_pagination_limit,
      public_key: core_keys().load().public.to_string(),
    };
    Ok(info)
  }
}

//

impl Resolve<ReadArgs> for ListSecrets {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> mogh_error::Result<ListSecretsResponse> {
    let mut secrets = core_config()
      .secrets
      .keys()
      .cloned()
      .collect::<HashSet<_>>();

    if let Some(target) = self.target {
      let server_id = match target {
        ResourceTarget::Server(id) => Some(id),
        ResourceTarget::Builder(id) => {
          match resource::get::<Builder>(&id).await?.config {
            BuilderConfig::Url(_) => None,
            BuilderConfig::Server(config) => {
              config.server_ids.first().cloned()
            }
            BuilderConfig::Aws(config) => {
              secrets.extend(config.secrets);
              None
            }
          }
        }
        _ => {
          return Err(
            anyhow!("target must be `Server` or `Builder`")
              .status_code(StatusCode::BAD_REQUEST),
          );
        }
      };
      if let Some(id) = server_id {
        let server = resource::get::<Server>(&id).await?;
        let more = periphery_client(&server)
          .await?
          .request(periphery_client::api::ListSecrets {})
          .await
          .with_context(|| {
            format!(
              "failed to get secrets from server {}",
              server.name
            )
          })?;
        secrets.extend(more);
      }
    }

    let mut secrets = secrets.into_iter().collect::<Vec<_>>();
    secrets.sort();

    Ok(secrets)
  }
}

//

impl Resolve<ReadArgs> for ListGitProvidersFromConfig {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListGitProvidersFromConfigResponse> {
    let mut providers = core_config().git_providers.clone();

    if let Some(target) = self.target {
      match target {
        ResourceTarget::Server(id) => {
          merge_git_providers_for_server(&mut providers, &id).await?;
        }
        ResourceTarget::Builder(id) => {
          match resource::get::<Builder>(&id).await?.config {
            BuilderConfig::Url(_) => {}
            BuilderConfig::Server(config) => {
              if let Some(server_id) = config.server_ids.first() {
                merge_git_providers_for_server(
                  &mut providers,
                  server_id,
                )
                .await?;
              }
            }
            BuilderConfig::Aws(config) => {
              merge_git_providers(
                &mut providers,
                config.git_providers,
              );
            }
          }
        }
        _ => {
          return Err(
            anyhow!("target must be `Server` or `Builder`")
              .status_code(StatusCode::BAD_REQUEST),
          );
        }
      }
    }

    let (builds, repos, syncs) = tokio::try_join!(
      resource::list_full_for_user::<Build>(
        Default::default(),
        None,
        None,
        user,
        PermissionLevel::Read.into(),
        &[]
      ),
      resource::list_full_for_user::<Repo>(
        Default::default(),
        None,
        None,
        user,
        PermissionLevel::Read.into(),
        &[]
      ),
      resource::list_full_for_user::<ResourceSync>(
        Default::default(),
        None,
        None,
        user,
        PermissionLevel::Read.into(),
        &[]
      ),
    )?;

    for build in builds {
      if !providers
        .iter()
        .any(|provider| provider.domain == build.config.git_provider)
      {
        providers.push(GitProvider {
          domain: build.config.git_provider,
          https: build.config.git_https,
          accounts: Default::default(),
        });
      }
    }
    for repo in repos {
      if !providers
        .iter()
        .any(|provider| provider.domain == repo.config.git_provider)
      {
        providers.push(GitProvider {
          domain: repo.config.git_provider,
          https: repo.config.git_https,
          accounts: Default::default(),
        });
      }
    }
    for sync in syncs {
      if !providers
        .iter()
        .any(|provider| provider.domain == sync.config.git_provider)
      {
        providers.push(GitProvider {
          domain: sync.config.git_provider,
          https: sync.config.git_https,
          accounts: Default::default(),
        });
      }
    }

    providers.sort();

    Ok(providers)
  }
}

//

impl Resolve<ReadArgs> for ListImageRegistriesFromConfig {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> mogh_error::Result<ListImageRegistriesFromConfigResponse> {
    let mut registries = core_config().image_registries.clone();

    if let Some(target) = self.target {
      match target {
        ResourceTarget::Server(id) => {
          merge_image_registries_for_server(&mut registries, &id)
            .await?;
        }
        ResourceTarget::Builder(id) => {
          match resource::get::<Builder>(&id).await?.config {
            BuilderConfig::Url(_) => {}
            BuilderConfig::Server(config) => {
              if let Some(server_id) = config.server_ids.first() {
                merge_image_registries_for_server(
                  &mut registries,
                  server_id,
                )
                .await?;
              }
            }
            BuilderConfig::Aws(config) => {
              merge_image_registries(
                &mut registries,
                config.image_registries,
              );
            }
          }
        }
        _ => {
          return Err(
            anyhow!("target must be `Server` or `Builder`").into(),
          );
        }
      }
    }

    registries.sort();

    Ok(registries)
  }
}

async fn merge_git_providers_for_server(
  providers: &mut Vec<GitProvider>,
  server_id: &str,
) -> mogh_error::Result<()> {
  let server = resource::get::<Server>(server_id).await?;
  let more = periphery_client(&server)
    .await?
    .request(periphery_client::api::ListGitProviders {})
    .await
    .with_context(|| {
      format!(
        "failed to get git providers from server {}",
        server.name
      )
    })?;
  merge_git_providers(providers, more);
  Ok(())
}

fn merge_git_providers(
  providers: &mut Vec<GitProvider>,
  more: Vec<GitProvider>,
) {
  for incoming_provider in more {
    if let Some(provider) = providers
      .iter_mut()
      .find(|provider| provider.domain == incoming_provider.domain)
    {
      for account in incoming_provider.accounts {
        if !provider.accounts.contains(&account) {
          provider.accounts.push(account);
        }
      }
    } else {
      providers.push(incoming_provider);
    }
  }
}

async fn merge_image_registries_for_server(
  registries: &mut Vec<ImageRegistry>,
  server_id: &str,
) -> mogh_error::Result<()> {
  let server = resource::get::<Server>(server_id).await?;
  let more = periphery_client(&server)
    .await?
    .request(periphery_client::api::ListImageRegistries {})
    .await
    .with_context(|| {
      format!(
        "failed to get docker registries from server {}",
        server.name
      )
    })?;
  merge_image_registries(registries, more);
  Ok(())
}

fn merge_image_registries(
  registries: &mut Vec<ImageRegistry>,
  more: Vec<ImageRegistry>,
) {
  for incoming_registry in more {
    if let Some(registry) = registries
      .iter_mut()
      .find(|registry| registry.domain == incoming_registry.domain)
    {
      for account in incoming_registry.accounts {
        if !registry.accounts.contains(&account) {
          registry.accounts.push(account);
        }
      }
    } else {
      registries.push(incoming_registry);
    }
  }
}
