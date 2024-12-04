use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Context};
use komodo_client::{
  api::write::{
    AddUserToUserGroup, CreateUserGroup, DeleteUserGroup,
    RemoveUserFromUserGroup, RenameUserGroup, SetUsersInUserGroup,
  },
  entities::{komodo_timestamp, user_group::UserGroup},
};
use mungos::{
  by_id::{delete_one_by_id, find_one_by_id, update_one_by_id},
  find::find_collect,
  mongodb::bson::{doc, oid::ObjectId},
};
use resolver_api::Resolve;

use crate::state::db_client;

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }
    let user_group = UserGroup {
      id: Default::default(),
      users: Default::default(),
      all: Default::default(),
      updated_at: komodo_timestamp(),
      name: self.name,
    };
    let db = db_client();
    let id = db
      .user_groups
      .insert_one(user_group)
      .await
      .context("failed to create UserGroup on db")?
      .inserted_id
      .as_object_id()
      .context("inserted id is not ObjectId")?
      .to_string();
    let res = find_one_by_id(&db.user_groups, &id)
      .await
      .context("failed to query db for user groups")?
      .context("user group at id not found")?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for RenameUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }
    let db = db_client();
    update_one_by_id(
      &db.user_groups,
      &self.id,
      doc! { "$set": { "name": self.name } },
      None,
    )
    .await
    .context("failed to rename UserGroup on db")?;
    let res = find_one_by_id(&db.user_groups, &self.id)
      .await
      .context("failed to query db for UserGroups")?
      .context("no user group with given id")?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for DeleteUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }

    let db = db_client();

    let ug = find_one_by_id(&db.user_groups, &self.id)
      .await
      .context("failed to query db for UserGroups")?
      .context("no UserGroup found with given id")?;

    delete_one_by_id(&db.user_groups, &self.id, None)
      .await
      .context("failed to delete UserGroup from db")?;

    db.permissions
      .delete_many(doc! {
        "user_target.type": "UserGroup",
        "user_target.id": self.id,
      })
      .await
      .context("failed to clean up UserGroups permissions. User Group has been deleted")?;

    Ok(ug)
  }
}

impl Resolve<WriteArgs> for AddUserToUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }

    let db = db_client();

    let filter = match ObjectId::from_str(&self.user) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "username": &self.user },
    };
    let user = db
      .users
      .find_one(filter)
      .await
      .context("failed to query mongo for users")?
      .context("no matching user found")?;

    let filter = match ObjectId::from_str(&self.user_group) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "name": &self.user_group },
    };
    db.user_groups
      .update_one(
        filter.clone(),
        doc! { "$addToSet": { "users": &user.id } },
      )
      .await
      .context("failed to add user to group on db")?;
    let res = db
      .user_groups
      .find_one(filter)
      .await
      .context("failed to query db for UserGroups")?
      .context("no user group with given id")?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for RemoveUserFromUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }

    let db = db_client();

    let filter = match ObjectId::from_str(&self.user) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "username": &self.user },
    };
    let user = db
      .users
      .find_one(filter)
      .await
      .context("failed to query mongo for users")?
      .context("no matching user found")?;

    let filter = match ObjectId::from_str(&self.user_group) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "name": &self.user_group },
    };
    db.user_groups
      .update_one(
        filter.clone(),
        doc! { "$pull": { "users": &user.id } },
      )
      .await
      .context("failed to add user to group on db")?;
    let res = db
      .user_groups
      .find_one(filter)
      .await
      .context("failed to query db for UserGroups")?
      .context("no user group with given id")?;
    Ok(res)
  }
}

impl Resolve<WriteArgs> for SetUsersInUserGroup {
  async fn resolve(
    self,
    WriteArgs { user: admin }: &WriteArgs,
  ) -> serror::Result<UserGroup> {
    if !admin.admin {
      return Err(anyhow!("This call is admin-only").into());
    }

    let db = db_client();

    let all_users = find_collect(&db.users, None, None)
      .await
      .context("failed to query db for users")?
      .into_iter()
      .map(|u| (u.username, u.id))
      .collect::<HashMap<_, _>>();

    // Make sure all users are user ids
    let users = self
      .users
      .into_iter()
      .filter_map(|user| match ObjectId::from_str(&user) {
        Ok(_) => Some(user),
        Err(_) => all_users.get(&user).cloned(),
      })
      .collect::<Vec<_>>();

    let filter = match ObjectId::from_str(&self.user_group) {
      Ok(id) => doc! { "_id": id },
      Err(_) => doc! { "name": &self.user_group },
    };
    db.user_groups
      .update_one(filter.clone(), doc! { "$set": { "users": users } })
      .await
      .context("failed to set users on user group")?;
    let res = db
      .user_groups
      .find_one(filter)
      .await
      .context("failed to query db for UserGroups")?
      .context("no user group with given id")?;
    Ok(res)
  }
}
