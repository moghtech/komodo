use std::sync::OnceLock;

use anyhow::{Context, anyhow};
use jsonwebtoken::{DecodingKey, Validation, decode};
use komodo_client::entities::config::core::{
  CoreConfig, OauthCredentials,
};
use reqwest::StatusCode;
use serde::{Deserialize, de::DeserializeOwned};
use tokio::sync::Mutex;

use crate::{
  auth::STATE_PREFIX_LENGTH, config::core_config,
  helpers::random_string,
};

pub fn google_oauth_client() -> &'static Option<GoogleOauthClient> {
  static GOOGLE_OAUTH_CLIENT: OnceLock<Option<GoogleOauthClient>> =
    OnceLock::new();
  GOOGLE_OAUTH_CLIENT
    .get_or_init(|| GoogleOauthClient::new(core_config()))
}

pub struct GoogleOauthClient {
  http: reqwest::Client,
  client_id: String,
  client_secret: String,
  redirect_uri: String,
  scopes: String,
  states: Mutex<Vec<String>>,
  user_agent: String,
}

impl GoogleOauthClient {
  pub fn new(
    CoreConfig {
      google_oauth:
        OauthCredentials {
          enabled,
          id,
          secret,
        },
      host,
      ..
    }: &CoreConfig,
  ) -> Option<GoogleOauthClient> {
    if !enabled {
      return None;
    }
    if host.is_empty() {
      warn!(
        "google oauth is enabled, but 'config.host' is not configured"
      );
      return None;
    }
    if id.is_empty() {
      warn!(
        "google oauth is enabled, but 'config.google_oauth.id' is not configured"
      );
      return None;
    }
    if secret.is_empty() {
      warn!(
        "google oauth is enabled, but 'config.google_oauth.secret' is not configured"
      );
      return None;
    }
    let scopes = urlencoding::encode(
      &[
        "https://www.googleapis.com/auth/userinfo.profile",
        "https://www.googleapis.com/auth/userinfo.email",
      ]
      .join(" "),
    )
    .to_string();
    GoogleOauthClient {
      http: Default::default(),
      client_id: id.clone(),
      client_secret: secret.clone(),
      redirect_uri: format!("{host}/auth/google/callback"),
      user_agent: String::from("komodo"),
      states: Default::default(),
      scopes,
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
      "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&state={state}&client_id={}&redirect_uri={}&scope={}",
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
      .post::<_>(
        "https://oauth2.googleapis.com/token",
        &[
          ("client_id", self.client_id.as_str()),
          ("client_secret", self.client_secret.as_str()),
          ("redirect_uri", self.redirect_uri.as_str()),
          ("code", code),
          ("grant_type", "authorization_code"),
        ],
        None,
      )
      .await
      .context("failed to get google access token using code")
  }

  #[instrument(level = "debug", skip(self))]
  pub fn get_google_user(
    &self,
    id_token: &str,
  ) -> anyhow::Result<GoogleUser> {
    let mut v = Validation::new(Default::default());
    v.insecure_disable_signature_validation();
    v.validate_aud = false;
    let res = decode::<GoogleUser>(
      id_token,
      &DecodingKey::from_secret(b""),
      &v,
    )
    .context("failed to decode google id token")?;
    Ok(res.claims)
  }

  #[instrument(level = "debug", skip(self))]
  async fn post<R: DeserializeOwned>(
    &self,
    endpoint: &str,
    body: &[(&str, &str)],
    bearer_token: Option<&str>,
  ) -> anyhow::Result<R> {
    let mut req = self
      .http
      .post(endpoint)
      .form(body)
      .header("Accept", "application/json")
      .header("User-Agent", &self.user_agent);

    if let Some(bearer_token) = bearer_token {
      req =
        req.header("Authorization", format!("Bearer {bearer_token}"));
    }

    let res = req.send().await.context("failed to reach google")?;

    let status = res.status();

    if status == StatusCode::OK {
      let body = res
        .json()
        .await
        .context("failed to parse POST body into expected type")?;
      Ok(body)
    } else {
      let text = res.text().await.context(format!(
        "method: POST | status: {status} | failed to get response text"
      ))?;
      Err(anyhow!("method: POST | status: {status} | text: {text}"))
    }
  }
}

#[derive(Deserialize)]
pub struct AccessTokenResponse {
  // pub access_token: String,
  pub id_token: String,
  // pub scope: String,
  // pub token_type: String,
}

#[derive(Deserialize, Clone)]
pub struct GoogleUser {
  #[serde(rename = "sub")]
  pub id: String,
  pub email: String,
  #[serde(default)]
  pub picture: String,
}
