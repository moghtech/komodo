use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use komodo_client::entities::config::core::{
  CoreConfig, OauthCredentials,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;

use crate::{
  auth::STATE_PREFIX_LENGTH, config::core_config,
  helpers::random_string,
};

pub fn github_oauth_client() -> &'static Option<GithubOauthClient> {
  static GITHUB_OAUTH_CLIENT: OnceLock<Option<GithubOauthClient>> =
    OnceLock::new();
  GITHUB_OAUTH_CLIENT
    .get_or_init(|| GithubOauthClient::new(core_config()))
}

pub struct GithubOauthClient {
  http: reqwest::Client,
  client_id: String,
  client_secret: String,
  redirect_uri: String,
  scopes: String,
  states: Mutex<Vec<String>>,
  user_agent: String,
}

impl GithubOauthClient {
  pub fn new(
    CoreConfig {
      github_oauth:
        OauthCredentials {
          enabled,
          id,
          secret,
        },
      host,
      ..
    }: &CoreConfig,
  ) -> Option<GithubOauthClient> {
    if !enabled {
      return None;
    }
    if host.is_empty() {
      warn!(
        "github oauth is enabled, but 'config.host' is not configured"
      );
      return None;
    }
    if id.is_empty() {
      warn!(
        "github oauth is enabled, but 'config.github_oauth.id' is not configured"
      );
      return None;
    }
    if secret.is_empty() {
      warn!(
        "github oauth is enabled, but 'config.github_oauth.secret' is not configured"
      );
      return None;
    }
    GithubOauthClient {
      http: reqwest::Client::new(),
      client_id: id.clone(),
      client_secret: secret.clone(),
      redirect_uri: format!("{host}/auth/github/callback"),
      user_agent: Default::default(),
      scopes: Default::default(),
      states: Default::default(),
    }
    .into()
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn get_login_redirect_url(
    &self,
    redirect: Option<String>,
  ) -> String {
    let state_prefix = random_string(STATE_PREFIX_LENGTH);
    let state = match redirect {
      Some(redirect) => format!("{state_prefix}{redirect}"),
      None => state_prefix,
    };
    let redirect_url = format!(
      "https://github.com/login/oauth/authorize?state={state}&client_id={}&redirect_uri={}&scope={}",
      self.client_id, self.redirect_uri, self.scopes
    );
    let mut states = self.states.lock().await;
    states.push(state);
    redirect_url
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn check_state(&self, state: &str) -> bool {
    let mut contained = false;
    self.states.lock().await.retain(|s| {
      if s.as_str() == state {
        contained = true;
        false
      } else {
        true
      }
    });
    contained
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn get_access_token(
    &self,
    code: &str,
  ) -> anyhow::Result<AccessTokenResponse> {
    self
      .post::<(), _>(
        "https://github.com/login/oauth/access_token",
        &[
          ("client_id", self.client_id.as_str()),
          ("client_secret", self.client_secret.as_str()),
          ("redirect_uri", self.redirect_uri.as_str()),
          ("code", code),
        ],
        None,
        None,
      )
      .await
      .context("failed to get github access token using code")
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn get_github_user(
    &self,
    token: &str,
  ) -> anyhow::Result<GithubUserResponse> {
    self
      .get("https://api.github.com/user", &[], Some(token))
      .await
      .context("failed to get github user using access token")
  }

  #[instrument(level = "debug", skip(self))]
  async fn get<R: DeserializeOwned>(
    &self,
    endpoint: &str,
    query: &[(&str, &str)],
    bearer_token: Option<&str>,
  ) -> anyhow::Result<R> {
    let mut req = self
      .http
      .get(endpoint)
      .query(query)
      .header("User-Agent", &self.user_agent);

    if let Some(bearer_token) = bearer_token {
      req =
        req.header("Authorization", format!("Bearer {bearer_token}"));
    }

    let res = req.send().await.context("failed to reach github")?;

    let status = res.status();

    if status == StatusCode::OK {
      let body = res
        .json()
        .await
        .context("failed to parse body into expected type")?;
      Ok(body)
    } else {
      let text = res.text().await.context(format!(
        "status: {status} | failed to get response text"
      ))?;
      Err(anyhow!("status: {status} | text: {text}"))
    }
  }

  async fn post<B: Serialize, R: DeserializeOwned>(
    &self,
    endpoint: &str,
    query: &[(&str, &str)],
    body: Option<&B>,
    bearer_token: Option<&str>,
  ) -> anyhow::Result<R> {
    let mut req = self
      .http
      .post(endpoint)
      .query(query)
      .header("Accept", "application/json")
      .header("User-Agent", &self.user_agent);

    if let Some(body) = body {
      req = req.json(body);
    }

    if let Some(bearer_token) = bearer_token {
      req =
        req.header("Authorization", format!("Bearer {bearer_token}"));
    }

    let res = req.send().await.context("failed to reach github")?;

    let status = res.status();

    if status == StatusCode::OK {
      let body = res
        .json()
        .await
        .context("failed to parse POST body into expected type")?;
      Ok(body)
    } else {
      let text = res.text().await.with_context(|| format!(
        "method: POST | status: {status} | failed to get response text"
      ))?;
      Err(anyhow!("method: POST | status: {status} | text: {text}"))
    }
  }
}

#[derive(Deserialize)]
pub struct AccessTokenResponse {
  pub access_token: String,
  // pub scope: String,
  // pub token_type: String,
}

#[derive(Deserialize)]
pub struct GithubUserResponse {
  pub login: String,
  pub id: u128,
  pub avatar_url: String,
  // pub email: Option<String>,
}
