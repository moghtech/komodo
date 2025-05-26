use std::collections::HashMap;

use anyhow::{Context, anyhow};
use komodo_client::{
  api::read::{GetUpdate, ListUpdates, ListUpdatesResponse},
  entities::{
    ResourceTarget,
    action::Action,
    alerter::Alerter,
    build::Build,
    builder::Builder,
    deployment::Deployment,
    permission::PermissionLevel,
    procedure::Procedure,
    repo::Repo,
    server::Server,
    stack::Stack,
    sync::ResourceSync,
    update::{Update, UpdateListItem},
    user::User,
  },
};
use mungos::{
  by_id::find_one_by_id,
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  permission::{get_check_permissions, get_resource_ids_for_user},
  state::db_client,
};

use super::ReadArgs;

const UPDATES_PER_PAGE: i64 = 100;

impl Resolve<ReadArgs> for ListUpdates {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListUpdatesResponse> {
    let query = if user.admin || core_config().transparent_mode {
      self.query
    } else {
      let server_query = get_resource_ids_for_user::<Server>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Server", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Server" });

      let deployment_query =
        get_resource_ids_for_user::<Deployment>(user)
          .await?
          .map(|ids| {
            doc! {
              "target.type": "Deployment", "target.id": { "$in": ids }
            }
          })
          .unwrap_or_else(|| doc! { "target.type": "Deployment" });

      let stack_query = get_resource_ids_for_user::<Stack>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Stack", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Stack" });

      let build_query = get_resource_ids_for_user::<Build>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Build", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Build" });

      let repo_query = get_resource_ids_for_user::<Repo>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Repo", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Repo" });

      let procedure_query =
        get_resource_ids_for_user::<Procedure>(user)
          .await?
          .map(|ids| {
            doc! {
              "target.type": "Procedure", "target.id": { "$in": ids }
            }
          })
          .unwrap_or_else(|| doc! { "target.type": "Procedure" });

      let action_query = get_resource_ids_for_user::<Action>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Action", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Action" });

      let builder_query = get_resource_ids_for_user::<Builder>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Builder", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Builder" });

      let alerter_query = get_resource_ids_for_user::<Alerter>(user)
        .await?
        .map(|ids| {
          doc! {
            "target.type": "Alerter", "target.id": { "$in": ids }
          }
        })
        .unwrap_or_else(|| doc! { "target.type": "Alerter" });

      let resource_sync_query = get_resource_ids_for_user::<
        ResourceSync,
      >(user)
      .await?
      .map(|ids| {
        doc! {
          "target.type": "ResourceSync", "target.id": { "$in": ids }
        }
      })
      .unwrap_or_else(|| doc! { "target.type": "ResourceSync" });

      let mut query = self.query.unwrap_or_default();
      query.extend(doc! {
        "$or": [
          server_query,
          deployment_query,
          stack_query,
          build_query,
          repo_query,
          procedure_query,
          action_query,
          alerter_query,
          builder_query,
          resource_sync_query,
        ]
      });
      query.into()
    };

    let usernames = find_collect(&db_client().users, None, None)
      .await
      .context("failed to pull users from db")?
      .into_iter()
      .map(|u| (u.id, u.username))
      .collect::<HashMap<_, _>>();

    let updates = find_collect(
      &db_client().updates,
      query,
      FindOptions::builder()
        .sort(doc! { "start_ts": -1 })
        .skip(self.page as u64 * UPDATES_PER_PAGE as u64)
        .limit(UPDATES_PER_PAGE)
        .build(),
    )
    .await
    .context("failed to pull updates from db")?
    .into_iter()
    .map(|u| {
      let username = if User::is_service_user(&u.operator) {
        u.operator.clone()
      } else {
        usernames
          .get(&u.operator)
          .cloned()
          .unwrap_or("unknown".to_string())
      };
      UpdateListItem {
        username,
        id: u.id,
        operation: u.operation,
        start_ts: u.start_ts,
        success: u.success,
        operator: u.operator,
        target: u.target,
        status: u.status,
        version: u.version,
        other_data: u.other_data,
      }
    })
    .collect::<Vec<_>>();

    let next_page = if updates.len() == UPDATES_PER_PAGE as usize {
      Some(self.page + 1)
    } else {
      None
    };

    Ok(ListUpdatesResponse { updates, next_page })
  }
}

impl Resolve<ReadArgs> for GetUpdate {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Update> {
    let update = find_one_by_id(&db_client().updates, &self.id)
      .await
      .context("failed to query to db")?
      .context("no update exists with given id")?;
    if user.admin || core_config().transparent_mode {
      return Ok(update);
    }
    match &update.target {
      ResourceTarget::System(_) => {
        return Err(
          anyhow!("user must be admin to view system updates").into(),
        );
      }
      ResourceTarget::Server(id) => {
        get_check_permissions::<Server>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Deployment(id) => {
        get_check_permissions::<Deployment>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Build(id) => {
        get_check_permissions::<Build>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Repo(id) => {
        get_check_permissions::<Repo>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Builder(id) => {
        get_check_permissions::<Builder>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Alerter(id) => {
        get_check_permissions::<Alerter>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Procedure(id) => {
        get_check_permissions::<Procedure>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Action(id) => {
        get_check_permissions::<Action>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::ResourceSync(id) => {
        get_check_permissions::<ResourceSync>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
      ResourceTarget::Stack(id) => {
        get_check_permissions::<Stack>(
          id,
          user,
          PermissionLevel::Read.into(),
        )
        .await?;
      }
    }
    Ok(update)
  }
}
