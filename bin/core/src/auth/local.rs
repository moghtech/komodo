use std::str::FromStr;

use anyhow::{anyhow, Context};
use async_timing_util::unix_timestamp_ms;
use komodo_client::{
  api::auth::{
    CreateLocalUser, CreateLocalUserResponse, LoginLocalUser,
    LoginLocalUserResponse,
  },
  entities::user::{User, UserConfig},
};
use mongo_indexed::Document;
use mungos::mongodb::bson::{doc, oid::ObjectId};
use resolver_api::Resolve;

use crate::{
  api::auth::AuthArgs,
  config::core_config,
  helpers::hash_password,
  state::{db_client, jwt_client},
};

impl Resolve<AuthArgs> for CreateLocalUser {
  #[instrument(name = "CreateLocalUser", skip(self))]
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<CreateLocalUserResponse> {
    let core_config = core_config();

    if !core_config.local_auth {
      return Err(anyhow!("Local auth is not enabled").into());
    }

    if self.username.is_empty() {
      return Err(anyhow!("Username cannot be empty string").into());
    }

    if ObjectId::from_str(&self.username).is_ok() {
      return Err(
        anyhow!("Username cannot be valid ObjectId").into(),
      );
    }

    if self.password.is_empty() {
      return Err(anyhow!("Password cannot be empty string").into());
    }

    let hashed_password = hash_password(self.password)?;

    let no_users_exist =
      db_client().users.find_one(Document::new()).await?.is_none();

    if !no_users_exist && core_config.disable_user_registration {
      return Err(anyhow!("User registration is disabled").into());
    }

    let ts = unix_timestamp_ms() as i64;

    let user = User {
      id: Default::default(),
      username: self.username,
      enabled: no_users_exist || core_config.enable_new_users,
      admin: no_users_exist,
      super_admin: no_users_exist,
      create_server_permissions: no_users_exist,
      create_build_permissions: no_users_exist,
      updated_at: ts,
      last_update_view: 0,
      recents: Default::default(),
      all: Default::default(),
      config: UserConfig::Local {
        password: hashed_password,
      },
    };

    let user_id = db_client()
      .users
      .insert_one(user)
      .await
      .context("failed to create user")?
      .inserted_id
      .as_object_id()
      .context("inserted_id is not ObjectId")?
      .to_string();

    let jwt = jwt_client()
      .generate(user_id)
      .context("failed to generate jwt for user")?;

    Ok(CreateLocalUserResponse { jwt })
  }
}

impl Resolve<AuthArgs> for LoginLocalUser {
  #[instrument(name = "LoginLocalUser", level = "debug", skip(self))]
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<LoginLocalUserResponse> {
    if !core_config().local_auth {
      return Err(anyhow!("local auth is not enabled").into());
    }

    let user = db_client()
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("failed at db query for users")?
      .with_context(|| {
        format!("did not find user with username {}", self.username)
      })?;

    let UserConfig::Local {
      password: user_pw_hash,
    } = user.config
    else {
      return Err(
        anyhow!(
          "non-local auth users can not log in with a password"
        )
        .into(),
      );
    };

    let verified = bcrypt::verify(self.password, &user_pw_hash)
      .context("failed at verify password")?;

    if !verified {
      return Err(anyhow!("invalid credentials").into());
    }

    let jwt = jwt_client()
      .generate(user.id)
      .context("failed at generating jwt for user")?;

    Ok(LoginLocalUserResponse { jwt })
  }
}
