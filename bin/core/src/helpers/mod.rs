use std::fmt::Write;

use anyhow::{Context, anyhow};
use database::mongo_indexed::Document;
use database::mungos::mongodb::bson::{Bson, doc};
use indexmap::IndexSet;
use komodo_client::entities::SwarmOrServer;
use komodo_client::entities::{
  ResourceTarget,
  build::Build,
  permission::{
    Permission, PermissionLevel, SpecificPermission, UserTarget,
  },
  repo::Repo,
  server::Server,
  stack::Stack,
  user::User,
};
use resolver_api::HasResponse;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::helpers::swarm::swarm_request;
use crate::{
  config::core_config, connection::PeripheryConnectionArgs,
  periphery::PeripheryClient, state::db_client,
};

pub mod action_state;
pub mod all_resources;
pub mod builder;
pub mod channel;
pub mod maintenance;
pub mod matcher;
pub mod procedure;
pub mod prune;
pub mod query;
pub mod swarm;
pub mod terminal;
pub mod update;
pub mod validations;

pub fn empty_or_only_spaces(word: &str) -> bool {
  if word.is_empty() {
    return true;
  }
  for char in word.chars() {
    if char != ' ' {
      return false;
    }
  }
  true
}

/// First checks db for token, then checks core config.
/// Only errors if db call errors.
/// Returns (token, use_https)
pub async fn git_token(
  provider_domain: &str,
  account_username: &str,
  mut on_https_found: impl FnMut(bool),
) -> anyhow::Result<Option<String>> {
  if provider_domain.is_empty() || account_username.is_empty() {
    return Ok(None);
  }
  let db_provider = db_client()
    .git_accounts
    .find_one(doc! { "domain": provider_domain, "username": account_username })
    .await
    .context("failed to query db for git provider accounts")?;
  if let Some(provider) = db_provider {
    on_https_found(provider.https);
    return Ok(Some(provider.token));
  }
  Ok(
    core_config()
      .git_providers
      .iter()
      .find(|provider| provider.domain == provider_domain)
      .and_then(|provider| {
        on_https_found(provider.https);
        provider
          .accounts
          .iter()
          .find(|account| account.username == account_username)
          .map(|account| account.token.clone())
      }),
  )
}

pub async fn stack_git_token(
  stack: &mut Stack,
  repo: Option<&mut Repo>,
) -> anyhow::Result<Option<String>> {
  if let Some(repo) = repo {
    return git_token(
      &repo.config.git_provider,
      &repo.config.git_account,
      |https| repo.config.git_https = https,
    )
    .await
    .with_context(|| {
      format!(
        "Failed to get git token. Stopping run. | {} | {}",
        repo.config.git_provider, repo.config.git_account
      )
    });
  }
  git_token(
    &stack.config.git_provider,
    &stack.config.git_account,
    |https| stack.config.git_https = https,
  )
  .await
  .with_context(|| {
    format!(
      "Failed to get git token. Stopping run. | {} | {}",
      stack.config.git_provider, stack.config.git_account
    )
  })
}

pub async fn build_git_token(
  build: &mut Build,
  repo: Option<&mut Repo>,
) -> anyhow::Result<Option<String>> {
  if let Some(repo) = repo {
    return git_token(
      &repo.config.git_provider,
      &repo.config.git_account,
      |https| repo.config.git_https = https,
    )
    .await
    .with_context(|| {
      format!(
        "Failed to get git token. Stopping run. | {} | {}",
        repo.config.git_provider, repo.config.git_account
      )
    });
  }
  git_token(
    &build.config.git_provider,
    &build.config.git_account,
    |https| build.config.git_https = https,
  )
  .await
  .with_context(|| {
    format!(
      "Failed to get git token. Stopping run. | {} | {}",
      build.config.git_provider, build.config.git_account
    )
  })
}

/// First checks db for token, then checks core config.
/// Only errors if db call errors.
pub async fn registry_token(
  provider_domain: &str,
  account_username: &str,
) -> anyhow::Result<Option<String>> {
  let provider = db_client()
    .registry_accounts
    .find_one(doc! { "domain": provider_domain, "username": account_username })
    .await
    .context("failed to query db for docker registry accounts")?;
  if let Some(provider) = provider {
    return Ok(Some(provider.token));
  }
  Ok(
    core_config()
      .docker_registries
      .iter()
      .find(|provider| provider.domain == provider_domain)
      .and_then(|provider| {
        provider
          .accounts
          .iter()
          .find(|account| account.username == account_username)
          .map(|account| account.token.clone())
      }),
  )
}

//

pub async fn periphery_client(
  server: &Server,
) -> anyhow::Result<PeripheryClient> {
  if !server.config.enabled {
    return Err(anyhow!("server not enabled"));
  }
  PeripheryClient::new(
    PeripheryConnectionArgs::from_server(server),
    server.config.insecure_tls,
  )
  .await
}

#[instrument(
  "CreatePermission",
  skip(user),
  fields(
    operator = user.id,
    username = user.username
  )
)]
pub async fn create_permission<T>(
  user: &User,
  target: T,
  level: PermissionLevel,
  specific: IndexSet<SpecificPermission>,
) where
  T: Into<ResourceTarget> + std::fmt::Debug,
{
  // No need to actually create permissions for admins
  if user.admin {
    return;
  }
  let target: ResourceTarget = target.into();
  if let Err(e) = db_client()
    .permissions
    .insert_one(Permission {
      id: Default::default(),
      user_target: UserTarget::User(user.id.clone()),
      resource_target: target.clone(),
      level,
      specific,
    })
    .await
  {
    error!("failed to create permission for {target:?} | {e:#}");
  };
}

/// Flattens a document only one level deep
///
/// eg `{ config: { label: "yes", thing: { field1: "ok", field2: "ok" } } }` ->
/// `{ "config.label": "yes", "config.thing": { field1: "ok", field2: "ok" } }`
pub fn flatten_document(doc: Document) -> Document {
  let mut target = Document::new();

  for (outer_field, bson) in doc {
    if let Bson::Document(doc) = bson {
      for (inner_field, bson) in doc {
        target.insert(format!("{outer_field}.{inner_field}"), bson);
      }
    } else {
      target.insert(outer_field, bson);
    }
  }

  target
}

pub fn repo_link(
  provider: &str,
  repo: &str,
  branch: &str,
  https: bool,
) -> String {
  let mut res = format!(
    "http{}://{provider}/{repo}",
    if https { "s" } else { "" }
  );
  // Each provider uses a different link format to get to branches.
  // At least can support github for branch aware link.
  if provider == "github.com" {
    let _ = write!(&mut res, "/tree/{branch}");
  }
  res
}

pub async fn swarm_or_server_request<T>(
  swarm_or_server: &SwarmOrServer,
  request: T,
) -> anyhow::Result<T::Response>
where
  T: std::fmt::Debug + Clone + Serialize + HasResponse,
  T::Response: DeserializeOwned,
{
  match swarm_or_server {
    SwarmOrServer::Swarm(swarm) => {
      swarm_request(&swarm.config.server_ids, request).await
    }
    SwarmOrServer::Server(server) => {
      periphery_client(server).await?.request(request).await
    }
  }
}
