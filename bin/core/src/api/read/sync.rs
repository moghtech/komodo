use anyhow::Context;
use komodo_client::{
  api::read::*,
  entities::{
    config::core::CoreConfig,
    permission::PermissionLevel,
    sync::{
      ResourceSync, ResourceSyncActionState, ResourceSyncListItem,
      ResourceSyncState,
    },
  },
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::query::get_all_tags,
  resource,
  state::{action_states, github_client, resource_sync_state_cache},
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetResourceSync {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ResourceSync> {
    Ok(
      resource::get_check_permissions::<ResourceSync>(
        &self.sync,
        &user,
        PermissionLevel::Read,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListResourceSyncs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<ResourceSyncListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<ResourceSync>(
        self.query, &user, &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullResourceSyncs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullResourceSyncsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<ResourceSync>(
        self.query, &user, &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetResourceSyncActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ResourceSyncActionState> {
    let sync = resource::get_check_permissions::<ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Read,
    )
    .await?;
    let action_state = action_states()
      .resource_sync
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
  ) -> serror::Result<GetResourceSyncsSummaryResponse> {
    let resource_syncs =
      resource::list_full_for_user::<ResourceSync>(
        Default::default(),
        user,
        &[],
      )
      .await
      .context("failed to get resource_syncs from db")?;

    let mut res = GetResourceSyncsSummaryResponse::default();

    let cache = resource_sync_state_cache();
    let action_states = action_states();

    for resource_sync in resource_syncs {
      res.total += 1;

      if !(resource_sync.info.pending_deploy.to_deploy == 0
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

      match (
        cache.get(&resource_sync.id).await.unwrap_or_default(),
        action_states
          .resource_sync
          .get(&resource_sync.id)
          .await
          .unwrap_or_default()
          .get()?,
      ) {
        (_, action_states) if action_states.syncing => {
          res.syncing += 1;
        }
        (ResourceSyncState::Ok, _) => res.ok += 1,
        (ResourceSyncState::Failed, _) => res.failed += 1,
        (ResourceSyncState::Unknown, _) => res.unknown += 1,
        // will never come off the cache in the building state, since that comes from action states
        (ResourceSyncState::Syncing, _) => {
          unreachable!()
        }
        (ResourceSyncState::Pending, _) => {
          unreachable!()
        }
      }
    }

    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetSyncWebhooksEnabled {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetSyncWebhooksEnabledResponse> {
    let Some(github) = github_client() else {
      return Ok(GetSyncWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        sync_enabled: false,
      });
    };

    let sync = resource::get_check_permissions::<ResourceSync>(
      &self.sync,
      user,
      PermissionLevel::Read,
    )
    .await?;

    if sync.config.git_provider != "github.com"
      || sync.config.repo.is_empty()
    {
      return Ok(GetSyncWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        sync_enabled: false,
      });
    }

    let mut split = sync.config.repo.split('/');
    let owner = split.next().context("Sync repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Ok(GetSyncWebhooksEnabledResponse {
        managed: false,
        refresh_enabled: false,
        sync_enabled: false,
      });
    };

    let repo_name =
      split.next().context("Repo repo has no repo after the /")?;

    let github_repos = github.repos();

    let webhooks = github_repos
      .list_all_webhooks(owner, repo_name)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      ..
    } = core_config();

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let refresh_url =
      format!("{host}/listener/github/sync/{}/refresh", sync.id);
    let sync_url =
      format!("{host}/listener/github/sync/{}/sync", sync.id);

    let mut refresh_enabled = false;
    let mut sync_enabled = false;

    for webhook in webhooks {
      if webhook.active && webhook.config.url == refresh_url {
        refresh_enabled = true
      }
      if webhook.active && webhook.config.url == sync_url {
        sync_enabled = true
      }
    }

    Ok(GetSyncWebhooksEnabledResponse {
      managed: true,
      refresh_enabled,
      sync_enabled,
    })
  }
}
