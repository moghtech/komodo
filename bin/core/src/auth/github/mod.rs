use std::net::SocketAddr;

use anyhow::{Context, anyhow};
use axum::{
  Router,
  extract::{ConnectInfo, Query},
  http::HeaderMap,
  response::Redirect,
  routing::get,
};
use database::mongo_indexed::Document;
use database::mungos::mongodb::bson::doc;
use futures_util::TryFutureExt;
use komodo_client::entities::{
  komodo_timestamp, random_string,
  user::{User, UserConfig},
};
use rate_limit::WithFailureRateLimit;
use reqwest::StatusCode;
use serde::Deserialize;
use serror::{AddStatusCode, AddStatusCodeError as _};

use crate::{
  config::core_config,
  state::{auth_rate_limiter, db_client, jwt_client},
};

use self::client::github_oauth_client;

use super::{RedirectQuery, STATE_PREFIX_LENGTH};

pub mod client;

pub fn router() -> Router {
  Router::new()
    .route(
      "/login",
      get(|Query(query): Query<RedirectQuery>| async {
        let uri = github_oauth_client()
          .as_ref()
          .context("Github Oauth not configured")
          .status_code(StatusCode::UNAUTHORIZED)?
          .get_login_redirect_url(query.redirect)
          .await;
        serror::Result::Ok(Redirect::to(&uri))
      }),
    )
    .route(
      "/callback",
      get(
        |query,
         headers: HeaderMap,
         ConnectInfo(info): ConnectInfo<SocketAddr>| async move {
          callback(query)
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

#[derive(Debug, Deserialize)]
struct CallbackQuery {
  state: String,
  code: String,
}

async fn callback(
  Query(query): Query<CallbackQuery>,
) -> anyhow::Result<Redirect> {
  let client = github_oauth_client().as_ref().unwrap();
  if !client.check_state(&query.state).await {
    return Err(anyhow!("state mismatch"));
  }
  let token = client.get_access_token(&query.code).await?;
  let github_user =
    client.get_github_user(&token.access_token).await?;
  let github_id = github_user.id.to_string();
  let db_client = db_client();
  let user = db_client
    .users
    .find_one(doc! { "config.data.github_id": &github_id })
    .await
    .context("failed at find user query from database")?;
  let jwt = match user {
    Some(user) => jwt_client()
      .encode(user.id)
      .context("failed to generate jwt")?,
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

      let user = User {
        id: Default::default(),
        username,
        enabled: no_users_exist || core_config.enable_new_users,
        admin: no_users_exist,
        super_admin: no_users_exist,
        create_server_permissions: no_users_exist,
        create_build_permissions: no_users_exist,
        updated_at: ts,
        last_update_view: 0,
        recents: Default::default(),
        all: Default::default(),
        config: UserConfig::Github {
          github_id,
          avatar: github_user.avatar_url,
        },
        totp: Default::default(),
        webauthn: Default::default()
      };
      let user_id = db_client
        .users
        .insert_one(user)
        .await
        .context("failed to create user on mongo")?
        .inserted_id
        .as_object_id()
        .context("inserted_id is not ObjectId")?
        .to_string();
      jwt_client()
        .encode(user_id)
        .context("failed to generate jwt")?
    }
  };
  let exchange_token = jwt_client().create_exchange_token(jwt).await;
  let redirect = &query.state[STATE_PREFIX_LENGTH..];
  let redirect_url = if redirect.is_empty() {
    format!("{}?token={exchange_token}", core_config().host)
  } else {
    let splitter = if redirect.contains('?') { '&' } else { '?' };
    format!("{redirect}{splitter}token={exchange_token}")
  };
  Ok(Redirect::to(&redirect_url))
}
