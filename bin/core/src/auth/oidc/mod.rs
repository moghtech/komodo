use std::net::SocketAddr;

use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::{ConnectInfo, Query},
  http::HeaderMap,
  response::Redirect,
  routing::get,
};
use database::mungos::{
  by_id::update_one_by_id,
  mongodb::bson::{Document, doc},
};
use futures_util::TryFutureExt;
use komodo_client::{
  api::auth::UserIdOrTwoFactor,
  entities::{
    komodo_timestamp, random_string,
    user::{NewUserParams, User, UserConfig},
  },
};
use openidconnect::{
  CsrfToken, Nonce, PkceCodeChallenge, PkceCodeVerifier,
};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serror::{AddStatusCode as _, AddStatusCodeError};
use tower_sessions::Session;

use crate::{
  api::user::SessionThirdPartyLinkInfo,
  auth::{
    SessionPasskeyLogin, SessionTotpLogin, SessionUserId,
    format_redirect,
    oidc::client::{OidcClient, load_oidc_client},
  },
  config::core_config,
  helpers::query::{find_oidc_user, get_user},
  state::{auth_rate_limiter, db_client, webauthn},
};

use super::RedirectQuery;

pub mod client;

pub fn router() -> Router {
  Router::new()
    .route(
      "/login",
      get(|session, query| async {
        login(session, query)
          .await
          .status_code(StatusCode::UNAUTHORIZED)
      }),
    )
    .route(
      "/link",
      get(|session| async {
        link(session).await.status_code(StatusCode::UNAUTHORIZED)
      }),
    )
    .route(
      "/callback",
      get(
        |query,
         session,
         headers: HeaderMap,
         ConnectInfo(info): ConnectInfo<SocketAddr>| async move {
          callback(session, query)
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

type RedirectUrl = Option<String>;

#[derive(Serialize, Deserialize)]
struct SessionOidcVerificationInfo {
  csrf_token: String,
  pkce_verifier: PkceCodeVerifier,
  nonce: Nonce,
  redirect: RedirectUrl,
}

impl SessionOidcVerificationInfo {
  const KEY: &str = "oidc-verification-info";
}

async fn login(
  session: Session,
  Query(RedirectQuery { redirect }): Query<RedirectQuery>,
) -> anyhow::Result<Redirect> {
  let client = load_oidc_client()
    .await
    .context("OIDC Client not available")?;

  let (pkce_challenge, pkce_verifier) =
    PkceCodeChallenge::new_random_sha256();

  // Generate the authorization URL.
  let (auth_url, csrf_token, nonce) =
    client.authorize_url(pkce_challenge);

  // Data inserted here will be matched on callback side for csrf protection.
  session
    .insert(
      SessionOidcVerificationInfo::KEY,
      SessionOidcVerificationInfo {
        csrf_token: csrf_token.secret().clone(),
        pkce_verifier,
        nonce,
        redirect,
      },
    )
    .await
    .context("Failed to insert session verification info")?;

  auth_redirect(auth_url.as_str())
}

#[derive(Serialize, Deserialize)]
struct SessionOidcLinkInfo {
  user_id: String,
  csrf_token: String,
  pkce_verifier: PkceCodeVerifier,
  nonce: Nonce,
}

impl SessionOidcLinkInfo {
  const KEY: &str = "oidc-link-info";
}

async fn link(session: Session) -> anyhow::Result<Redirect> {
  let SessionThirdPartyLinkInfo { user_id } = session
    .remove(SessionThirdPartyLinkInfo::KEY)
    .await
    .context("Invalid session third party link info.")?
    .context("Missing session third party link info")?;

  let client = load_oidc_client()
    .await
    .context("OIDC Client not available")?;

  let (pkce_challenge, pkce_verifier) =
    PkceCodeChallenge::new_random_sha256();

  // Generate the authorization URL.
  let (auth_url, csrf_token, nonce) =
    client.authorize_url(pkce_challenge);

  session
    .insert(
      SessionOidcLinkInfo::KEY,
      SessionOidcLinkInfo {
        user_id,
        csrf_token: csrf_token.secret().clone(),
        pkce_verifier,
        nonce,
      },
    )
    .await
    .context("Failed to insert session link info")?;

  auth_redirect(auth_url.as_str())
}

/// Applies 'oidc_redirect_host'
fn auth_redirect(auth_url: &str) -> anyhow::Result<Redirect> {
  let config = core_config();
  let redirect = if !config.oidc_redirect_host.is_empty() {
    let (protocol, rest) = auth_url
      .split_once("://")
      .context("Invalid URL: Missing protocol (eg 'https://')")?;
    let host = rest
      .split_once(['/', '?'])
      .map(|(host, _)| host)
      .unwrap_or(rest);
    Redirect::to(&auth_url.replace(
      &format!("{protocol}://{host}"),
      &config.oidc_redirect_host,
    ))
  } else {
    Redirect::to(auth_url)
  };
  Ok(redirect)
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackQuery {
  state: Option<String>,
  code: Option<String>,
  error: Option<String>,
}

async fn callback(
  session: Session,
  Query(query): Query<OidcCallbackQuery>,
) -> anyhow::Result<Redirect> {
  let client = load_oidc_client()
    .await
    .context("OIDC Client not available")?;

  if let Some(e) = query.error {
    return Err(anyhow!("Provider returned error: {e}"));
  }

  let code = query.code.context("Provider did not return code")?;
  let state = CsrfToken::new(
    query.state.context("Provider did not return state")?,
  );

  // Check first if this is a link callback
  // and use the linking handler if so.
  if let Ok(Some(info)) =
    session.remove(SessionOidcLinkInfo::KEY).await
  {
    return link_oidc_callback(&client, info, state, code).await;
  }

  let SessionOidcVerificationInfo {
    csrf_token,
    pkce_verifier,
    nonce,
    redirect,
  } = session
    .remove(SessionOidcVerificationInfo::KEY)
    .await
    .context("Invalid session verification info.")?
    .context(
      "Missing session verification info for CSRF protection.",
    )?;

  let (subject, token) = client
    .validate_extract_subject_and_token(
      (state, csrf_token),
      code,
      pkce_verifier,
      nonce,
    )
    .await?;

  let user = find_oidc_user(&subject).await?;

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
      let db_client = db_client();
      let no_users_exist =
        db_client.users.find_one(Document::new()).await?.is_none();
      let core_config = core_config();
      if !no_users_exist && core_config.disable_user_registration {
        return Err(anyhow!("User registration is disabled"));
      }

      // Fetch user info
      let user_info =
        client.fetch_user_info(token, subject.clone()).await?;

      // Will use preferred_username, then email, then user_id if it isn't available.
      let mut username = user_info
        .preferred_username()
        .map(|username| username.to_string())
        .unwrap_or_else(|| {
          let email = user_info
            .email()
            .map(|email| email.as_str())
            .unwrap_or(subject.as_str());
          if core_config.oidc_use_full_email {
            email
          } else {
            email
              .split_once('@')
              .map(|(username, _)| username)
              .unwrap_or(email)
          }
          .to_string()
        });

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
        config: UserConfig::Oidc {
          provider: core_config.oidc_provider.clone(),
          user_id: subject.to_string(),
        },
        updated_at: ts,
      });

      let user_id = db_client
        .users
        .insert_one(user)
        .await
        .context("failed to create user on database")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();

      UserIdOrTwoFactor::UserId(user_id)
    }
  };

  match user_id_or_two_factor {
    UserIdOrTwoFactor::UserId(_) => {
      Ok(format_redirect(redirect.as_deref(), "redeem_ready=true"))
    }
    UserIdOrTwoFactor::Totp {} => {
      Ok(format_redirect(redirect.as_deref(), "totp=true"))
    }
    UserIdOrTwoFactor::Passkey(passkey) => {
      let passkey = serde_json::to_string(&passkey)
        .context("Failed to serialize passkey response")?;
      let passkey = urlencoding::encode(&passkey);
      Ok(format_redirect(
        redirect.as_deref(),
        &format!("passkey={passkey}"),
      ))
    }
  }
}

/// This intercepts during the normal oauth callback if
/// 'oidc-link-info' is found on session.
async fn link_oidc_callback(
  client: &OidcClient,
  SessionOidcLinkInfo {
    user_id,
    csrf_token,
    pkce_verifier,
    nonce,
  }: SessionOidcLinkInfo,
  state: CsrfToken,
  code: String,
) -> anyhow::Result<Redirect> {
  let (subject, _) = client
    .validate_extract_subject_and_token(
      (state, csrf_token),
      code,
      pkce_verifier,
      nonce,
    )
    .await?;

  let oidc_provider = &core_config().oidc_provider;
  let oidc_user_id = subject.as_str();

  // Ensure there are no other existing users with this login linked.
  if let Some(existing_user) = find_oidc_user(&subject).await? {
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

  if let UserConfig::Oidc { .. } = &user.config {
    return Err(anyhow!(
      "User is primary Oidc user, cannot link another Oidc login."
    ));
  }

  let update = doc! {
    "$set": {
      "linked_logins.Oidc.type": "Oidc",
      "linked_logins.Oidc.data.provider": oidc_provider,
      "linked_logins.Oidc.data.user_id": oidc_user_id,
    }
  };

  update_one_by_id(&db_client().users, &user_id, update, None)
    .await
    .context(
      "Failed to link OIDC login to existing user on database",
    )?;

  Ok(Redirect::to(&format!("{}/settings", core_config().host,)))
}
