use std::{sync::OnceLock, time::Duration};

use anyhow::Context;
use reqwest::StatusCode;
use resolver_api::HasResponse;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

pub mod api;

mod terminal;

fn periphery_http_client() -> &'static reqwest::Client {
  static PERIPHERY_HTTP_CLIENT: OnceLock<reqwest::Client> =
    OnceLock::new();
  PERIPHERY_HTTP_CLIENT.get_or_init(|| {
    reqwest::Client::builder()
      // Use to allow communication with Periphery self-signed certs.
      .danger_accept_invalid_certs(true)
      .build()
      .expect("Failed to build Periphery http client")
  })
}

pub struct PeripheryClient {
  address: String,
  passkey: String,
  timeout: Duration,
}

impl PeripheryClient {
  pub fn new(
    address: impl Into<String>,
    passkey: impl Into<String>,
    timeout: impl Into<Duration>,
  ) -> PeripheryClient {
    PeripheryClient {
      address: address.into(),
      passkey: passkey.into(),
      timeout: timeout.into(),
    }
  }

  // tracing will skip self, to avoid including passkey in traces
  #[tracing::instrument(
    name = "PeripheryRequest",
    level = "debug",
    skip(self)
  )]
  pub async fn request<T>(
    &self,
    request: T,
  ) -> anyhow::Result<T::Response>
  where
    T: std::fmt::Debug + Serialize + HasResponse,
    T::Response: DeserializeOwned,
  {
    tracing::debug!("running health check");
    self.health_check().await?;
    tracing::debug!("health check passed. running inner request");
    self.request_inner(request, None).await
  }

  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn health_check(&self) -> anyhow::Result<()> {
    self
      .request_inner(api::GetHealth {}, Some(self.timeout))
      .await?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", skip(self))]
  async fn request_inner<T>(
    &self,
    request: T,
    timeout: Option<Duration>,
  ) -> anyhow::Result<T::Response>
  where
    T: std::fmt::Debug + Serialize + HasResponse,
    T::Response: DeserializeOwned,
  {
    let req_type = T::req_type();
    tracing::trace!(
      "sending request | type: {req_type} | body: {request:?}"
    );
    let mut req = periphery_http_client()
      .post(&self.address)
      .json(&json!({
        "type": req_type,
        "params": request
      }))
      .header("authorization", &self.passkey);
    if let Some(timeout) = timeout {
      req = req.timeout(timeout);
    }
    let res =
      req.send().await.context("failed at request to periphery")?;
    let status = res.status();
    tracing::debug!(
      "got response | type: {req_type} | {status} | response: {res:?}",
    );
    if status == StatusCode::OK {
      tracing::debug!("response ok, deserializing");
      res.json().await.with_context(|| format!(
        "failed to parse response to json | type: {req_type} | request: {request:?}"
      ))
    } else {
      tracing::debug!("response is non-200");

      let text = res
        .text()
        .await
        .context("failed to convert response to text")?;

      tracing::debug!("got response text, deserializing error");

      let error = serror::deserialize_error(text).context(status);

      Err(error)
    }
  }
}
