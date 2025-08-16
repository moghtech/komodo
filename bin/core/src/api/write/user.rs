use std::str::FromStr;

use anyhow::{Context, anyhow};
use async_timing_util::unix_timestamp_ms;
use database::{
  hash_password,
  mungos::mongodb::bson::{doc, oid::ObjectId},
};
use komodo_client::{
  api::write::*,
  entities::{
    NoData,
    user::{User, UserConfig},
  },
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::AddStatusCodeError;

use crate::{config::core_config, state::db_client};

use super::WriteArgs;

//

impl Resolve<WriteArgs> for CreateLocalUser {
  #[instrument(name = "CreateLocalUser", skip(admin, self), fields(admin_id = admin.id, username = self.username))]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<CreateLocalUserResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This method is admin-only.")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    }

    if self.username.is_empty() {
      return Err(anyhow!("Username cannot be empty.").into());
    }

    if ObjectId::from_str(&self.username).is_ok() {
      return Err(
        anyhow!("Username cannot be valid ObjectId").into(),
      );
    }

    if self.password.is_empty() {
      return Err(anyhow!("Password cannot be empty.").into());
    }

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

    let ts = unix_timestamp_ms() as i64;
    let hashed_password = hash_password(self.password)?;

    let mut user = User {
      id: Default::default(),
      username: self.username,
      enabled: true,
      admin: false,
      super_admin: false,
      create_server_permissions: false,
      create_build_permissions: false,
      updated_at: ts,
      last_update_view: 0,
      recents: Default::default(),
      all: Default::default(),
      config: UserConfig::Local {
        password: hashed_password,
      },
    };

    user.id = db_client()
      .users
      .insert_one(&user)
      .await
      .context("failed to create user")?
      .inserted_id
      .as_object_id()
      .context("inserted_id is not ObjectId")?
      .to_string();

    user.sanitize();

    Ok(user)
  }
}

//

impl Resolve<WriteArgs> for UpdateUserUsername {
  #[instrument(name = "UpdateUserUsername", skip(user), fields(user_id = user.id))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateUserUsernameResponse> {
    for locked_username in &core_config().lock_login_credentials_for {
      if locked_username == "__ALL__"
        || *locked_username == user.username
      {
        return Err(
          anyhow!("User not allowed to update their username.")
            .into(),
        );
      }
    }
    if self.username.is_empty() {
      return Err(anyhow!("Username cannot be empty.").into());
    }

    if ObjectId::from_str(&self.username).is_ok() {
      return Err(
        anyhow!("Username cannot be valid ObjectId").into(),
      );
    }

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
    let id = ObjectId::from_str(&user.id)
      .context("User id not valid ObjectId.")?;
    db.users
      .update_one(
        doc! { "_id": id },
        doc! { "$set": { "username": self.username } },
      )
      .await
      .context("Failed to update user username on database.")?;
    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for UpdateUserPassword {
  #[instrument(name = "UpdateUserPassword", skip(user, self), fields(user_id = user.id))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateUserPasswordResponse> {
    for locked_username in &core_config().lock_login_credentials_for {
      if locked_username == "__ALL__"
        || *locked_username == user.username
      {
        return Err(
          anyhow!("User not allowed to update their password.")
            .into(),
        );
      }
    }
    db_client().set_user_password(user, &self.password).await?;
    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for DeleteUser {
  #[instrument(name = "DeleteUser", skip(admin), fields(user = self.user))]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<DeleteUserResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This method is admin-only.")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    }
    if admin.username == self.user || admin.id == self.user {
      return Err(anyhow!("User cannot delete themselves.").into());
    }
    let query = if let Ok(id) = ObjectId::from_str(&self.user) {
      doc! { "_id": id }
    } else {
      doc! { "username": self.user }
    };
    let db = db_client();
    let Some(user) = db
      .users
      .find_one(query.clone())
      .await
      .context("Failed to query database for users.")?
    else {
      return Err(
        anyhow!("No user found with given id / username").into(),
      );
    };
    if user.super_admin {
      return Err(
        anyhow!("Cannot delete a super admin user.").into(),
      );
    }
    if user.admin && !admin.super_admin {
      return Err(
        anyhow!("Only a Super Admin can delete an admin user.")
          .into(),
      );
    }
    db.users
      .delete_one(query)
      .await
      .context("Failed to delete user from database")?;
    Ok(user)
  }
}
