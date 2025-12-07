use std::net::SocketAddr;

use anyhow::{Context, anyhow};
use async_timing_util::unix_timestamp_ms;
use axum::{
  extract::{ConnectInfo, Request},
  http::HeaderMap,
  middleware::Next,
  response::Response,
};
use database::mungos::mongodb::bson::doc;
use futures_util::TryFutureExt;
use komodo_client::entities::{komodo_timestamp, user::User};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serde::Deserialize;
use serror::AddStatusCodeError as _;

use crate::{
  helpers::query::get_user,
  state::{auth_rate_limiter, db_client, jwt_client},
};

use self::jwt::JwtClaims;

pub mod github;
pub mod google;
pub mod jwt;
pub mod oidc;
pub mod totp;

mod local;

/// Length of random token in Oauth / OIDC 'state'
const STATE_PREFIX_LENGTH: usize = 20;
/// JWT Clock skew tolerance in milliseconds (10 seconds for JWTs)
const JWT_CLOCK_SKEW_TOLERANCE_MS: u128 = 10 * 1000;
/// Exchange Token Clock skew tolerance in milliseconds (5 seconds for Exchange tokens)
const EXCHANGE_TOKEN_CLOCK_SKEW_TOLERANCE_MS: u128 = 5 * 1000;
/// Api Key Clock skew tolerance in milliseconds (5 minutes for Api Keys)
const API_KEY_CLOCK_SKEW_TOLERANCE_MS: i64 = 5 * 60 * 1000;

#[derive(Debug, Deserialize)]
struct RedirectQuery {
  redirect: Option<String>,
}

pub async fn auth_request(
  headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> serror::Result<Response> {
  let fallback = req
    .extensions()
    .get::<ConnectInfo<SocketAddr>>()
    .map(|addr| addr.ip());
  let user = authenticate_check_enabled(&headers)
    .map_err(|e| e.status_code(StatusCode::UNAUTHORIZED))
    .with_failure_rate_limit_using_headers(
      auth_rate_limiter(),
      &headers,
      fallback,
    )
    .await?;
  req.extensions_mut().insert(user);
  Ok(next.run(req).await)
}

pub async fn get_user_id_from_headers(
  headers: &HeaderMap,
) -> anyhow::Result<String> {
  match (
    headers.get("authorization"),
    headers.get("x-api-key"),
    headers.get("x-api-secret"),
  ) {
    (Some(jwt), _, _) => {
      // USE JWT
      let jwt = jwt.to_str().context("JWT is not valid UTF-8")?;
      auth_jwt_get_user_id(jwt).await
    }
    (None, Some(key), Some(secret)) => {
      // USE API KEY / SECRET
      let key =
        key.to_str().context("X-API-KEY is not valid UTF-8")?;
      let secret =
        secret.to_str().context("X-API-SECRET is not valid UTF-8")?;
      auth_api_key_get_user_id(key, secret).await
    }
    _ => {
      // AUTH FAIL
      Err(anyhow!(
        "Must attach either AUTHORIZATION header with jwt OR pass X-API-KEY and X-API-SECRET"
      ))
    }
  }
}

pub async fn authenticate_check_enabled(
  headers: &HeaderMap,
) -> anyhow::Result<User> {
  let user_id = get_user_id_from_headers(headers).await?;
  let user = get_user(&user_id)
    .await
    .map_err(|_| anyhow!("Invalid user credentials"))?;
  if user.enabled {
    Ok(user)
  } else {
    Err(anyhow!("Invalid user credentials"))
  }
}

pub async fn auth_jwt_get_user_id(
  jwt: &str,
) -> anyhow::Result<String> {
  let claims: JwtClaims = jwt_client()
    .decode(jwt)
    .map_err(|_| anyhow!("Invalid user credentials"))?;
  // Apply clock skew tolerance.
  // Token is valid if expiration is greater than (now - tolerance)
  if claims.exp
    > unix_timestamp_ms().saturating_sub(JWT_CLOCK_SKEW_TOLERANCE_MS)
  {
    Ok(claims.id)
  } else {
    Err(anyhow!("Invalid user credentials"))
  }
}

pub async fn auth_jwt_check_enabled(
  jwt: &str,
) -> anyhow::Result<User> {
  let user_id = auth_jwt_get_user_id(jwt).await?;
  check_enabled(user_id).await
}

pub async fn auth_api_key_get_user_id(
  key: &str,
  secret: &str,
) -> anyhow::Result<String> {
  let key = db_client()
    .api_keys
    .find_one(doc! { "key": key })
    .await
    .context("Failed to query db")?
    .context("Invalid user credentials")?;
  // Apply clock skew tolerance.
  // Token is invalid if expiration is less than (now - tolerance)
  if key.expires != 0
    && key.expires
      < komodo_timestamp()
        .saturating_sub(API_KEY_CLOCK_SKEW_TOLERANCE_MS)
  {
    return Err(anyhow!("Invalid user credentials"));
  }
  if bcrypt::verify(secret, &key.secret)
    .map_err(|_| anyhow!("Invalid user credentials"))?
  {
    // secret matches
    Ok(key.user_id)
  } else {
    // secret mismatch
    Err(anyhow!("Invalid user credentials"))
  }
}

pub async fn auth_api_key_check_enabled(
  key: &str,
  secret: &str,
) -> anyhow::Result<User> {
  let user_id = auth_api_key_get_user_id(key, secret).await?;
  check_enabled(user_id).await
}

async fn check_enabled(user_id: String) -> anyhow::Result<User> {
  let user = get_user(&user_id).await?;
  if user.enabled {
    Ok(user)
  } else {
    Err(anyhow!("Invalid user credentials"))
  }
}
