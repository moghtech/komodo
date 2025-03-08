use anyhow::{Context, anyhow};
use axum::http::{HeaderName, HeaderValue};
use reqwest::{RequestBuilder, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use super::{
  common::{
    HetznerActionResponse, HetznerDatacenterResponse,
    HetznerServerResponse, HetznerVolumeResponse,
  },
  create_server::{CreateServerBody, CreateServerResponse},
  create_volume::{CreateVolumeBody, CreateVolumeResponse},
};

const BASE_URL: &str = "https://api.hetzner.cloud/v1";

pub struct HetznerClient(reqwest::Client);

impl HetznerClient {
  pub fn new(token: &str) -> HetznerClient {
    HetznerClient(
      reqwest::ClientBuilder::new()
        .default_headers(
          [(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {token}"))
              .unwrap(),
          )]
          .into_iter()
          .collect(),
        )
        .build()
        .context("failed to build Hetzner request client")
        .unwrap(),
    )
  }

  pub async fn get_server(
    &self,
    id: i64,
  ) -> anyhow::Result<HetznerServerResponse> {
    self.get(&format!("/servers/{id}")).await
  }

  pub async fn create_server(
    &self,
    body: &CreateServerBody,
  ) -> anyhow::Result<CreateServerResponse> {
    self.post("/servers", body).await
  }

  #[allow(unused)]
  pub async fn delete_server(
    &self,
    id: i64,
  ) -> anyhow::Result<HetznerActionResponse> {
    self.delete(&format!("/servers/{id}")).await
  }

  pub async fn get_volume(
    &self,
    id: i64,
  ) -> anyhow::Result<HetznerVolumeResponse> {
    self.get(&format!("/volumes/{id}")).await
  }

  pub async fn create_volume(
    &self,
    body: &CreateVolumeBody,
  ) -> anyhow::Result<CreateVolumeResponse> {
    self.post("/volumes", body).await
  }

  #[allow(unused)]
  pub async fn delete_volume(&self, id: i64) -> anyhow::Result<()> {
    let res = self
      .0
      .delete(format!("{BASE_URL}/volumes/{id}"))
      .send()
      .await
      .context("failed at request to delete volume")?;

    let status = res.status();

    if status == StatusCode::NO_CONTENT {
      Ok(())
    } else {
      let text = res
        .text()
        .await
        .context("failed to get response body as text")?;
      Err(anyhow!("{status} | {text}"))
    }
  }

  #[allow(unused)]
  pub async fn list_datacenters(
    &self,
  ) -> anyhow::Result<HetznerDatacenterResponse> {
    self.get("/datacenters").await
  }

  async fn get<Res: DeserializeOwned>(
    &self,
    path: &str,
  ) -> anyhow::Result<Res> {
    let req = self.0.get(format!("{BASE_URL}{path}"));
    handle_req(req).await.with_context(|| {
      format!("failed at GET request to Hetzner | path: {path}")
    })
  }

  async fn post<Body: Serialize, Res: DeserializeOwned>(
    &self,
    path: &str,
    body: &Body,
  ) -> anyhow::Result<Res> {
    let req = self.0.post(format!("{BASE_URL}{path}")).json(&body);
    handle_req(req).await.with_context(|| {
      format!("failed at POST request to Hetzner | path: {path}")
    })
  }

  #[allow(unused)]
  async fn delete<Res: DeserializeOwned>(
    &self,
    path: &str,
  ) -> anyhow::Result<Res> {
    let req = self.0.delete(format!("{BASE_URL}{path}"));
    handle_req(req).await.with_context(|| {
      format!("failed at DELETE request to Hetzner | path: {path}")
    })
  }
}

async fn handle_req<Res: DeserializeOwned>(
  req: RequestBuilder,
) -> anyhow::Result<Res> {
  let res = req.send().await?;

  let status = res.status();

  if status.is_success() {
    res.json().await.context("failed to parse response to json")
  } else {
    let text = res
      .text()
      .await
      .context("failed to get response body as text")?;
    if let Ok(json_error) =
      serde_json::from_str::<serde_json::Value>(&text)
    {
      return Err(anyhow!("{status} | {json_error:?}"));
    }
    Err(anyhow!("{status} | {text}"))
  }
}
