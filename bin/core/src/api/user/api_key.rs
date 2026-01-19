use anyhow::{Context as _, anyhow};
use database::bson::doc;
use komodo_client::{
  api::user::{
    CreateApiKey, CreateApiKeyResponse, DeleteApiKey,
    DeleteApiKeyResponse,
  },
  entities::{api_key::ApiKey, komodo_timestamp, random_string},
};
use mogh_resolver::Resolve;
use reqwest::StatusCode;
use serror::{AddStatusCode as _, AddStatusCodeError as _};

use crate::{
  helpers::{query::get_user, validations::validate_api_key_name},
  state::db_client,
};

use super::UserArgs;

const SECRET_LENGTH: usize = 40;
const BCRYPT_COST: u32 = 10;

impl Resolve<UserArgs> for CreateApiKey {
  #[instrument(
    "CreateApiKey",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<CreateApiKeyResponse> {
    let user = get_user(&user.id).await?;

    validate_api_key_name(&self.name)
      .status_code(StatusCode::BAD_REQUEST)?;

    let key = format!("K_{}_K", random_string(SECRET_LENGTH));
    let secret = format!("S_{}_S", random_string(SECRET_LENGTH));
    let secret_hash = bcrypt::hash(&secret, BCRYPT_COST)
      .context("Failed at hashing secret string")?;

    let api_key = ApiKey {
      name: self.name,
      key: key.clone(),
      secret: secret_hash,
      user_id: user.id.clone(),
      created_at: komodo_timestamp(),
      expires: self.expires,
    };

    db_client()
      .api_keys
      .insert_one(api_key)
      .await
      .context("Failed to create api key on database")?;

    Ok(CreateApiKeyResponse { key, secret })
  }
}

impl Resolve<UserArgs> for DeleteApiKey {
  #[instrument(
    "DeleteApiKey",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<DeleteApiKeyResponse> {
    let client = db_client();

    let key = client
      .api_keys
      .find_one(doc! { "key": &self.key })
      .await
      .context("Failed at database query")?
      .context("No api key with key found")?;

    if user.id != key.user_id {
      return Err(
        anyhow!("Api key does not belong to user")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    client
      .api_keys
      .delete_one(doc! { "key": key.key })
      .await
      .context("Failed to delete api key from database")?;

    Ok(DeleteApiKeyResponse {})
  }
}
