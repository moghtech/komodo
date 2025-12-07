use anyhow::{Context, anyhow};
use database::mungos::{by_id::find_one_by_id, mongodb::bson::doc};
use komodo_client::{
  api::{user::CreateApiKey, write::*},
  entities::{
    komodo_timestamp,
    user::{User, UserConfig},
  },
};
use reqwest::StatusCode;
use resolver_api::Resolve;
use serror::{AddStatusCode as _, AddStatusCodeError as _};

use crate::{
  api::user::UserArgs,
  helpers::validations::{validate_api_key_name, validate_username},
  state::db_client,
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateServiceUser {
  #[instrument(
    "CreateServiceUser",
    skip_all,
    fields(
      operator = user.id,
      username = self.username,
      description = self.description,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateServiceUserResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can manage Service Users")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    validate_username(&self.username)
      .status_code(StatusCode::BAD_REQUEST)?;

    let config = UserConfig::Service {
      description: self.description,
    };

    let mut user = User {
      id: Default::default(),
      username: self.username,
      config,
      totp: Default::default(),
      webauthn: Default::default(),
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
  #[instrument(
    "UpdateServiceUserDescription",
    skip_all,
    fields(
      operator = user.id,
      username = self.username,
      description = self.description,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<UpdateServiceUserDescriptionResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can manage Service Users")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    let db = db_client();

    let service_user = db
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("Failed to query db for user")?
      .context("No user with given username")?;

    let UserConfig::Service { .. } = &service_user.config else {
      return Err(
        anyhow!("Target user is not Service User")
          .status_code(StatusCode::FORBIDDEN),
      );
    };

    db.users
      .update_one(
        doc! { "username": &self.username },
        doc! { "$set": { "config.data.description": self.description } },
      )
      .await
      .context("failed to update user on db")?;

    let service_user = db
      .users
      .find_one(doc! { "username": &self.username })
      .await
      .context("failed to query db for user")?
      .context("user with username not found")?;

    Ok(service_user)
  }
}

impl Resolve<WriteArgs> for CreateApiKeyForServiceUser {
  #[instrument(
    "CreateApiKeyForServiceUser",
    skip_all,
    fields(
      operator = user.id,
      service_user = self.user_id,
      name = self.name,
      expires = self.expires,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<CreateApiKeyForServiceUserResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can manage Service Users")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    validate_api_key_name(&self.name)
      .status_code(StatusCode::BAD_REQUEST)?;

    let service_user =
      find_one_by_id(&db_client().users, &self.user_id)
        .await
        .context("Failed to query db for user")?
        .context("No user found with id")?;

    let UserConfig::Service { .. } = &service_user.config else {
      return Err(
        anyhow!("Target user is not Service User")
          .status_code(StatusCode::FORBIDDEN),
      );
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
  #[instrument(
    "DeleteApiKeyForServiceUser",
    skip_all,
    fields(
      operator = user.id,
      key = self.key,
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteApiKeyForServiceUserResponse> {
    if !user.admin {
      return Err(
        anyhow!("Only Admins can manage Service Users")
          .status_code(StatusCode::FORBIDDEN),
      );
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
      return Err(
        anyhow!("Target user is not Service User")
          .status_code(StatusCode::FORBIDDEN),
      );
    };

    db.api_keys
      .delete_one(doc! { "key": self.key })
      .await
      .context("failed to delete api key on db")?;
    Ok(DeleteApiKeyForServiceUserResponse {})
  }
}
