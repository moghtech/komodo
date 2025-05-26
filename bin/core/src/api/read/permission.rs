use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::{
    GetPermission, GetPermissionResponse, ListPermissions,
    ListPermissionsResponse, ListUserTargetPermissions,
    ListUserTargetPermissionsResponse,
  },
  entities::permission::PermissionLevel,
};
use mungos::{find::find_collect, mongodb::bson::doc};
use resolver_api::Resolve;

use crate::{
  helpers::query::get_user_permission_on_target, state::db_client,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for ListPermissions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListPermissionsResponse> {
    let res = find_collect(
      &db_client().permissions,
      doc! {
        "user_target.type": "User",
        "user_target.id": &user.id
      },
      None,
    )
    .await
    .context("failed to query db for permissions")?;
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetPermission {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetPermissionResponse> {
    if user.admin {
      return Ok(PermissionLevel::Write.all());
    }
    Ok(get_user_permission_on_target(user, &self.target).await?)
  }
}

impl Resolve<ReadArgs> for ListUserTargetPermissions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListUserTargetPermissionsResponse> {
    if !user.admin {
      return Err(anyhow!("this method is admin only").into());
    }
    let (variant, id) = self.user_target.extract_variant_id();
    let res = find_collect(
      &db_client().permissions,
      doc! {
        "user_target.type": variant.as_ref(),
        "user_target.id": id
      },
      None,
    )
    .await
    .context("failed to query db for permissions")?;
    Ok(res)
  }
}
