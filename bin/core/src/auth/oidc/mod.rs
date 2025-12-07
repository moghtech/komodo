use std::{net::SocketAddr, sync::OnceLock};

use anyhow::{Context, anyhow};
use async_timing_util::{ONE_MIN_MS, unix_timestamp_ms};
use axum::{
  Router,
  extract::{ConnectInfo, Query},
  http::HeaderMap,
  response::Redirect,
  routing::get,
};
use client::oidc_client;
use dashmap::DashMap;
use database::mungos::mongodb::bson::{Document, doc};
use futures_util::TryFutureExt;
use komodo_client::{
  api::auth::JwtOrTwoFactor,
  entities::{
    komodo_timestamp, random_string,
    user::{User, UserConfig},
  },
};
use openidconnect::{
  AccessTokenHash, AuthorizationCode, CsrfToken,
  EmptyAdditionalClaims, Nonce, OAuth2TokenResponse,
  PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
  core::{CoreAuthenticationFlow, CoreGenderClaim},
};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serde::Deserialize;
use serror::{AddStatusCode as _, AddStatusCodeError};

use crate::{
  config::core_config,
  state::{
    auth_rate_limiter, db_client, jwt_client,
    totp_pending_login_cache,
  },
};

use super::RedirectQuery;

pub mod client;

static APP_USER_AGENT: &str =
  concat!("Komodo/", env!("CARGO_PKG_VERSION"),);

fn reqwest_client() -> &'static reqwest::Client {
  static REQWEST: OnceLock<reqwest::Client> = OnceLock::new();
  REQWEST.get_or_init(|| {
    reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .user_agent(APP_USER_AGENT)
      .build()
      .expect("Invalid OIDC reqwest client")
  })
}

/// CSRF tokens can only be used once from the callback,
/// and must be used within this timeframe
const CSRF_VALID_FOR_MS: i64 = 120_000; // 2 minutes for user to log in.

type RedirectUrl = Option<String>;
/// Maps the csrf secrets to other information added in the "login" method (before auth provider redirect).
/// This information is retrieved in the "callback" method (after auth provider redirect).
type VerifierMap =
  DashMap<String, (PkceCodeVerifier, Nonce, RedirectUrl, i64)>;
fn verifier_tokens() -> &'static VerifierMap {
  static VERIFIERS: OnceLock<VerifierMap> = OnceLock::new();
  VERIFIERS.get_or_init(Default::default)
}

pub fn router() -> Router {
  Router::new()
    .route(
      "/login",
      get(|query| async {
        login(query).await.status_code(StatusCode::UNAUTHORIZED)
      }),
    )
    .route(
      "/callback",
      get(
        |query,
         headers: HeaderMap,
         ConnectInfo(info): ConnectInfo<SocketAddr>| async move {
          callback(query)
            .map_err(|e| e.status_code(StatusCode::UNAUTHORIZED))
            .with_failure_rate_limit_using_headers(
              auth_rate_limiter(),
              &headers,
              Some(info.ip()),
            )
            .await
        },
      ),
    )
}

async fn login(
  Query(RedirectQuery { redirect }): Query<RedirectQuery>,
) -> anyhow::Result<Redirect> {
  let client = oidc_client().load();
  let client =
    client.as_ref().context("OIDC Client not configured")?;

  let (pkce_challenge, pkce_verifier) =
    PkceCodeChallenge::new_random_sha256();

  // Generate the authorization URL.
  let (auth_url, csrf_token, nonce) = client
    .authorize_url(
      CoreAuthenticationFlow::AuthorizationCode,
      CsrfToken::new_random,
      Nonce::new_random,
    )
    .set_pkce_challenge(pkce_challenge)
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new("profile".to_string()))
    .add_scope(Scope::new("email".to_string()))
    .url();

  // Data inserted here will be matched on callback side for csrf protection.
  verifier_tokens().insert(
    csrf_token.secret().clone(),
    (
      pkce_verifier,
      nonce,
      redirect,
      komodo_timestamp() + CSRF_VALID_FOR_MS,
    ),
  );

  let config = core_config();
  let redirect = if !config.oidc_redirect_host.is_empty() {
    let auth_url = auth_url.as_str();
    let (protocol, rest) = auth_url
      .split_once("://")
      .context("Invalid URL: Missing protocol (eg 'https://')")?;
    let host = rest
      .split_once(['/', '?'])
      .map(|(host, _)| host)
      .unwrap_or(rest);
    Redirect::to(&auth_url.replace(
      &format!("{protocol}://{host}"),
      &config.oidc_redirect_host,
    ))
  } else {
    Redirect::to(auth_url.as_str())
  };

  Ok(redirect)
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
  state: Option<String>,
  code: Option<String>,
  error: Option<String>,
}

async fn callback(
  Query(query): Query<CallbackQuery>,
) -> anyhow::Result<Redirect> {
  let client = oidc_client().load();
  let client =
    client.as_ref().context("OIDC Client not initialized successfully. Is the provider properly configured?")?;

  if let Some(e) = query.error {
    return Err(anyhow!("Provider returned error: {e}"));
  }

  let code = query.code.context("Provider did not return code")?;
  let state = CsrfToken::new(
    query.state.context("Provider did not return state")?,
  );

  let (_, (pkce_verifier, nonce, redirect, valid_until)) =
    verifier_tokens()
      .remove(state.secret())
      .context("CSRF token invalid")?;

  if komodo_timestamp() > valid_until {
    return Err(anyhow!(
      "CSRF token invalid (Timed out). The token must be used within 2 minutes."
    ));
  }

  let reqwest_client = reqwest_client();
  let token_response = client
    .exchange_code(AuthorizationCode::new(code))
    .context("Failed to get Oauth token at exchange code")?
    .set_pkce_verifier(pkce_verifier)
    .request_async(reqwest_client)
    .await
    .context("Failed to get Oauth token")?;

  // Extract the ID token claims after verifying its authenticity and nonce.
  let id_token = token_response
    .id_token()
    .context("OIDC Server did not return an ID token")?;

  // Some providers attach additional audiences, they must be added here
  // so token verification succeeds.
  let verifier = client.id_token_verifier();
  let additional_audiences = &core_config().oidc_additional_audiences;
  let verifier = if additional_audiences.is_empty() {
    verifier
  } else {
    verifier.set_other_audience_verifier_fn(|aud| {
      additional_audiences.contains(aud)
    })
  };

  let claims = id_token
    .claims(&verifier, &nonce)
    .context("Failed to verify token claims. This issue may be temporary (60 seconds max).")?;

  // Verify the access token hash to ensure that the access token hasn't been substituted for
  // another user's.
  if let Some(expected_access_token_hash) = claims.access_token_hash()
  {
    let actual_access_token_hash = AccessTokenHash::from_token(
      token_response.access_token(),
      id_token.signing_alg()?,
      id_token.signing_key(&verifier)?,
    )?;
    if actual_access_token_hash != *expected_access_token_hash {
      return Err(anyhow!("Invalid access token"));
    }
  }

  let user_id = claims.subject().as_str();

  let db_client = db_client();
  let user = db_client
    .users
    .find_one(doc! {
      "config.data.provider": &core_config().oidc_provider,
      "config.data.user_id": user_id
    })
    .await
    .context("failed at find user query from database")?;

  let jwt_or_two_factor = match user {
    Some(user) => {
      if user.totp.secret.is_empty() {
        let jwt = jwt_client()
          .encode(user.id)
          .context("failed to generate jwt")?;
        JwtOrTwoFactor::Jwt(jwt)
      } else {
        let token = random_string(20);
        totp_pending_login_cache()
          .insert(
            token.clone(),
            (user.id, unix_timestamp_ms() + ONE_MIN_MS),
          )
          .await;
        JwtOrTwoFactor::TwoFactor { token }
      }
    }
    None => {
      let ts = komodo_timestamp();
      let no_users_exist =
        db_client.users.find_one(Document::new()).await?.is_none();
      let core_config = core_config();
      if !no_users_exist && core_config.disable_user_registration {
        return Err(anyhow!("User registration is disabled"));
      }

      // Fetch user info
      let user_info = client
        .user_info(
          token_response.access_token().clone(),
          claims.subject().clone().into(),
        )
        .context("Invalid user info request")?
        .request_async::<EmptyAdditionalClaims, _, CoreGenderClaim>(
          reqwest_client,
        )
        .await
        .context("Failed to fetch user info for new user")?;

      // Will use preferred_username, then email, then user_id if it isn't available.
      let mut username = user_info
        .preferred_username()
        .map(|username| username.to_string())
        .unwrap_or_else(|| {
          let email = user_info
            .email()
            .map(|email| email.as_str())
            .unwrap_or(user_id);
          if core_config.oidc_use_full_email {
            email
          } else {
            email
              .split_once('@')
              .map(|(username, _)| username)
              .unwrap_or(email)
          }
          .to_string()
        });

      // Modify username if it already exists
      if db_client
        .users
        .find_one(doc! { "username": &username })
        .await
        .context("Failed to query users collection")?
        .is_some()
      {
        username += "-";
        username += &random_string(5);
      };

      let user = User {
        id: Default::default(),
        username,
        enabled: no_users_exist || core_config.enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        create_server_permissions: no_users_exist,
        create_build_permissions: no_users_exist,
        updated_at: ts,
        last_update_view: 0,
        recents: Default::default(),
        all: Default::default(),
        config: UserConfig::Oidc {
          provider: core_config.oidc_provider.clone(),
          user_id: user_id.to_string(),
        },
        totp: Default::default(),
        webauthn: Default::default(),
      };

      let user_id = db_client
        .users
        .insert_one(user)
        .await
        .context("failed to create user on database")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();

      let jwt = jwt_client()
        .encode(user_id)
        .context("failed to generate jwt")?;
      JwtOrTwoFactor::Jwt(jwt)
    }
  };
  match jwt_or_two_factor {
    JwtOrTwoFactor::Jwt(jwt) => {
      let exchange_token =
        jwt_client().create_exchange_token(jwt).await;
      let redirect_url = if let Some(redirect) = redirect {
        let splitter = if redirect.contains('?') { '&' } else { '?' };
        format!("{redirect}{splitter}token={exchange_token}")
      } else {
        format!("{}?token={exchange_token}", core_config().host)
      };
      Ok(Redirect::to(&redirect_url))
    }
    JwtOrTwoFactor::TwoFactor { token } => {
      let redirect_url = if let Some(redirect) = redirect {
        let splitter = if redirect.contains('?') { '&' } else { '?' };
        format!("{redirect}{splitter}two_factor={token}")
      } else {
        format!("{}?two_factor={token}", core_config().host)
      };
      Ok(Redirect::to(&redirect_url))
    }
  }
}
