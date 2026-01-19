use anyhow::{Context, anyhow};
use axum::{
  extract::Request, http::HeaderMap, middleware::Next,
  response::Response,
};
use database::mungos::mongodb::bson::doc;
use futures_util::TryFutureExt;
use komodo_client::entities::{komodo_timestamp, user::User};
use mogh_auth_server::request_ip::RequestIp;
use mogh_error::AddStatusCodeError as _;
use mogh_rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;

use crate::{
  auth::JWT_PROVIDER, helpers::query::get_user, state::db_client,
};

pub async fn auth_request(
  RequestIp(ip): RequestIp,
  mut req: Request,
  next: Next,
) -> mogh_error::Result<Response> {
  let mut user = authenticate_check_enabled(req.headers())
    .map_err(|e| e.status_code(StatusCode::UNAUTHORIZED))
    .with_failure_rate_limit_using_ip(
      &super::GENERAL_RATE_LIMITER,
      &ip,
    )
    .await?;
  // Sanitize the user for safety before
  // attaching to the request handlers.
  user.sanitize();
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
  JWT_PROVIDER.decode_sub(jwt)
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
        .saturating_sub(super::API_KEY_CLOCK_SKEW_TOLERANCE_MS)
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
