use std::str::FromStr;

use anyhow::{anyhow, Context};
use komodo_client::{
  api::{user::CreateApiKey, write::*},
  entities::{
    komodo_timestamp,
    user::{User, UserConfig},
  },
};
use mungos::{
  by_id::find_one_by_id,
  mongodb::bson::{doc, oid::ObjectId},
};
use resolver_api::Resolve;

use crate::{api::user::UserArgs, state::db_client};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateServiceUser {
  #[instrument(name = "CreateServiceUser", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateServiceUserResponse> {
    if !user.admin {
      return Err(anyhow!("user not admin").into());
    }
    if ObjectId::from_str(&self.username).is_ok() {
      return Err(
        anyhow!("username cannot be valid ObjectId").into(),
      );
    }
    let config = UserConfig::Service {
      description: self.description,
    };
    let mut user = User {
      id: Default::default(),
      username: self.username,
      config,
      enabled: true,
      admin: false,
      super_admin: false,
      create_server_permissions: false,
      create_build_permissions: false,
      last_update_view: 0,
      recents: Default::default(),
      all: Default::default(),
      updated_at: komodo_timestamp(),
    };
    user.id = db_client()
      .users
      .insert_one(&user)
      .await
      .context("failed to create service user on db")?
      .inserted_id
      .as_object_id()
      .context("inserted id is not object id")?
      .to_string();
    Ok(user)
  }
}

impl Resolve<WriteArgs> for UpdateServiceUserDescription {
  #[instrument(name = "UpdateServiceUserDescription", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateServiceUserDescriptionResponse> {
    if !user.admin {
      return Err(anyhow!("user not admin").into());
    }
    let db = db_client();
    let service_user = db
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("failed to query db for user")?
      .context("no user with given username")?;
    let UserConfig::Service { .. } = &service_user.config else {
      return Err(anyhow!("user is not service user").into());
    };
    db.users
      .update_one(
        doc! { "username": &self.username },
        doc! { "$set": { "config.data.description": self.description } },
      )
      .await
      .context("failed to update user on db")?;
    let res = db
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("failed to query db for user")?
      .context("user with username not found")?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for CreateApiKeyForServiceUser {
  #[instrument(name = "CreateApiKeyForServiceUser", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateApiKeyForServiceUserResponse> {
    if !user.admin {
      return Err(anyhow!("user not admin").into());
    }
    let service_user =
      find_one_by_id(&db_client().users, &self.user_id)
        .await
        .context("failed to query db for user")?
        .context("no user found with id")?;
    let UserConfig::Service { .. } = &service_user.config else {
      return Err(anyhow!("user is not service user").into());
    };
    CreateApiKey {
      name: self.name,
      expires: self.expires,
    }
    .resolve(&UserArgs { user: service_user })
    .await
  }
}

impl Resolve<WriteArgs> for DeleteApiKeyForServiceUser {
  #[instrument(name = "DeleteApiKeyForServiceUser", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteApiKeyForServiceUserResponse> {
    if !user.admin {
      return Err(anyhow!("user not admin").into());
    }
    let db = db_client();
    let api_key = db
      .api_keys
      .find_one(doc! { "key": &self.key })
      .await
      .context("failed to query db for api key")?
      .context("did not find matching api key")?;
    let service_user =
      find_one_by_id(&db_client().users, &api_key.user_id)
        .await
        .context("failed to query db for user")?
        .context("no user found with id")?;
    let UserConfig::Service { .. } = &service_user.config else {
      return Err(anyhow!("user is not service user").into());
    };
    db.api_keys
      .delete_one(doc! { "key": self.key })
      .await
      .context("failed to delete api key on db")?;
    Ok(DeleteApiKeyForServiceUserResponse {})
  }
}
