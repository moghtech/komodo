use std::time::Duration;

use anyhow::{Context, anyhow};
use indexmap::IndexSet;
use komodo_client::entities::{
  ResourceTarget,
  permission::{
    Permission, PermissionLevel, SpecificPermission, UserTarget,
  },
  server::Server,
  user::User,
};
use mongo_indexed::Document;
use mungos::mongodb::bson::{Bson, doc};
use periphery_client::PeripheryClient;
use rand::Rng;

use crate::{config::core_config, state::db_client};

pub mod action_state;
pub mod builder;
pub mod cache;
pub mod channel;
pub mod interpolate;
pub mod procedure;
pub mod prune;
pub mod query;
pub mod update;

// pub mod resource;

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

pub fn random_string(length: usize) -> String {
  rand::rng()
    .sample_iter(&rand::distr::Alphanumeric)
    .take(length)
    .map(char::from)
    .collect()
}

const BCRYPT_COST: u32 = 10;
pub fn hash_password<P>(password: P) -> anyhow::Result<String>
where
  P: AsRef<[u8]>,
{
  bcrypt::hash(password, BCRYPT_COST)
    .context("failed to hash password")
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

pub fn periphery_client(
  server: &Server,
) -> anyhow::Result<PeripheryClient> {
  if !server.config.enabled {
    return Err(anyhow!("server not enabled"));
  }

  let client = PeripheryClient::new(
    &server.config.address,
    if server.config.passkey.is_empty() {
      &core_config().passkey
    } else {
      &server.config.passkey
    },
    Duration::from_secs(server.config.timeout_seconds as u64),
  );

  Ok(client)
}

#[instrument]
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
