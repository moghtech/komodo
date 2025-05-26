use std::str::FromStr;

use anyhow::{Context, anyhow};
use komodo_client::{
  api::write::*,
  entities::{
    ResourceTarget, ResourceTargetVariant,
    permission::{UserTarget, UserTargetVariant},
  },
};
use mungos::{
  by_id::{find_one_by_id, update_one_by_id},
  mongodb::{
    bson::{Document, doc, oid::ObjectId, to_bson},
    options::UpdateOptions,
  },
};
use resolver_api::Resolve;

use crate::{helpers::query::get_user, state::db_client};

use super::WriteArgs;

impl Resolve<WriteArgs> for UpdateUserAdmin {
  #[instrument(name = "UpdateUserAdmin", skip(super_admin))]
  async fn resolve(
    self,
    WriteArgs { user: super_admin }: &WriteArgs,
  ) -> serror::Result<UpdateUserAdminResponse> {
    if !super_admin.super_admin {
      return Err(
        anyhow!("Only super admins can call this method.").into(),
      );
    }
    let user = find_one_by_id(&db_client().users, &self.user_id)
      .await
      .context("failed to query mongo for user")?
      .context("did not find user with given id")?;

    if !user.enabled {
      return Err(
        anyhow!("User is disabled. Enable user first.").into(),
      );
    }

    if user.super_admin {
      return Err(anyhow!("Cannot update other super admins").into());
    }

    update_one_by_id(
      &db_client().users,
      &self.user_id,
      doc! { "$set": { "admin": self.admin } },
      None,
    )
    .await?;

    Ok(UpdateUserAdminResponse {})
  }
}

impl Resolve<WriteArgs> for UpdateUserBasePermissions {
  #[instrument(name = "UpdateUserBasePermissions", skip(admin))]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UpdateUserBasePermissionsResponse> {
    if !admin.admin {
      return Err(anyhow!("this method is admin only").into());
    }

    let UpdateUserBasePermissions {
      user_id,
      enabled,
      create_servers,
      create_builds,
    } = self;

    let user = find_one_by_id(&db_client().users, &user_id)
      .await
      .context("failed to query mongo for user")?
      .context("did not find user with given id")?;
    if user.super_admin {
      return Err(
        anyhow!(
          "Cannot use this method to update super admins permissions"
        )
        .into(),
      );
    }
    if user.admin && !admin.super_admin {
      return Err(anyhow!(
        "Only super admins can use this method to update other admins permissions"
      ).into());
    }
    let mut update_doc = Document::new();
    if let Some(enabled) = enabled {
      update_doc.insert("enabled", enabled);
    }
    if let Some(create_servers) = create_servers {
      update_doc.insert("create_server_permissions", create_servers);
    }
    if let Some(create_builds) = create_builds {
      update_doc.insert("create_build_permissions", create_builds);
    }

    update_one_by_id(
      &db_client().users,
      &user_id,
      mungos::update::Update::Set(update_doc),
      None,
    )
    .await?;

    Ok(UpdateUserBasePermissionsResponse {})
  }
}

impl Resolve<WriteArgs> for UpdatePermissionOnResourceType {
  #[instrument(name = "UpdatePermissionOnResourceType", skip(admin))]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UpdatePermissionOnResourceTypeResponse> {
    if !admin.admin {
      return Err(anyhow!("this method is admin only").into());
    }

    let Self {
      user_target,
      resource_type,
      permission,
    } = self;

    // Some extra checks if user target is an actual User
    if let UserTarget::User(user_id) = &user_target {
      let user = get_user(user_id).await?;
      if user.admin {
        return Err(
          anyhow!(
          "cannot use this method to update other admins permissions"
        )
          .into(),
        );
      }
      if !user.enabled {
        return Err(anyhow!("user not enabled").into());
      }
    }

    let (user_target_variant, user_target_id) =
      extract_user_target_with_validation(&user_target).await?;

    let id = ObjectId::from_str(&user_target_id)
      .context("id is not ObjectId")?;
    let filter = doc! { "_id": id };
    let field = format!("all.{resource_type}");
    let set =
      to_bson(&permission).context("permission is not Bson")?;
    let update = doc! { "$set": { &field: &set } };

    match user_target_variant {
      UserTargetVariant::User => {
        db_client()
          .users
          .update_one(filter, update)
          .await
          .with_context(|| {
            format!("failed to set {field}: {set} on db")
          })?;
      }
      UserTargetVariant::UserGroup => {
        db_client()
          .user_groups
          .update_one(filter, update)
          .await
          .with_context(|| {
            format!("failed to set {field}: {set} on db")
          })?;
      }
    }

    Ok(UpdatePermissionOnResourceTypeResponse {})
  }
}

impl Resolve<WriteArgs> for UpdatePermissionOnTarget {
  #[instrument(name = "UpdatePermissionOnTarget", skip(admin))]
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UpdatePermissionOnTargetResponse> {
    if !admin.admin {
      return Err(anyhow!("this method is admin only").into());
    }

    let UpdatePermissionOnTarget {
      user_target,
      resource_target,
      permission,
    } = self;

    // Some extra checks relevant if user target is an actual User
    if let UserTarget::User(user_id) = &user_target {
      let user = get_user(user_id).await?;
      if !user.enabled {
        return Err(anyhow!("user not enabled").into());
      }
      if user.admin {
        return Err(
          anyhow!(
          "cannot use this method to update other admins permissions"
        )
          .into(),
        );
      }
    }

    let (user_target_variant, user_target_id) =
      extract_user_target_with_validation(&user_target).await?;
    let (resource_variant, resource_id) =
      extract_resource_target_with_validation(&resource_target)
        .await?;

    let (user_target_variant, resource_variant) =
      (user_target_variant.as_ref(), resource_variant.as_ref());

    let specific = to_bson(&permission.specific)
      .context("permission.specific is not valid Bson")?;
    
    db_client()
      .permissions
      .update_one(
        doc! {
          "user_target.type": user_target_variant,
          "user_target.id": &user_target_id,
          "resource_target.type": resource_variant,
          "resource_target.id": &resource_id
        },
        doc! {
          "$set": {
            "user_target.type": user_target_variant,
            "user_target.id": user_target_id,
            "resource_target.type": resource_variant,
            "resource_target.id": resource_id,
            "level": permission.level.as_ref(),
            "specific": specific
          }
        },
      )
      .with_options(UpdateOptions::builder().upsert(true).build())
      .await?;

    Ok(UpdatePermissionOnTargetResponse {})
  }
}

/// checks if inner id is actually a `name`, and replaces it with id if so.
async fn extract_user_target_with_validation(
  user_target: &UserTarget,
) -> serror::Result<(UserTargetVariant, String)> {
  match user_target {
    UserTarget::User(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "username": ident },
      };
      let id = db_client()
        .users
        .find_one(filter)
        .await
        .context("failed to query db for users")?
        .context("no matching user found")?
        .id;
      Ok((UserTargetVariant::User, id))
    }
    UserTarget::UserGroup(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .user_groups
        .find_one(filter)
        .await
        .context("failed to query db for user_groups")?
        .context("no matching user_group found")?
        .id;
      Ok((UserTargetVariant::UserGroup, id))
    }
  }
}

/// checks if inner id is actually a `name`, and replaces it with id if so.
async fn extract_resource_target_with_validation(
  resource_target: &ResourceTarget,
) -> serror::Result<(ResourceTargetVariant, String)> {
  match resource_target {
    ResourceTarget::System(_) => {
      let res = resource_target.extract_variant_id();
      Ok((res.0, res.1.clone()))
    }
    ResourceTarget::Build(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .builds
        .find_one(filter)
        .await
        .context("failed to query db for builds")?
        .context("no matching build found")?
        .id;
      Ok((ResourceTargetVariant::Build, id))
    }
    ResourceTarget::Builder(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .builders
        .find_one(filter)
        .await
        .context("failed to query db for builders")?
        .context("no matching builder found")?
        .id;
      Ok((ResourceTargetVariant::Builder, id))
    }
    ResourceTarget::Deployment(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .deployments
        .find_one(filter)
        .await
        .context("failed to query db for deployments")?
        .context("no matching deployment found")?
        .id;
      Ok((ResourceTargetVariant::Deployment, id))
    }
    ResourceTarget::Server(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .servers
        .find_one(filter)
        .await
        .context("failed to query db for servers")?
        .context("no matching server found")?
        .id;
      Ok((ResourceTargetVariant::Server, id))
    }
    ResourceTarget::Repo(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .repos
        .find_one(filter)
        .await
        .context("failed to query db for repos")?
        .context("no matching repo found")?
        .id;
      Ok((ResourceTargetVariant::Repo, id))
    }
    ResourceTarget::Alerter(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .alerters
        .find_one(filter)
        .await
        .context("failed to query db for alerters")?
        .context("no matching alerter found")?
        .id;
      Ok((ResourceTargetVariant::Alerter, id))
    }
    ResourceTarget::Procedure(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .procedures
        .find_one(filter)
        .await
        .context("failed to query db for procedures")?
        .context("no matching procedure found")?
        .id;
      Ok((ResourceTargetVariant::Procedure, id))
    }
    ResourceTarget::Action(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .actions
        .find_one(filter)
        .await
        .context("failed to query db for actions")?
        .context("no matching action found")?
        .id;
      Ok((ResourceTargetVariant::Action, id))
    }
    ResourceTarget::ResourceSync(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .resource_syncs
        .find_one(filter)
        .await
        .context("failed to query db for resource syncs")?
        .context("no matching resource sync found")?
        .id;
      Ok((ResourceTargetVariant::ResourceSync, id))
    }
    ResourceTarget::Stack(ident) => {
      let filter = match ObjectId::from_str(ident) {
        Ok(id) => doc! { "_id": id },
        Err(_) => doc! { "name": ident },
      };
      let id = db_client()
        .stacks
        .find_one(filter)
        .await
        .context("failed to query db for stacks")?
        .context("no matching stack found")?
        .id;
      Ok((ResourceTargetVariant::Stack, id))
    }
  }
}
