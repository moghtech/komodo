use std::str::FromStr;

use anyhow::{Context, anyhow};
use database::mungos::mongodb::bson::{doc, oid::ObjectId};
use komodo_client::entities::{
  ResourceTarget, komodo_timestamp,
  secret_provider::SecretExecutionActor,
  user::{User, action_user},
};
use mogh_auth_server::RequestAuthentication;

use crate::{
  auth::JWT_PROVIDER, helpers::query::get_user,
  secret_provider::authz::ExecutionSubjectContext, state::db_client,
};

pub async fn extract_user_and_subject_from_auth(
  auth: RequestAuthentication,
  require_user_enabled: bool,
) -> anyhow::Result<(User, Option<ExecutionSubjectContext>)> {
  let (user_id, subject) = match auth {
    RequestAuthentication::UserId(user_id) => (user_id, None),
    RequestAuthentication::KeyAndSecret { key, secret } => {
      auth_api_key_get_user_and_subject(&key, &secret).await?
    }
    RequestAuthentication::PublicKey(_) => todo!(),
  };
  let user = if require_user_enabled {
    check_enabled(&user_id).await?
  } else {
    get_user(&user_id).await?
  };
  Ok((user, subject))
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
  auth_api_key_get_user_and_subject(key, secret)
    .await
    .map(|(uid, _)| uid)
}

pub async fn auth_api_key_get_user_and_subject(
  key: &str,
  secret: &str,
) -> anyhow::Result<(String, Option<ExecutionSubjectContext>)> {
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
    // If this is an ephemeral Action API key (`user_id == action_user().id`),
    // resolve the Action ID from the originating Update (`key.name == update.id`).
    let subject = if key.user_id == action_user().id {
      let action_id = if let Ok(oid) = ObjectId::from_str(&key.name) {
        db_client()
          .updates
          .find_one(doc! { "_id": oid })
          .await
          .ok()
          .flatten()
          .and_then(|u| match u.target {
            ResourceTarget::Action(id) => Some(id),
            _ => None,
          })
          .unwrap_or_default()
      } else {
        String::new()
      };
      Some(ExecutionSubjectContext::SystemActor {
        actor: SecretExecutionActor::Action,
        actor_resource_id: action_id,
      })
    } else {
      None
    };
    Ok((key.user_id, subject))
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
