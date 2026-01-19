use std::{collections::VecDeque, time::Instant};

use anyhow::Context;
use axum::{
  Extension, Json, Router, extract::Path, middleware, routing::post,
};
use database::{
  bson::{doc, to_bson},
  mungos::by_id::update_one_by_id,
};
use komodo_client::{
  api::user::*,
  entities::{komodo_timestamp, user::User},
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::Response;
use strum::EnumDiscriminants;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::auth_request, helpers::query::get_user, state::db_client,
};

use super::Variant;

mod api_key;

pub struct UserArgs {
  pub user: User,
}

#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EnumDiscriminants,
)]
#[strum_discriminants(name(UserRequestVariant))]
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
    .route(
      "/",
      post(handler)
        // TODO: allow this to return when user not enabled to display "not enabled" page.
        .get(|Extension(user): Extension<User>| async { Json(user) }),
    )
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
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<PushRecentlyViewedResponse> {
    let user = get_user(&user.id).await?;

    let (resource_type, id) = self.resource.extract_variant_id();

    let field = format!("recents.{resource_type}");

    let update = match user.recents.get(&resource_type) {
      Some(recents) => {
        let mut recents = recents
          .iter()
          .filter(|_id| !id.eq(*_id))
          .take(RECENTLY_VIEWED_MAX - 1)
          .collect::<VecDeque<_>>();

        recents.push_front(id);

        doc! { &field: to_bson(&recents)? }
      }
      None => {
        doc! { &field: [id] }
      }
    };

    update_one_by_id(
      &db_client().users,
      &user.id,
      database::mungos::update::Update::Set(update),
      None,
    )
    .await
    .with_context(|| format!("Failed to update user '{field}'"))?;

    Ok(PushRecentlyViewedResponse {})
  }
}

impl Resolve<UserArgs> for SetLastSeenUpdate {
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<SetLastSeenUpdateResponse> {
    update_one_by_id(
      &db_client().users,
      &user.id,
      database::mungos::update::Update::Set(doc! {
        "last_update_view": komodo_timestamp()
      }),
      None,
    )
    .await
    .context("Failed to update user 'last_update_view'")?;

    Ok(SetLastSeenUpdateResponse {})
  }
}
