use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    permission::PermissionLevel,
    sync::{
      ResourceSync, ResourceSyncActionState, ResourceSyncListItem,
    },
  },
};
use mogh_resolver::Resolve;

use crate::{
  helpers::query::get_all_tags, permission::get_check_permissions,
  resource, state::action_states,
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetResourceSync {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ResourceSync> {
    Ok(
      get_check_permissions::<ResourceSync>(
        &self.sync,
        user,
        PermissionLevel::Read.into(),
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListResourceSyncs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<Vec<ResourceSyncListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<ResourceSync>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullResourceSyncs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ListFullResourceSyncsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<ResourceSync>(
        self.query,
        user,
        PermissionLevel::Read.into(),
        &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetResourceSyncActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<ResourceSyncActionState> {
    let sync = get_check_permissions::<ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    let action_state = action_states()
      .sync
      .get(&sync.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetResourceSyncsSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> mogh_error::Result<GetResourceSyncsSummaryResponse> {
    let resource_syncs =
      resource::list_full_for_user::<ResourceSync>(
        Default::default(),
        user,
        PermissionLevel::Read.into(),
        &[],
      )
      .await
      .context("failed to get resource_syncs from db")?;

    let mut res = GetResourceSyncsSummaryResponse::default();

    let action_states = action_states();

    for resource_sync in resource_syncs {
      res.total += 1;

      if !(resource_sync.info.pending_deploys.is_empty()
        && resource_sync.info.resource_updates.is_empty()
        && resource_sync.info.variable_updates.is_empty()
        && resource_sync.info.user_group_updates.is_empty())
      {
        res.pending += 1;
        continue;
      } else if resource_sync.info.pending_error.is_some()
        || !resource_sync.info.remote_errors.is_empty()
      {
        res.failed += 1;
        continue;
      }
      if action_states
        .sync
        .get(&resource_sync.id)
        .await
        .unwrap_or_default()
        .get()?
        .syncing
      {
        res.syncing += 1;
        continue;
      }
      res.ok += 1;
    }

    Ok(res)
  }
}
