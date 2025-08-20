//! # Komodo
//! *A system to build and deploy software across many servers*. [**https://komo.do**](https://komo.do)
//!
//! This is a client library for the Komodo Core API.
//! It contains:
//! - Definitions for the application [api] and [entities].
//! - A [client][KomodoClient] to interact with the Komodo Core API.
//! - Information on configuring Komodo [Core][entities::config::core] and [Periphery][entities::config::periphery].
//!
//! ## Client Configuration
//!
//! The client includes a convenenience method to parse the Komodo API url and credentials from the environment:
//! - `KOMODO_ADDRESS`
//! - `KOMODO_API_KEY`
//! - `KOMODO_API_SECRET`
//!
//! ## Client Example
//! ```text
//! dotenvy::dotenv().ok();
//!
//! let client = KomodoClient::new_from_env()?;
//!
//! // Get all the deployments
//! let deployments = client.read(ListDeployments::default()).await?;
//!
//! println!("{deployments:#?}");
//!
//! let update = client.execute(RunBuild { build: "test-build".to_string() }).await?:
//! ```

use std::{sync::OnceLock, time::Duration};

use anyhow::Context;
use api::read::GetVersion;
use serde::Deserialize;

pub mod api;
pub mod busy;
pub mod deserializers;
pub mod entities;
pub mod parsers;
pub mod terminal;
pub mod ws;

mod request;

/// &'static KomodoClient initialized from environment.
pub fn komodo_client() -> &'static KomodoClient {
  static KOMODO_CLIENT: OnceLock<KomodoClient> = OnceLock::new();
  KOMODO_CLIENT.get_or_init(|| {
    KomodoClient::new_from_env()
      .context("Missing KOMODO_ADDRESS, KOMODO_API_KEY, KOMODO_API_SECRET from env")
      .unwrap()
  })
}

/// Default environment variables for the [KomodoClient].
#[derive(Deserialize)]
pub struct KomodoEnv {
  /// KOMODO_ADDRESS
  pub komodo_address: String,
  /// KOMODO_API_KEY
  pub komodo_api_key: String,
  /// KOMODO_API_SECRET
  pub komodo_api_secret: String,
}

/// Client to interface with [Komodo](https://komo.do/docs/api#rust-client)
#[derive(Clone)]
pub struct KomodoClient {
  #[cfg(not(feature = "blocking"))]
  reqwest: reqwest::Client,
  #[cfg(feature = "blocking")]
  reqwest: reqwest::blocking::Client,
  address: String,
  key: String,
  secret: String,
}

impl KomodoClient {
  /// Initializes KomodoClient, including a health check.
  pub fn new(
    address: impl Into<String>,
    key: impl Into<String>,
    secret: impl Into<String>,
  ) -> KomodoClient {
    KomodoClient {
      reqwest: Default::default(),
      address: address.into(),
      key: key.into(),
      secret: secret.into(),
    }
  }

  /// Initializes KomodoClient from environment: [KomodoEnv]
  pub fn new_from_env() -> anyhow::Result<KomodoClient> {
    let KomodoEnv {
      komodo_address,
      komodo_api_key,
      komodo_api_secret,
    } = envy::from_env()
      .context("failed to parse environment for komodo client")?;
    Ok(KomodoClient::new(
      komodo_address,
      komodo_api_key,
      komodo_api_secret,
    ))
  }

  /// Add a healthcheck in the initialization pipeline:
  ///
  /// ```text
  /// let komodo = KomodoClient::new_from_env()?
  ///   .with_healthcheck().await?;
  /// ```
  #[cfg(not(feature = "blocking"))]
  pub async fn with_healthcheck(self) -> anyhow::Result<Self> {
    self.health_check().await?;
    Ok(self)
  }

  /// Add a healthcheck in the initialization pipeline:
  ///
  /// ```text
  /// let komodo = KomodoClient::new_from_env()?
  ///   .with_healthcheck().await?;
  /// ```
  #[cfg(feature = "blocking")]
  pub fn with_healthcheck(self) -> anyhow::Result<Self> {
    self.health_check()?;
    Ok(self)
  }

  /// Get the Core version.
  #[cfg(not(feature = "blocking"))]
  pub async fn core_version(&self) -> anyhow::Result<String> {
    self.read(GetVersion {}).await.map(|r| r.version)
  }

  /// Get the Core version.
  #[cfg(feature = "blocking")]
  pub fn core_version(&self) -> anyhow::Result<String> {
    self.read(GetVersion {}).map(|r| r.version)
  }

  /// Send a health check.
  #[cfg(not(feature = "blocking"))]
  pub async fn health_check(&self) -> anyhow::Result<()> {
    self.read(GetVersion {}).await.map(|_| ())
  }

  /// Send a health check.
  #[cfg(feature = "blocking")]
  pub fn health_check(&self) -> anyhow::Result<()> {
    self.read(GetVersion {}).map(|_| ())
  }

  /// Use a custom reqwest client.
  #[cfg(not(feature = "blocking"))]
  pub fn set_reqwest(mut self, reqwest: reqwest::Client) -> Self {
    self.reqwest = reqwest;
    self
  }

  /// Use a custom reqwest client.
  #[cfg(feature = "blocking")]
  pub fn set_reqwest(
    mut self,
    reqwest: reqwest::blocking::Client,
  ) -> Self {
    self.reqwest = reqwest;
    self
  }

  /// Poll an [Update][entities::update::Update] (returned by the `execute` calls) until the
  /// [UpdateStatus][entities::update::UpdateStatus] is `Complete`, and then return it.
  #[cfg(not(feature = "blocking"))]
  pub async fn poll_update_until_complete(
    &self,
    update_id: impl Into<String>,
  ) -> anyhow::Result<entities::update::Update> {
    let update_id = update_id.into();
    loop {
      let update = self
        .read(api::read::GetUpdate {
          id: update_id.clone(),
        })
        .await?;
      if update.status == entities::update::UpdateStatus::Complete {
        return Ok(update);
      }
      tokio::time::sleep(Duration::from_millis(500)).await;
    }
  }

  /// Poll an [Update][entities::update::Update] (returned by the `execute` calls) until the
  /// [UpdateStatus][entities::update::UpdateStatus] is `Complete`, and then return it.
  #[cfg(feature = "blocking")]
  pub fn poll_update_until_complete(
    &self,
    update_id: impl Into<String>,
  ) -> anyhow::Result<entities::update::Update> {
    let update_id = update_id.into();
    loop {
      let update = self.read(api::read::GetUpdate {
        id: update_id.clone(),
      })?;
      if update.status == entities::update::UpdateStatus::Complete {
        return Ok(update);
      }
    }
  }
}
