use std::str::FromStr;

use anyhow::{anyhow, Context};
use komodo_client::{
  api::write::{
    DeleteUser, DeleteUserResponse, UpdateUserPassword,
    UpdateUserPasswordResponse, UpdateUserUsername,
    UpdateUserUsernameResponse,
  },
  entities::{user::UserConfig, NoData},
};
use mungos::mongodb::bson::{doc, oid::ObjectId};
use resolver_api::Resolve;

use crate::{helpers::hash_password, state::db_client};

use super::WriteArgs;

//

impl Resolve<WriteArgs> for UpdateUserUsername {
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateUserUsernameResponse> {
    if self.username.is_empty() {
      return Err(anyhow!("Username cannot be empty.").into());
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
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateUserPasswordResponse> {
    let UserConfig::Local { .. } = user.config else {
      return Err(anyhow!("User is not local user").into());
    };
    if self.password.is_empty() {
      return Err(anyhow!("Password cannot be empty.").into());
    }
    let id = ObjectId::from_str(&user.id)
      .context("User id not valid ObjectId.")?;
    let hashed_password = hash_password(self.password)?;
    db_client()
      .users
      .update_one(
        doc! { "_id": id },
        doc! { "$set": {
          "config.data.password": hashed_password
        } },
      )
      .await
      .context("Failed to update user password on database.")?;
    Ok(NoData {})
  }
}

//

impl Resolve<WriteArgs> for DeleteUser {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<DeleteUserResponse> {
    if !admin.admin {
      return Err(anyhow!("Calling user is not admin.").into());
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
