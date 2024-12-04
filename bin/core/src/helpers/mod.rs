use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context};
use futures::future::join_all;
use komodo_client::{
  api::write::{CreateBuilder, CreateServer},
  entities::{
    builder::{PartialBuilderConfig, PartialServerBuilderConfig},
    komodo_timestamp,
    permission::{Permission, PermissionLevel, UserTarget},
    server::{PartialServerConfig, Server},
    sync::ResourceSync,
    update::Log,
    user::{system_user, User},
    ResourceTarget,
  },
};
use mongo_indexed::Document;
use mungos::{
  find::find_collect,
  mongodb::bson::{doc, oid::ObjectId, to_document, Bson},
};
use periphery_client::PeripheryClient;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use resolver_api::Resolve;

use crate::{
  api::write::WriteArgs, config::core_config, resource,
  state::db_client,
};

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
  thread_rng()
    .sample_iter(&Alphanumeric)
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
    &core_config().passkey,
    Duration::from_secs(server.config.timeout_seconds as u64),
  );

  Ok(client)
}

#[instrument]
pub async fn create_permission<T>(
  user: &User,
  target: T,
  level: PermissionLevel,
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

pub async fn startup_cleanup() {
  tokio::join!(
    startup_in_progress_update_cleanup(),
    startup_open_alert_cleanup(),
  );
}

/// Run on startup, as no updates should be in progress on startup
async fn startup_in_progress_update_cleanup() {
  let log = Log::error(
    "Komodo shutdown",
    String::from("Komodo shutdown during execution. If this is a build, the builder may not have been terminated.")
  );
  // This static log won't fail to serialize, unwrap ok.
  let log = to_document(&log).unwrap();
  if let Err(e) = db_client()
    .updates
    .update_many(
      doc! { "status": "InProgress" },
      doc! {
        "$set": {
          "status": "Complete",
          "success": false,
        },
        "$push": {
          "logs": log
        }
      },
    )
    .await
  {
    error!("failed to cleanup in progress updates on startup | {e:#}")
  }
}

/// Run on startup, ensure open alerts pointing to invalid resources are closed.
async fn startup_open_alert_cleanup() {
  let db = db_client();
  let Ok(alerts) =
    find_collect(&db.alerts, doc! { "resolved": false }, None)
      .await
      .inspect_err(|e| {
        error!(
          "failed to list all alerts for startup open alert cleanup | {e:?}"
        )
      })
  else {
    return;
  };
  let futures = alerts.into_iter().map(|alert| async move {
    match alert.target {
      ResourceTarget::Server(id) => {
        resource::get::<Server>(&id)
          .await
          .is_err()
          .then(|| ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok()).flatten()
      }
      ResourceTarget::ResourceSync(id) => {
        resource::get::<ResourceSync>(&id)
          .await
          .is_err()
          .then(|| ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok()).flatten()
      }
      // No other resources should have open alerts.
      _ => ObjectId::from_str(&alert.id).inspect_err(|e| warn!("failed to clean up alert - id is invalid ObjectId | {e:?}")).ok(),
    }
  });
  let to_update_ids = join_all(futures)
    .await
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
  if let Err(e) = db
    .alerts
    .update_many(
      doc! { "_id": { "$in": to_update_ids } },
      doc! { "$set": {
        "resolved": true,
        "resolved_ts": komodo_timestamp()
      } },
    )
    .await
  {
    error!(
      "failed to clean up invalid open alerts on startup | {e:#}"
    )
  }
}

/// Ensures a default server / builder exists with the defined address
pub async fn ensure_first_server_and_builder() {
  let first_server = &core_config().first_server;
  if first_server.is_empty() {
    return;
  }
  let db = db_client();
  let Ok(server) = db
    .servers
    .find_one(Document::new())
    .await
    .inspect_err(|e| error!("Failed to initialize 'first_server'. Failed to query db. {e:?}"))
  else {
    return;
  };
  let server = if let Some(server) = server {
    server
  } else {
    match (CreateServer {
      name: format!("server-{}", random_string(5)),
      config: PartialServerConfig {
        address: Some(first_server.to_string()),
        enabled: Some(true),
        ..Default::default()
      },
    })
    .resolve(&WriteArgs {
      user: system_user().to_owned(),
    })
    .await
    {
      Ok(server) => server,
      Err(e) => {
        error!("Failed to initialize 'first_server'. Failed to CreateServer. {:#}", e.error);
        return;
      }
    }
  };
  let Ok(None) = db.builders
    .find_one(Document::new()).await
    .inspect_err(|e| error!("Failed to initialize 'first_builder' | Failed to query db | {e:?}")) else {
      return;
    };
  if let Err(e) = (CreateBuilder {
    name: String::from("local"),
    config: PartialBuilderConfig::Server(
      PartialServerBuilderConfig {
        server_id: Some(server.id),
      },
    ),
  })
  .resolve(&WriteArgs {
    user: system_user().to_owned(),
  })
  .await
  {
    error!("Failed to initialize 'first_builder' | Failed to CreateBuilder | {:#}", e.error);
  }
}
