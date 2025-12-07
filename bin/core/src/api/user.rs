use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, anyhow};
use async_timing_util::{FIVE_MINS_MS, unix_timestamp_ms};
use axum::{
  Extension, Json, Router, extract::Path, middleware, routing::post,
};
use data_encoding::BASE32_NOPAD;
use database::hash_password;
use database::mongo_indexed::doc;
use database::mungos::{
  by_id::update_one_by_id, mongodb::bson::to_bson,
};
use derive_variants::EnumVariants;
use komodo_client::entities::{random_bytes, random_string};
use komodo_client::{
  api::user::*,
  entities::{api_key::ApiKey, komodo_timestamp, user::User},
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{AddStatusCode, AddStatusCodeError};
use typeshare::typeshare;
use uuid::Uuid;

use crate::auth::totp::make_totp;
use crate::helpers::validations::validate_api_key_name;
use crate::state::totp_enrollment_cache;
use crate::{
  auth::auth_request, helpers::query::get_user, state::db_client,
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
  BeginTotpEnrollment(BeginTotpEnrollment),
  ConfirmTotpEnrollment(ConfirmTotpEnrollment),
  UnenrollTotp(UnenrollTotp),
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
    UserArgs { user }: &UserArgs,
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
    UserArgs { user }: &UserArgs,
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
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<CreateApiKeyResponse> {
    let user = get_user(&user.id).await?;

    validate_api_key_name(&self.name)
      .status_code(StatusCode::BAD_REQUEST)?;

    let key = format!("K-{}", random_string(SECRET_LENGTH));
    let secret = format!("S-{}", random_string(SECRET_LENGTH));
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
    UserArgs { user }: &UserArgs,
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

const TOTP_ENROLLMENT_SECRET_LENGTH: usize = 20;

impl Resolve<UserArgs> for BeginTotpEnrollment {
  #[instrument(
    "BeginTotpEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<BeginTotpEnrollmentResponse> {
    let secret_bytes = random_bytes(TOTP_ENROLLMENT_SECRET_LENGTH);
    let totp = make_totp(secret_bytes.clone(), user.id.clone())?;
    let png = totp
      .get_qr_base64()
      .map_err(|e| anyhow::Error::msg(e))
      .context("Failed to generate QR code png")?;
    let expiry = unix_timestamp_ms() + FIVE_MINS_MS;
    totp_enrollment_cache()
      .insert(user.id.clone(), (secret_bytes, expiry))
      .await;

    Ok(BeginTotpEnrollmentResponse {
      uri: totp.get_url(),
      png,
    })
  }
}

impl Resolve<UserArgs> for ConfirmTotpEnrollment {
  #[instrument(
    "ConfirmTotpEnrollment",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<ConfirmTotpEnrollmentResponse> {
    let (secret_bytes, expiry) =
      totp_enrollment_cache().remove(&user.id).await.context(
        "User has not called BeginTotpEnrollment within 5 minutes.",
      )?;

    if unix_timestamp_ms() > expiry {
      return Err(anyhow!(
        "Totp enrollment timed out, please try BeginTotpEnrollment again and confirm the Totp code within 5 minutes."
      ).status_code(StatusCode::REQUEST_TIMEOUT));
    }

    let encoded_secret = BASE32_NOPAD.encode(&secret_bytes);

    let totp = make_totp(secret_bytes, None)?;

    let valid = totp
      .check_current(&self.code)
      .context("Failed to check code validity")?;

    if !valid {
      return Err(anyhow!(
        "The provided code was not valid. Please try BeginTotpEnrollment flow again."
      ).status_code(StatusCode::BAD_REQUEST));
    }

    let recovery_codes =
      (0..10).map(|_| random_string(20)).collect::<Vec<_>>();
    let hashed_recovery_codes = recovery_codes
      .iter()
      .map(|code| hash_password(code))
      .collect::<anyhow::Result<Vec<_>>>()
      .context("Failed to generate valid recovery codes")?;

    update_one_by_id(
      &db_client().users,
      &user.id,
      doc! {
        "$set": {
          "totp.secret": encoded_secret,
          "totp.confirmed_at": komodo_timestamp(),
          "totp.recovery_codes": hashed_recovery_codes,
        }
      },
      None,
    )
    .await
    .context("Failed to update user totp fields on database")?;

    Ok(ConfirmTotpEnrollmentResponse { recovery_codes })
  }
}

impl Resolve<UserArgs> for UnenrollTotp {
  #[instrument(
    "UnenrollTotp",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user }: &UserArgs,
  ) -> serror::Result<UnenrollTotpResponse> {
    update_one_by_id(
      &db_client().users,
      &user.id,
      doc! {
        "$set": {
          "totp.secret": "",
          "totp.confirmed_at": 0,
          "totp.recovery_codes": [],
        }
      },
      None,
    )
    .await
    .context("Failed to clear user totp fields on database")?;
    Ok(UnenrollTotpResponse {})
  }
}
