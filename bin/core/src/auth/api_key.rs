use anyhow::Context as _;
use database::bson::doc;
use komodo_client::entities::{api_key::ApiKey, komodo_timestamp};
use mogh_auth_client::api::manage::CreateApiKey;

use crate::state::db_client;

pub async fn create_api_key(
  user_id: String,
  CreateApiKey { name, expires }: CreateApiKey,
  key: String,
  hashed_secret: String,
) -> anyhow::Result<()> {
  let api_key = ApiKey {
    name,
    key,
    secret: hashed_secret,
    user_id,
    created_at: komodo_timestamp(),
    expires: expires as i64,
  };

  db_client()
    .api_keys
    .insert_one(api_key)
    .await
    .context("Failed to create api key on database")?;

  Ok(())
}

pub async fn delete_api_key(key: &str) -> anyhow::Result<()> {
  db_client()
    .api_keys
    .delete_one(doc! { "key": key })
    .await
    .context("Failed to delete api key from database")?;
  Ok(())
}
