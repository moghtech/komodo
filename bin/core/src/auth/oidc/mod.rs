use std::sync::OnceLock;

use anyhow::{anyhow, Context};
use axum::{
  extract::Query, response::Redirect, routing::get, Router,
};
use client::default_oidc_client;
use dashmap::DashMap;
use komodo_client::entities::{
  komodo_timestamp,
  user::{User, UserConfig},
};
use mungos::mongodb::bson::{doc, Document};
use openidconnect::{
  core::CoreAuthenticationFlow, AccessTokenHash, AuthorizationCode,
  CsrfToken, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
  PkceCodeVerifier, Scope, TokenResponse,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serror::AddStatusCode;

use crate::{
  config::core_config,
  state::{db_client, jwt_client},
};

use super::RedirectQuery;

pub mod client;

/// CSRF tokens can only be used once from the callback,
/// and must be used within this timeframe
const CSRF_VALID_FOR_MS: i64 = 120_000; // 2 minutes for user to log in.

type RedirectUrl = Option<String>;
type CsrfMap =
  DashMap<String, (PkceCodeVerifier, Nonce, RedirectUrl, i64)>;
fn csrf_verifier_tokens() -> &'static CsrfMap {
  static CSRF: OnceLock<CsrfMap> = OnceLock::new();
  CSRF.get_or_init(Default::default)
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
      get(|query| async {
        callback(query).await.status_code(StatusCode::UNAUTHORIZED)
      }),
    )
}

#[instrument(name = "OidcRedirect", level = "debug")]
async fn login(
  Query(RedirectQuery { redirect }): Query<RedirectQuery>,
) -> anyhow::Result<Redirect> {
  let client =
    default_oidc_client().context("OIDC Client not configured")?;

  // Generate a PKCE challenge.
  let (pkce_challenge, pkce_verifier) =
    PkceCodeChallenge::new_random_sha256();

  // Generate the authorization URL.
  let (auth_url, csrf_token, nonce) = client
    .authorize_url(
      CoreAuthenticationFlow::AuthorizationCode,
      CsrfToken::new_random,
      Nonce::new_random,
    )
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new("email".to_string()))
    .set_pkce_challenge(pkce_challenge)
    .url();

  // Data inserted here will be matched on callback side for csrf protection.
  csrf_verifier_tokens().insert(
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

#[instrument(name = "OidcCallback", level = "debug")]
async fn callback(
  Query(query): Query<CallbackQuery>,
) -> anyhow::Result<Redirect> {
  let client =
    default_oidc_client().context("OIDC Client not configured")?;

  if let Some(e) = query.error {
    return Err(anyhow!("Provider returned error: {e}"));
  }

  let code = query.code.context("Provider did not return code")?;
  let state = CsrfToken::new(
    query.state.context("Provider did not return state")?,
  );

  let (_, (pkce_verifier, nonce, redirect, valid_until)) =
    csrf_verifier_tokens()
      .remove(state.secret())
      .context("CSRF Token invalid")?;

  if komodo_timestamp() > valid_until {
    return Err(anyhow!(
      "CSRF token invalid (Timed out). The token must be used within 2 minutes."
    ));
  }

  let token_response = client
    .exchange_code(AuthorizationCode::new(code))
    // Set the PKCE code verifier.
    .set_pkce_verifier(pkce_verifier)
    .request_async(openidconnect::reqwest::async_http_client)
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
    .context("Failed to verify token claims")?;

  // Verify the access token hash to ensure that the access token hasn't been substituted for
  // another user's.
  if let Some(expected_access_token_hash) = claims.access_token_hash()
  {
    let actual_access_token_hash = AccessTokenHash::from_token(
      token_response.access_token(),
      &id_token.signing_alg()?,
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

  let jwt = match user {
    Some(user) => jwt_client()
      .generate(user.id)
      .context("failed to generate jwt")?,
    None => {
      let ts = komodo_timestamp();
      let no_users_exist =
        db_client.users.find_one(Document::new()).await?.is_none();
      let core_config = core_config();
      if !no_users_exist && core_config.disable_user_registration {
        return Err(anyhow!("User registration is disabled"));
      }
      // Will use preferred_username, then email, then user_id if it isn't available.
      let username = claims
        .preferred_username()
        .map(|username| username.to_string())
        .unwrap_or_else(|| {
          let email = claims
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
      jwt_client()
        .generate(user_id)
        .context("failed to generate jwt")?
    }
  };
  let exchange_token = jwt_client().create_exchange_token(jwt).await;
  let redirect_url = if let Some(redirect) = redirect {
    let splitter = if redirect.contains('?') { '&' } else { '?' };
    format!("{}{splitter}token={exchange_token}", redirect)
  } else {
    format!("{}?token={exchange_token}", core_config().host)
  };
  Ok(Redirect::to(&redirect_url))
}
