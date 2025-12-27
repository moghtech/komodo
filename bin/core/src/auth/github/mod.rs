use std::net::SocketAddr;

use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::{ConnectInfo, Query},
  http::HeaderMap,
  response::Redirect,
  routing::get,
};
use database::mungos::mongodb::bson::doc;
use database::{
  mongo_indexed::Document, mungos::by_id::update_one_by_id,
};
use futures_util::TryFutureExt;
use komodo_client::{
  api::auth::UserIdOrTwoFactor,
  entities::{
    komodo_timestamp, random_string,
    user::{NewUserParams, User, UserConfig},
  },
};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serror::{AddStatusCode, AddStatusCodeError as _};
use tower_sessions::Session;

use crate::{
  api::user::SessionThirdPartyLinkInfo,
  auth::{
    SessionPasskeyLogin, SessionTotpLogin, SessionUserId,
    format_redirect, github::client::GithubOauthClient,
  },
  config::core_config,
  helpers::query::{find_github_user, get_user},
  state::{auth_rate_limiter, db_client, webauthn},
};

use self::client::github_oauth_client;

use super::{RedirectQuery, STATE_PREFIX_LENGTH};

pub mod client;

pub fn router() -> Router {
  Router::new()
    .route("/login", get(login))
    .route("/link", get(link))
    .route(
      "/callback",
      get(
        |query,
         session: Session,
         headers: HeaderMap,
         ConnectInfo(info): ConnectInfo<SocketAddr>| async move {
          callback(query, session)
            .map_err(|e| e.status_code(StatusCode::UNAUTHORIZED))
            .with_failure_rate_limit_using_headers(
              auth_rate_limiter(),
              &headers,
              Some(info.ip()),
            )
            .await
        },
      ),
    )
}

#[derive(Serialize, Deserialize)]
struct SessionGithubVerificationInfo {
  state: String,
}

impl SessionGithubVerificationInfo {
  const KEY: &str = "github-verification-info";
}

async fn login(
  session: Session,
  Query(query): Query<RedirectQuery>,
) -> serror::Result<Redirect> {
  let (state, uri) = github_oauth_client()
    .as_ref()
    .context("Github Oauth not configured")
    .status_code(StatusCode::UNAUTHORIZED)?
    .get_state_and_login_redirect_url(query.redirect)
    .await;
  session
    .insert(
      SessionGithubVerificationInfo::KEY,
      SessionGithubVerificationInfo { state },
    )
    .await
    .context("Failed to insert github oauth session state")?;
  Ok(Redirect::to(&uri))
}

#[derive(Serialize, Deserialize)]
struct SessionGithubLinkInfo {
  user_id: String,
  state: String,
}

impl SessionGithubLinkInfo {
  const KEY: &str = "github-link-info";
}

async fn link(session: Session) -> serror::Result<Redirect> {
  let SessionThirdPartyLinkInfo { user_id } = session
    .remove(SessionThirdPartyLinkInfo::KEY)
    .await
    .context("Invalid session third party link info.")?
    .context("Missing session third party link info")?;
  let (state, uri) = github_oauth_client()
    .as_ref()
    .context("Github Oauth not configured")
    .status_code(StatusCode::UNAUTHORIZED)?
    .get_state_and_login_redirect_url(None)
    .await;
  session
    .insert(
      SessionGithubLinkInfo::KEY,
      SessionGithubLinkInfo { user_id, state },
    )
    .await
    .context("Failed to insert session link info")?;
  Ok(Redirect::to(&uri))
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
  state: String,
  code: String,
}

async fn callback(
  Query(query): Query<CallbackQuery>,
  session: Session,
) -> anyhow::Result<Redirect> {
  let client =
    github_oauth_client().context("Missing github oauth client")?;

  // Check first if this is a link callback
  // and use the linking handler if so.
  if let Ok(Some(info)) =
    session.remove(SessionGithubLinkInfo::KEY).await
  {
    return link_github_callback(client, info, query).await;
  }

  let SessionGithubVerificationInfo { state } = session
    .remove(SessionGithubVerificationInfo::KEY)
    .await
    .context("Invalid github oauth session state")?
    .context("Missing github oauth session state")?;

  if query.state != state {
    return Err(anyhow!("State mismatch"));
  }

  let token = client.get_access_token(&query.code).await?;

  let github_user =
    client.get_github_user(&token.access_token).await?;

  let github_id = github_user.id.to_string();

  let db_client = db_client();

  let user = find_github_user(&github_id).await?;

  let user_id_or_two_factor = match user {
    Some(user) => {
      match (
        user.third_party_skip_2fa,
        user.passkey.passkey,
        user.totp.enrolled(),
      ) {
        // Skip / No 2FA
        (true, _, _) | (false, None, false) => {
          session
            .insert(
              SessionUserId::KEY,
              SessionUserId(user.id.clone()),
            )
            .await
            .context("Failed to store user id for client session")?;
          UserIdOrTwoFactor::UserId(user.id)
        }
        // WebAuthn Passkey 2FA
        (false, Some(passkey), _) => {
          let webauthn = webauthn().context(
            "No webauthn provider available, invalid KOMODO_HOST config",
          )?;
          let (response, server_state) = webauthn
            .start_passkey_authentication(&[passkey])
            .context("Failed to start passkey authentication flow")?;
          session
            .insert(
              SessionPasskeyLogin::KEY,
              SessionPasskeyLogin {
                user_id: user.id,
                state: server_state,
              },
            )
            .await?;
          UserIdOrTwoFactor::Passkey(response)
        }
        // TOTP 2FA
        (false, None, true) => {
          session
            .insert(
              SessionTotpLogin::KEY,
              SessionTotpLogin { user_id: user.id },
            )
            .await
            .context(
              "Failed to store totp login state in for user session",
            )?;
          UserIdOrTwoFactor::Totp {}
        }
      }
    }
    None => {
      let ts = komodo_timestamp();
      let no_users_exist =
        db_client.users.find_one(Document::new()).await?.is_none();
      let core_config = core_config();
      if !no_users_exist && core_config.disable_user_registration {
        return Err(anyhow!("User registration is disabled"));
      }

      let mut username = github_user.login;
      // Modify username if it already exists
      if db_client
        .users
        .find_one(doc! { "username": &username })
        .await
        .context("Failed to query users collection")?
        .is_some()
      {
        username += "-";
        username += &random_string(5);
      };

      let user = User::new(NewUserParams {
        username,
        enabled: no_users_exist || core_config.enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        config: UserConfig::Github {
          github_id,
          avatar: github_user.avatar_url,
        },
        updated_at: ts,
      });

      let user_id = db_client
        .users
        .insert_one(user)
        .await
        .context("Failed to create user on mongo")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();
      UserIdOrTwoFactor::UserId(user_id)
    }
  };
  let redirect = Some(&query.state[STATE_PREFIX_LENGTH..]);
  match user_id_or_two_factor {
    UserIdOrTwoFactor::UserId(_) => {
      Ok(format_redirect(redirect, "redeem_ready=true"))
    }
    UserIdOrTwoFactor::Totp {} => {
      Ok(format_redirect(redirect, "totp=true"))
    }
    UserIdOrTwoFactor::Passkey(passkey) => {
      let passkey = serde_json::to_string(&passkey)
        .context("Failed to serialize passkey response")?;
      let passkey = urlencoding::encode(&passkey);
      Ok(format_redirect(redirect, &format!("passkey={passkey}")))
    }
  }
}

/// This intercepts during the normal oauth callback if
/// 'github-link-info' is found on session.
async fn link_github_callback(
  client: &GithubOauthClient,
  SessionGithubLinkInfo { user_id, state }: SessionGithubLinkInfo,
  query: CallbackQuery,
) -> anyhow::Result<Redirect> {
  if query.state != state {
    return Err(anyhow!("State mismatch"));
  }

  let token = client.get_access_token(&query.code).await?;

  let github_user =
    client.get_github_user(&token.access_token).await?;
  let github_id = github_user.id.to_string();

  if let Some(existing_user) = find_github_user(&github_id).await? {
    if existing_user.id == user_id {
      // Link is already complete, this is a no-op
      return Ok(Redirect::to(&format!(
        "{}/settings",
        core_config().host,
      )));
    } else {
      return Err(anyhow!("Account already linked to another user."));
    }
  }

  let user = get_user(&user_id).await?;

  if let UserConfig::Github { .. } = &user.config {
    return Err(anyhow!(
      "User is primary Github user, cannot link another Github login."
    ));
  }

  let update = doc! {
    "$set": {
      "linked_logins.Github.type": "Github",
      "linked_logins.Github.data.github_id": &github_id,
      "linked_logins.Github.data.avatar": &github_user.avatar_url,
    }
  };

  update_one_by_id(&db_client().users, &user_id, update, None)
    .await
    .context(
      "Failed to link Github login to existing user on database",
    )?;

  Ok(Redirect::to(&format!("{}/settings", core_config().host,)))
}
