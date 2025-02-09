use std::collections::{HashMap, HashSet};

use anyhow::Context;
use async_timing_util::unix_timestamp_ms;
use futures::TryStreamExt;
use komodo_client::{
  api::read::*,
  entities::{
    build::{Build, BuildActionState, BuildListItem, BuildState},
    config::core::CoreConfig,
    permission::PermissionLevel,
    update::UpdateStatus,
    Operation,
  },
};
use mungos::{
  find::find_collect,
  mongodb::{bson::doc, options::FindOptions},
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::query::get_all_tags,
  resource,
  state::{
    action_states, build_state_cache, db_client, github_client,
  },
};

use super::ReadArgs;

impl Resolve<ReadArgs> for GetBuild {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Build> {
    Ok(
      resource::get_check_permissions::<Build>(
        &self.build,
        user,
        PermissionLevel::Read,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListBuilds {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<BuildListItem>> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_for_user::<Build>(self.query, user, &all_tags)
        .await?,
    )
  }
}

impl Resolve<ReadArgs> for ListFullBuilds {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListFullBuildsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    Ok(
      resource::list_full_for_user::<Build>(
        self.query, user, &all_tags,
      )
      .await?,
    )
  }
}

impl Resolve<ReadArgs> for GetBuildActionState {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<BuildActionState> {
    let build = resource::get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Read,
    )
    .await?;
    let action_state = action_states()
      .build
      .get(&build.id)
      .await
      .unwrap_or_default()
      .get()?;
    Ok(action_state)
  }
}

impl Resolve<ReadArgs> for GetBuildsSummary {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetBuildsSummaryResponse> {
    let builds = resource::list_full_for_user::<Build>(
      Default::default(),
      user,
      &[],
    )
    .await
    .context("failed to get all builds")?;

    let mut res = GetBuildsSummaryResponse::default();

    let cache = build_state_cache();
    let action_states = action_states();

    for build in builds {
      res.total += 1;

      match (
        cache.get(&build.id).await.unwrap_or_default(),
        action_states
          .build
          .get(&build.id)
          .await
          .unwrap_or_default()
          .get()?,
      ) {
        (_, action_states) if action_states.building => {
          res.building += 1;
        }
        (BuildState::Ok, _) => res.ok += 1,
        (BuildState::Failed, _) => res.failed += 1,
        (BuildState::Unknown, _) => res.unknown += 1,
        // will never come off the cache in the building state, since that comes from action states
        (BuildState::Building, _) => unreachable!(),
      }
    }

    Ok(res)
  }
}

const ONE_DAY_MS: i64 = 86400000;

impl Resolve<ReadArgs> for GetBuildMonthlyStats {
  async fn resolve(
    self,
    _: &ReadArgs,
  ) -> serror::Result<GetBuildMonthlyStatsResponse> {
    let curr_ts = unix_timestamp_ms() as i64;
    let next_day = curr_ts - curr_ts % ONE_DAY_MS + ONE_DAY_MS;

    let close_ts = next_day - self.page as i64 * 30 * ONE_DAY_MS;
    let open_ts = close_ts - 30 * ONE_DAY_MS;

    let mut build_updates = db_client()
      .updates
      .find(doc! {
        "start_ts": {
          "$gte": open_ts,
          "$lt": close_ts
        },
        "operation": Operation::RunBuild.to_string(),
      })
      .await
      .context("failed to get updates cursor")?;

    let mut days = HashMap::<i64, BuildStatsDay>::with_capacity(32);

    let mut curr = open_ts;

    while curr < close_ts {
      let stats = BuildStatsDay {
        ts: curr as f64,
        ..Default::default()
      };
      days.insert(curr, stats);
      curr += ONE_DAY_MS;
    }

    while let Some(update) = build_updates.try_next().await? {
      if let Some(end_ts) = update.end_ts {
        let day = update.start_ts - update.start_ts % ONE_DAY_MS;
        let entry = days.entry(day).or_default();
        entry.count += 1.0;
        entry.time += ms_to_hour(end_ts - update.start_ts);
      }
    }

    Ok(GetBuildMonthlyStatsResponse::new(
      days.into_values().collect(),
    ))
  }
}

const MS_TO_HOUR_DIVISOR: f64 = 1000.0 * 60.0 * 60.0;
fn ms_to_hour(duration: i64) -> f64 {
  duration as f64 / MS_TO_HOUR_DIVISOR
}

impl Resolve<ReadArgs> for ListBuildVersions {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<Vec<BuildVersionResponseItem>> {
    let ListBuildVersions {
      build,
      major,
      minor,
      patch,
      limit,
    } = self;
    let build = resource::get_check_permissions::<Build>(
      &build,
      user,
      PermissionLevel::Read,
    )
    .await?;

    let mut filter = doc! {
      "target": {
        "type": "Build",
        "id": build.id
      },
      "operation": Operation::RunBuild.to_string(),
      "status": UpdateStatus::Complete.to_string(),
      "success": true
    };
    if let Some(major) = major {
      filter.insert("version.major", major);
    }
    if let Some(minor) = minor {
      filter.insert("version.minor", minor);
    }
    if let Some(patch) = patch {
      filter.insert("version.patch", patch);
    }

    let versions = find_collect(
      &db_client().updates,
      filter,
      FindOptions::builder()
        .sort(doc! { "_id": -1 })
        .limit(limit)
        .build(),
    )
    .await
    .context("failed to pull versions from mongo")?
    .into_iter()
    .map(|u| (u.version, u.start_ts))
    .filter(|(v, _)| !v.is_none())
    .map(|(version, ts)| BuildVersionResponseItem { version, ts })
    .collect();
    Ok(versions)
  }
}

impl Resolve<ReadArgs> for ListCommonBuildExtraArgs {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<ListCommonBuildExtraArgsResponse> {
    let all_tags = if self.query.tags.is_empty() {
      vec![]
    } else {
      get_all_tags(None).await?
    };
    let builds = resource::list_full_for_user::<Build>(
      self.query, user, &all_tags,
    )
    .await
    .context("failed to get resources matching query")?;

    // first collect with guaranteed uniqueness
    let mut res = HashSet::<String>::new();

    for build in builds {
      for extra_arg in build.config.extra_args {
        res.insert(extra_arg);
      }
    }

    let mut res = res.into_iter().collect::<Vec<_>>();
    res.sort();
    Ok(res)
  }
}

impl Resolve<ReadArgs> for GetBuildWebhookEnabled {
  async fn resolve(
    self,
    ReadArgs { user }: &ReadArgs,
  ) -> serror::Result<GetBuildWebhookEnabledResponse> {
    let Some(github) = github_client() else {
      return Ok(GetBuildWebhookEnabledResponse {
        managed: false,
        enabled: false,
      });
    };

    let build = resource::get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Read,
    )
    .await?;

    if build.config.git_provider != "github.com"
      || build.config.repo.is_empty()
    {
      return Ok(GetBuildWebhookEnabledResponse {
        managed: false,
        enabled: false,
      });
    }

    let mut split = build.config.repo.split('/');
    let owner = split.next().context("Build repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Ok(GetBuildWebhookEnabledResponse {
        managed: false,
        enabled: false,
      });
    };

    let repo =
      split.next().context("Build repo has no repo after the /")?;

    let github_repos = github.repos();

    let webhooks = github_repos
      .list_all_webhooks(owner, repo)
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
    let url = format!("{host}/listener/github/build/{}", build.id);

    for webhook in webhooks {
      if webhook.active && webhook.config.url == url {
        return Ok(GetBuildWebhookEnabledResponse {
          managed: true,
          enabled: true,
        });
      }
    }

    Ok(GetBuildWebhookEnabledResponse {
      managed: true,
      enabled: false,
    })
  }
}
