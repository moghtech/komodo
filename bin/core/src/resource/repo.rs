use std::time::Duration;

use anyhow::Context;
use database::mungos::{
  find::find_collect,
  mongodb::{Collection, bson::doc, options::FindOneOptions},
};
use formatting::format_serror;
use komodo_client::entities::{
  Operation, ResourceTarget, ResourceTargetVariant,
  builder::Builder,
  permission::PermissionLevel,
  repo::{
    PartialRepoConfig, Repo, RepoConfig, RepoConfigDiff, RepoInfo,
    RepoListItem, RepoListItemInfo, RepoQuerySpecifics, RepoState,
  },
  resource::Resource,
  server::Server,
  to_path_compatible_name,
  update::Update,
  user::User,
};
use periphery_client::api::git::DeleteRepo;

use crate::{
  config::core_config,
  helpers::{periphery_client, repo_link},
  state::{
    action_states, db_client, repo_state_cache, repo_status_cache,
  },
};

use super::get_check_permissions;

impl super::KomodoResource for Repo {
  type Config = RepoConfig;
  type PartialConfig = PartialRepoConfig;
  type ConfigDiff = RepoConfigDiff;
  type Info = RepoInfo;
  type ListItem = RepoListItem;
  type QuerySpecifics = RepoQuerySpecifics;

  fn resource_type() -> ResourceTargetVariant {
    ResourceTargetVariant::Repo
  }

  fn resource_target(id: impl Into<String>) -> ResourceTarget {
    ResourceTarget::Repo(id.into())
  }

  fn validated_name(name: &str) -> String {
    to_path_compatible_name(name)
  }

  fn coll() -> &'static Collection<Resource<Self::Config, Self::Info>>
  {
    &db_client().repos
  }

  async fn to_list_item(
    repo: Resource<Self::Config, Self::Info>,
  ) -> Self::ListItem {
    let state = get_repo_state(&repo.id).await;
    let status =
      repo_status_cache().get(&repo.id).await.unwrap_or_default();
    RepoListItem {
      name: repo.name,
      id: repo.id,
      template: repo.template,
      tags: repo.tags,
      resource_type: ResourceTargetVariant::Repo,
      info: RepoListItemInfo {
        server_id: repo.config.server_id,
        builder_id: repo.config.builder_id,
        last_pulled_at: repo.info.last_pulled_at,
        last_built_at: repo.info.last_built_at,
        repo_link: repo_link(
          &repo.config.git_provider,
          &repo.config.repo,
          &repo.config.branch,
          repo.config.git_https,
        ),
        git_provider: repo.config.git_provider,
        repo: repo.config.repo,
        branch: repo.config.branch,
        state,
        cloned_hash: status.latest_hash.clone(),
        cloned_message: status.latest_message.clone(),
        latest_hash: repo.info.latest_hash,
        built_hash: repo.info.built_hash,
      },
    }
  }

  async fn busy(id: &String) -> anyhow::Result<bool> {
    action_states()
      .repo
      .get(id)
      .await
      .unwrap_or_default()
      .busy()
  }

  // CREATE

  fn create_operation() -> Operation {
    Operation::CreateRepo
  }

  fn user_can_create(user: &User) -> bool {
    user.admin || !core_config().disable_non_admin_create
  }

  async fn validate_create_config(
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_create(
    _created: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    refresh_repo_state_cache().await;
    Ok(())
  }

  // UPDATE

  fn update_operation() -> Operation {
    Operation::UpdateRepo
  }

  async fn validate_update_config(
    _id: &str,
    config: &mut Self::PartialConfig,
    user: &User,
  ) -> anyhow::Result<()> {
    validate_config(config, user).await
  }

  async fn post_update(
    _updated: &Self,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    refresh_repo_state_cache().await;
    Ok(())
  }

  // RENAME

  fn rename_operation() -> Operation {
    Operation::RenameRepo
  }

  // DELETE

  fn delete_operation() -> Operation {
    Operation::DeleteRepo
  }

  async fn pre_delete(
    repo: &Resource<Self::Config, Self::Info>,
    update: &mut Update,
  ) -> anyhow::Result<()> {
    if repo.config.server_id.is_empty() {
      return Ok(());
    }

    let server = super::get::<Server>(&repo.config.server_id).await?;
    let periphery = periphery_client(&server).await?;

    match periphery
      .request(DeleteRepo {
        name: if repo.config.path.is_empty() {
          to_path_compatible_name(&repo.name)
        } else {
          repo.config.path.clone()
        },
        is_build: false,
      })
      .await
    {
      Ok(log) => update.logs.push(log),
      Err(e) => update.push_error_log(
        "Delete Repo on Periphery",
        format_serror(
          &e.context("Failed to delete repo files").into(),
        ),
      ),
    }

    Ok(())
  }

  async fn post_delete(
    repo: &Resource<Self::Config, Self::Info>,
    _update: &mut Update,
  ) -> anyhow::Result<()> {
    repo_state_cache().remove(&repo.id).await;

    Ok(())
  }
}

pub fn spawn_repo_state_refresh_loop() {
  tokio::spawn(async move {
    loop {
      refresh_repo_state_cache().await;
      tokio::time::sleep(Duration::from_secs(60)).await;
    }
  });
}

pub async fn refresh_repo_state_cache() {
  let _ = async {
    let repos = find_collect(&db_client().repos, None, None)
      .await
      .context("failed to get repos from db")?;
    let cache = repo_state_cache();
    for repo in repos {
      let state = get_repo_state_from_db(&repo.id).await;
      cache.insert(repo.id, state).await;
    }
    anyhow::Ok(())
  }
  .await
  .inspect_err(|e| {
    warn!("failed to refresh repo state cache | {e:#}")
  });
}

#[instrument("ValidateRepoConfig", skip_all)]
async fn validate_config(
  config: &mut PartialRepoConfig,
  user: &User,
) -> anyhow::Result<()> {
  if let Some(server_id) = &config.server_id
    && !server_id.is_empty()
  {
    let server = get_check_permissions::<Server>(
      server_id,
      user,
      PermissionLevel::Read.attach(),
    )
    .await
    .context("Cannot attach Repo to this Server")?;
    config.server_id = Some(server.id);
  }
  if let Some(builder_id) = &config.builder_id
    && !builder_id.is_empty()
  {
    let builder = super::get_check_permissions::<Builder>(
      builder_id,
      user,
      PermissionLevel::Read.attach(),
    )
    .await
    .context("Cannot attach Repo to this Builder")?;
    config.builder_id = Some(builder.id);
  }
  Ok(())
}

async fn get_repo_state(id: &String) -> RepoState {
  if let Some(state) = action_states()
    .repo
    .get(id)
    .await
    .and_then(|s| {
      s.get()
        .map(|s| {
          if s.cloning {
            Some(RepoState::Cloning)
          } else if s.pulling {
            Some(RepoState::Pulling)
          } else if s.building {
            Some(RepoState::Building)
          } else {
            None
          }
        })
        .ok()
    })
    .flatten()
  {
    return state;
  }
  repo_state_cache().get(id).await.unwrap_or_default()
}

async fn get_repo_state_from_db(id: &str) -> RepoState {
  async {
    let state = db_client()
      .updates
      .find_one(doc! {
        "target.type": "Repo",
        "target.id": id,
        "$or": [
          { "operation": "CloneRepo" },
          { "operation": "PullRepo" },
          { "operation": "BuildRepo" },
        ],
      })
      .with_options(
        FindOneOptions::builder()
          .sort(doc! { "start_ts": -1 })
          .build(),
      )
      .await?
      .map(|u| {
        if u.success {
          RepoState::Ok
        } else {
          RepoState::Failed
        }
      })
      .unwrap_or(RepoState::Ok);
    anyhow::Ok(state)
  }
  .await
  .inspect_err(|e| warn!("failed to get repo state for {id} | {e:#}"))
  .unwrap_or(RepoState::Unknown)
}
