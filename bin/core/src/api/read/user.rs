use anyhow::{anyhow, Context};
use komodo_client::{
  api::read::{
    FindUser, FindUserResponse, GetUsername, GetUsernameResponse,
    ListApiKeys, ListApiKeysForServiceUser,
    ListApiKeysForServiceUserResponse, ListApiKeysResponse,
    ListUsers, ListUsersResponse,
  },
  entities::user::{admin_service_user, UserConfig},
};
use mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use resolver_api::Resolve;

use crate::{helpers::query::get_user, state::db_client};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetUsername {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> serror::Result<GetUsernameResponse> {
    if let Some(user) = admin_service_user(&self.user_id) {
      return Ok(GetUsernameResponse {
        username: user.username,
        avatar: None,
      });
    }

    let user = find_one_by_id(&db_client().users, &self.user_id)
      .await
      .context("failed at mongo query for user")?
      .context("no user found with id")?;

    let avatar = match user.config {
      UserConfig::Github { avatar, .. } => Some(avatar),
      UserConfig::Google { avatar, .. } => Some(avatar),
      _ => None,
    };

    Ok(GetUsernameResponse {
      username: user.username,
      avatar,
    })
  }
}

impl Resolve<ReadArgs> for FindUser {
  async fn resolve(
    self,
    ReadArgs { user: admin }: &ReadArgs,
  ) -> serror::Result<FindUserResponse> {
    if !admin.admin {
      return Err(anyhow!("This method is admin only.").into());
    }
    Ok(get_user(&self.user).await?)
  }
}

impl Resolve<ReadArgs> for ListUsers {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListUsersResponse> {
    if !user.admin {
      return Err(
        anyhow!("this route is only accessable by admins").into(),
      );
    }
    let mut users = find_collect(
      &db_client().users,
      None,
      FindOptions::builder().sort(doc! { "username": 1 }).build(),
    )
    .await
    .context("failed to pull users from db")?;
    users.iter_mut().for_each(|user| user.sanitize());
    Ok(users)
  }
}

impl Resolve<ReadArgs> for ListApiKeys {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListApiKeysResponse> {
    let api_keys = find_collect(
      &db_client().api_keys,
      doc! { "user_id": &user.id },
      FindOptions::builder().sort(doc! { "name": 1 }).build(),
    )
    .await
    .context("failed to query db for api keys")?
    .into_iter()
    .map(|mut api_keys| {
      api_keys.sanitize();
      api_keys
    })
    .collect();
    Ok(api_keys)
  }
}

impl Resolve<ReadArgs> for ListApiKeysForServiceUser {
  async fn resolve(
    self,
    ReadArgs { user: admin }: &ReadArgs,
  ) -> serror::Result<ListApiKeysForServiceUserResponse> {
    if !admin.admin {
      return Err(anyhow!("This method is admin only.").into());
    }

    let user = get_user(&self.user).await?;

    let UserConfig::Service { .. } = user.config else {
      return Err(anyhow!("Given user is not service user").into());
    };
    let api_keys = find_collect(
      &db_client().api_keys,
      doc! { "user_id": &user.id },
      None,
    )
    .await
    .context("failed to query db for api keys")?
    .into_iter()
    .map(|mut api_keys| {
      api_keys.sanitize();
      api_keys
    })
    .collect();
    Ok(api_keys)
  }
}
