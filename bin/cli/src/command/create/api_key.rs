use anyhow::Context as _;
use database::bson::doc;
use komodo_client::{
  api::read::FindUser,
  entities::{
    api_key::ApiKey, config::cli::args::create::CreateApiKey,
    komodo_timestamp, random_string,
  },
};
use serde_json::json;

use crate::config::cli_config;

pub async fn create(
  CreateApiKey {
    name,
    for_user,
    expires,
    use_api,
  }: &CreateApiKey,
) -> anyhow::Result<()> {
  let expires = if let Some(expires_days) = expires {
    // now + expires in ms
    komodo_timestamp() + expires_days * 24 * 60 * 60 * 1000
  } else {
    0
  };

  if *use_api {
    // USE API
    let client = crate::command::komodo_client().await?;

    let keys = if let Some(username) = for_user {
      // For service user
      let user = client
        .read(FindUser {
          user: username.to_string(),
        })
        .await?;
      client
        .write(
          komodo_client::api::write::CreateApiKeyForServiceUser {
            user_id: user.id,
            name: name.clone().unwrap_or_default(),
            expires,
          },
        )
        .await?
    } else {
      // For self
      client
        .auth_manage(komodo_client::api::auth::manage::CreateApiKey {
          name: name.clone().unwrap_or_default(),
          expires: expires as u64,
        })
        .await?
    };

    println!(
      "{}",
      serde_json::to_string_pretty(&keys)
        .context("Failed to serialize api keys to JSON")?
    );
  } else {
    // USE DATABASE
    let db = database::Client::new(&cli_config().database).await?;

    let user = db
      .users
      .find_one(doc! { "username": for_user })
      .await
      .context("Failed to query database for user")?
      .context("No user found with given username")?;

    let key = format!("K_{}_K", random_string(40));
    let secret = format!("S_{}_S", random_string(40));
    let hashed_secret = bcrypt::hash(&secret, 10)
      .context("Failed at hashing secret string")?;

    db.api_keys
      .insert_one(&ApiKey {
        name: name.clone().unwrap_or_default(),
        user_id: user.id,
        key: key.clone(),
        secret: hashed_secret.clone(),
        created_at: komodo_timestamp(),
        expires,
      })
      .await?;

    println!(
      "{}",
      serde_json::to_string_pretty(
        &json!({ "key": key, "secret": secret })
      )
      .context("Failed to serialize api keys to JSON")?
    );
  }

  Ok(())
}
