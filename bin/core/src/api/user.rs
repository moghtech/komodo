use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, anyhow};
use axum::{
  Extension, Json, Router, extract::Path, middleware, routing::post,
};
use derive_variants::EnumVariants;
use komodo_client::{
  api::user::*,
  entities::{api_key::ApiKey, komodo_timestamp, user::User},
};
use mongo_indexed::doc;
use mungos::{by_id::update_one_by_id, mongodb::bson::to_bson};
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::auth_request,
  helpers::{query::get_user, random_string},
  state::db_client,
};

use super::Variant;

pub struct UserArgs {
  pub user: User,
}

#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Resolve, EnumVariants,
)]
#[args(UserArgs)]
#[response(Response)]
#[error(serror::Error)]
#[serde(tag = "type", content = "params")]
enum UserRequest {
  PushRecentlyViewed(PushRecentlyViewed),
  SetLastSeenUpdate(SetLastSeenUpdate),
  CreateApiKey(CreateApiKey),
  DeleteApiKey(DeleteApiKey),
}

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .route("/{variant}", post(variant_handler))
    .layer(middleware::from_fn(auth_request))
}

async fn variant_handler(
  user: Extension<User>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<axum::response::Response> {
  let req: UserRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(user, Json(req)).await
}

#[instrument(name = "UserHandler", level = "debug", skip(user))]
async fn handler(
  Extension(user): Extension<User>,
  Json(request): Json<UserRequest>,
) -> serror::Result<axum::response::Response> {
  let timer = Instant::now();
  let req_id = Uuid::new_v4();
  debug!(
    "/user request {req_id} | user: {} ({})",
    user.username, user.id
  );
  let res = request.resolve(&UserArgs { user }).await;
  if let Err(e) = &res {
    warn!("/user request {req_id} error: {:#}", e.error);
  }
  let elapsed = timer.elapsed();
  debug!("/user request {req_id} | resolve time: {elapsed:?}");
  res.map(|res| res.0)
}

const RECENTLY_VIEWED_MAX: usize = 10;

impl Resolve<UserArgs> for PushRecentlyViewed {
  #[instrument(
    name = "PushRecentlyViewed",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<PushRecentlyViewedResponse> {
    let user = get_user(&user.id).await?;

    let (resource_type, id) = self.resource.extract_variant_id();
    let update = match user.recents.get(&resource_type) {
      Some(recents) => {
        let mut recents = recents
          .iter()
          .filter(|_id| !id.eq(*_id))
          .take(RECENTLY_VIEWED_MAX - 1)
          .collect::<VecDeque<_>>();
        recents.push_front(id);
        doc! { format!("recents.{resource_type}"): to_bson(&recents)? }
      }
      None => {
        doc! { format!("recents.{resource_type}"): [id] }
      }
    };
    update_one_by_id(
      &db_client().users,
      &user.id,
      mungos::update::Update::Set(update),
      None,
    )
    .await
    .with_context(|| {
      format!("failed to update recents.{resource_type}")
    })?;

    Ok(PushRecentlyViewedResponse {})
  }
}

impl Resolve<UserArgs> for SetLastSeenUpdate {
  #[instrument(
    name = "SetLastSeenUpdate",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<SetLastSeenUpdateResponse> {
    update_one_by_id(
      &db_client().users,
      &user.id,
      mungos::update::Update::Set(doc! {
        "last_update_view": komodo_timestamp()
      }),
      None,
    )
    .await
    .context("failed to update user last_update_view")?;
    Ok(SetLastSeenUpdateResponse {})
  }
}

const SECRET_LENGTH: usize = 40;
const BCRYPT_COST: u32 = 10;

impl Resolve<UserArgs> for CreateApiKey {
  #[instrument(name = "CreateApiKey", level = "debug", skip(user))]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<CreateApiKeyResponse> {
    let user = get_user(&user.id).await?;

    let key = format!("K-{}", random_string(SECRET_LENGTH));
    let secret = format!("S-{}", random_string(SECRET_LENGTH));
    let secret_hash = bcrypt::hash(&secret, BCRYPT_COST)
      .context("failed at hashing secret string")?;

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
      .context("failed to create api key on db")?;
    Ok(CreateApiKeyResponse { key, secret })
  }
}

impl Resolve<UserArgs> for DeleteApiKey {
  #[instrument(name = "DeleteApiKey", level = "debug", skip(user))]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<DeleteApiKeyResponse> {
    let client = db_client();
    let key = client
      .api_keys
      .find_one(doc! { "key": &self.key })
      .await
      .context("failed at db query")?
      .context("no api key with key found")?;
    if user.id != key.user_id {
      return Err(anyhow!("api key does not belong to user").into());
    }
    client
      .api_keys
      .delete_one(doc! { "key": key.key })
      .await
      .context("failed to delete api key from db")?;
    Ok(DeleteApiKeyResponse {})
  }
}
