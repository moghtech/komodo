use std::{collections::VecDeque, str::FromStr};

use anyhow::{Context, anyhow};
use async_timing_util::unix_timestamp_ms;
use database::{
  bson::to_bson,
  hash_password,
  mungos::{
    by_id::update_one_by_id,
    mongodb::bson::{doc, oid::ObjectId},
  },
};
use komodo_client::{
  api::write::*,
  entities::{
    komodo_timestamp,
    user::{NewUserParams, User, UserConfig},
  },
};
use mogh_error::{AddStatusCode as _, AddStatusCodeError};
use mogh_resolver::Resolve;
use reqwest::StatusCode;

use crate::{
  helpers::{
    query::get_user,
    validations::{validate_password, validate_username},
  },
  state::db_client,
};

use super::WriteArgs;

//

const RECENTLY_VIEWED_MAX: usize = 10;

impl Resolve<WriteArgs> for PushRecentlyViewed {
  #[instrument(
    "PushRecentlyViewed",
    level = "debug",
    skip_all,
    fields(
      user_id = user.id,
      resource = format!("{:?}", self.resource)
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user, .. }: &WriteArgs,
  ) -> mogh_error::Result<PushRecentlyViewedResponse> {
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

//

impl Resolve<WriteArgs> for SetLastSeenUpdate {
  #[instrument(
    "SetLastSeenUpdate",
    level = "debug",
    skip_all,
    fields(user_id = user.id)
  )]
  async fn resolve(
    self,
    WriteArgs { user, .. }: &WriteArgs,
  ) -> mogh_error::Result<SetLastSeenUpdateResponse> {
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

impl Resolve<WriteArgs> for CreateLocalUser {
  #[instrument(
    "CreateLocalUser",
    skip_all,
    fields(
      admin_id = admin.id,
      username = self.username
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> mogh_error::Result<CreateLocalUserResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This method is Admin Only.")
          .status_code(StatusCode::FORBIDDEN),
      );
    }

    validate_username(&self.username)
      .status_code(StatusCode::BAD_REQUEST)?;
    validate_password(&self.password)
      .status_code(StatusCode::BAD_REQUEST)?;

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

    let mut user = User::new(NewUserParams {
      username: self.username,
      enabled: true,
      admin: false,
      super_admin: false,
      config: UserConfig::Local {
        password: hashed_password,
      },
      updated_at: ts,
    });

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

impl Resolve<WriteArgs> for DeleteUser {
  #[instrument(
    "DeleteUser",
    skip_all,
    fields(
      admin_id = admin.id,
      user_to_delete = self.user
    )
  )]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> mogh_error::Result<DeleteUserResponse> {
    if !admin.admin {
      return Err(
        anyhow!("This method is admin-only.")
          .status_code(StatusCode::FORBIDDEN),
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

    // Also remove user id from all user groups
    if let Err(e) = db
      .user_groups
      .update_many(doc! {}, doc! { "$pull": { "users": &user.id } })
      .await
    {
      warn!("Failed to remove deleted user from user groups | {e:?}");
    };

    Ok(user)
  }
}
