use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, anyhow};
use axum::{
  Extension, Json, Router, extract::Path, middleware, routing::post,
};
use database::{
  bson::{doc, to_bson},
  mungos::by_id::update_one_by_id,
};
use derive_variants::EnumVariants;
use komodo_client::{
  api::user::*,
  entities::{NoData, komodo_timestamp, user::User},
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serror::{AddStatusCode as _, AddStatusCodeError};
use tower_sessions::Session;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
  auth::auth_request,
  config::core_config,
  helpers::{
    query::get_user,
    validations::{validate_password, validate_username},
  },
  state::db_client,
};

use super::Variant;

mod api_key;
mod passkey;
mod totp;

pub struct UserArgs {
  pub user: User,
  /// Per-client session state
  pub session: Option<Session>,
}

#[typeshare]
#[derive(
  Debug, Clone, Serialize, Deserialize, Resolve, EnumVariants,
)]
#[args(UserArgs)]
#[response(Response)]
#[error(serror::Error)]
#[serde(tag = "type", content = "params")]
enum UserRequest {
  PushRecentlyViewed(PushRecentlyViewed),
  SetLastSeenUpdate(SetLastSeenUpdate),
  UpdateUsername(UpdateUsername),
  UpdatePassword(UpdatePassword),
  CreateApiKey(CreateApiKey),
  DeleteApiKey(DeleteApiKey),
  BeginTotpEnrollment(BeginTotpEnrollment),
  ConfirmTotpEnrollment(ConfirmTotpEnrollment),
  UnenrollTotp(UnenrollTotp),
  BeginPasskeyEnrollment(BeginPasskeyEnrollment),
  ConfirmPasskeyEnrollment(ConfirmPasskeyEnrollment),
  UnenrollPasskey(UnenrollPasskey),
  BeginThirdPartyLoginLink(BeginThirdPartyLoginLink),
  UnlinkLogin(UnlinkLogin),
}

pub fn router() -> Router {
  Router::new()
    .route("/", post(handler))
    .route("/{variant}", post(variant_handler))
    .layer(middleware::from_fn(auth_request))
}

async fn variant_handler(
  session: Session,
  user: Extension<User>,
  Path(Variant { variant }): Path<Variant>,
  Json(params): Json<serde_json::Value>,
) -> serror::Result<axum::response::Response> {
  let req: UserRequest = serde_json::from_value(json!({
    "type": variant,
    "params": params,
  }))?;
  handler(session, user, Json(req)).await
}

async fn handler(
  session: Session,
  Extension(user): Extension<User>,
  Json(request): Json<UserRequest>,
) -> serror::Result<axum::response::Response> {
  let timer = Instant::now();
  let req_id = Uuid::new_v4();
  debug!(
    "/user request {req_id} | user: {} ({})",
    user.username, user.id
  );
  let res = request
    .resolve(&UserArgs {
      user,
      session: Some(session),
    })
    .await;
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

//

impl Resolve<UserArgs> for UpdateUsername {
  #[instrument(
    "UpdateUsername",
    skip_all,
    fields(
      operator = user.id,
      new_username = self.username,
    )
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<UpdateUsernameResponse> {
    check_locked(&user.username)?;

    validate_username(&self.username)?;

    let db = db_client();

    if db
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("Failed to query for existing users")?
      .is_some()
    {
      return Err(anyhow!("Username already taken.").into());
    }

    let update = doc! { "$set": { "username": self.username } };

    update_one_by_id(&db.users, &user.id, update, None)
      .await
      .context("Failed to update user username on database.")?;

    Ok(NoData {})
  }
}

//

impl Resolve<UserArgs> for UpdatePassword {
  #[instrument(
    "UpdatePassword",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<UpdatePasswordResponse> {
    check_locked(&user.username)?;

    validate_password(&self.password)
      .status_code(StatusCode::BAD_REQUEST)?;

    db_client().set_user_password(user, &self.password).await?;

    Ok(NoData {})
  }
}

//

#[derive(Serialize, Deserialize)]
pub struct SessionThirdPartyLinkInfo {
  pub user_id: String,
}

impl SessionThirdPartyLinkInfo {
  pub const KEY: &str = "third-party-link-info";
}

impl Resolve<UserArgs> for BeginThirdPartyLoginLink {
  #[instrument(
    "BeginThirdPartyLoginLink",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, session }: &UserArgs,
  ) -> serror::Result<BeginThirdPartyLoginLinkResponse> {
    check_locked(&user.username)?;

    let session = session.as_ref().context(
      "Method called in invalid context. This should not happen",
    )?;

    session
      .insert(
        SessionThirdPartyLinkInfo::KEY,
        SessionThirdPartyLinkInfo {
          user_id: user.id.clone(),
        },
      )
      .await
      .context(
        "Failed to insert third party link info into client session",
      )?;

    Ok(NoData {})
  }
}

impl Resolve<UserArgs> for UnlinkLogin {
  #[instrument(
    "UnlinkLogin",
    skip_all,
    fields(operator = user.id)
  )]
  async fn resolve(
    self,
    UserArgs { user, .. }: &UserArgs,
  ) -> serror::Result<UnlinkLoginResponse> {
    check_locked(&user.username)?;

    let field = match self.provider.to_lowercase().as_str() {
      "local" => "linked_logins.Local",
      "github" => "linked_logins.Github",
      "google" => "linked_logins.Google",
      "oidc" => "linked_logins.Oidc",
      _ => {
        return Err(
          anyhow!(
            "Unrecognized third party login provider: {}",
            self.provider
          )
          .status_code(StatusCode::BAD_REQUEST),
        );
      }
    };

    let update = doc! {
      "$unset": {
        field: ""
      }
    };

    update_one_by_id(&db_client().users, &user.id, update, None)
      .await
      .context("Failed to unlink third partly login on database")?;

    Ok(NoData {})
  }
}

fn check_locked(username: &str) -> serror::Result<()> {
  for locked_username in &core_config().lock_login_credentials_for {
    if locked_username == "__ALL__" || *locked_username == username {
      return Err(
        anyhow!("User login credentials are locked.")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    }
  }
  Ok(())
}
