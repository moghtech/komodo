use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::doc;
use komodo_client::entities::{komodo_timestamp, user::User};
use mogh_auth_server::RequestAuthentication;

use crate::{
  auth::JWT_PROVIDER, helpers::query::get_user, state::db_client,
};

pub async fn extract_user_from_auth(
  auth: RequestAuthentication,
  require_user_enabled: bool,
) -> anyhow::Result<User> {
  let user_id = match auth {
    RequestAuthentication::UserId(user_id) => user_id,
    RequestAuthentication::KeyAndSecret { key, secret } => {
      auth_api_key_get_user_id(&key, &secret).await?
    }
    RequestAuthentication::PublicKey(_) => todo!(),
  };
  if require_user_enabled {
    check_enabled(&user_id).await
  } else {
    get_user(&user_id).await
  }
}

pub async fn auth_jwt_check_enabled(
  jwt: &str,
) -> anyhow::Result<User> {
  let user_id = JWT_PROVIDER.decode_sub(jwt)?;
  check_enabled(&user_id).await
}

pub async fn auth_api_key_check_enabled(
  key: &str,
  secret: &str,
) -> anyhow::Result<User> {
  let user_id = auth_api_key_get_user_id(key, secret).await?;
  check_enabled(&user_id).await
}

/// Api Key Clock skew tolerance in milliseconds (5 minutes for Api Keys)
const API_KEY_CLOCK_SKEW_TOLERANCE_MS: i64 = 5 * 60 * 1000;

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

async fn check_enabled(user_id: &str) -> anyhow::Result<User> {
  let user = get_user(user_id).await?;
  if user.enabled {
    Ok(user)
  } else {
    Err(anyhow!("Invalid user credentials"))
  }
}
