use anyhow::{Context, anyhow};
use database::mongo_indexed::doc;
use database::mungos::{
  by_id::update_one_by_id, mongodb::bson::to_document,
};
use formatting::format_serror;
use komodo_client::{
  api::write::*,
  entities::{
    NoData, Operation, RepoExecutionArgs,
    config::core::CoreConfig,
    komodo_timestamp,
    permission::PermissionLevel,
    repo::{PartialRepoConfig, Repo, RepoInfo},
    server::Server,
    to_path_compatible_name,
    update::{Log, Update},
  },
};
use octorust::types::{
  ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
};
use periphery_client::api;
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::{
    git_token, periphery_client,
    update::{add_update, make_update},
  },
  permission::get_check_permissions,
  resource,
  state::{action_states, db_client, github_client},
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateRepo {
  #[instrument(name = "CreateRepo", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Repo> {
    resource::create::<Repo>(&self.name, self.config, user).await
  }
}

impl Resolve<WriteArgs> for CopyRepo {
  #[instrument(name = "CopyRepo", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Repo> {
    let Repo { config, .. } = get_check_permissions::<Repo>(
      &self.id,
      user,
      PermissionLevel::Read.into(),
    )
    .await?;
    resource::create::<Repo>(&self.name, config.into(), user).await
  }
}

impl Resolve<WriteArgs> for DeleteRepo {
  #[instrument(name = "DeleteRepo", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Repo> {
    Ok(resource::delete::<Repo>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateRepo {
  #[instrument(name = "UpdateRepo", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Repo> {
    Ok(resource::update::<Repo>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameRepo {
  #[instrument(name = "RenameRepo", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    let repo = get_check_permissions::<Repo>(
      &self.id,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if repo.config.server_id.is_empty()
      || !repo.config.path.is_empty()
    {
      return Ok(
        resource::rename::<Repo>(&repo.id, &self.name, user).await?,
      );
    }

    // get the action state for the repo (or insert default).
    let action_state =
      action_states().repo.get_or_insert_default(&repo.id).await;

    // Will check to ensure repo not already busy before updating, and return Err if so.
    // The returned guard will set the action state back to default when dropped.
    let _action_guard =
      action_state.update(|state| state.renaming = true)?;

    let name = to_path_compatible_name(&self.name);

    let mut update = make_update(&repo, Operation::RenameRepo, user);

    update_one_by_id(
      &db_client().repos,
      &repo.id,
      database::mungos::update::Update::Set(
        doc! { "name": &name, "updated_at": komodo_timestamp() },
      ),
      None,
    )
    .await
    .context("Failed to update Repo name on db")?;

    let server =
      resource::get::<Server>(&repo.config.server_id).await?;

    let log = match periphery_client(&server)?
      .request(api::git::RenameRepo {
        curr_name: to_path_compatible_name(&repo.name),
        new_name: name.clone(),
      })
      .await
      .context("Failed to rename Repo directory on Server")
    {
      Ok(log) => log,
      Err(e) => Log::error(
        "Rename Repo directory failure",
        format_serror(&e.into()),
      ),
    };

    update.logs.push(log);

    update.push_simple_log(
      "Rename Repo",
      format!("Renamed Repo from {} to {}", repo.name, name),
    );
    update.finalize();
    update.id = add_update(update.clone()).await?;

    Ok(update)
  }
}

impl Resolve<WriteArgs> for RefreshRepoCache {
  #[instrument(
    name = "RefreshRepoCache",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<NoData> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // repo should be able to do this.
    let repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Execute.into(),
    )
    .await?;

    if repo.config.git_provider.is_empty()
      || repo.config.repo.is_empty()
    {
      // Nothing to do
      return Ok(NoData {});
    }

    let mut clone_args: RepoExecutionArgs = (&repo).into();
    let repo_path =
      clone_args.unique_path(&core_config().repo_directory)?;
    clone_args.destination = Some(repo_path.display().to_string());

    let access_token = if let Some(username) = &clone_args.account {
      git_token(&clone_args.provider, username, |https| {
          clone_args.https = https
        })
        .await
        .with_context(
          || format!("Failed to get git token in call to db. Stopping run. | {} | {username}", clone_args.provider),
        )?
    } else {
      None
    };

    let (res, _) = git::pull_or_clone(
      clone_args,
      &core_config().repo_directory,
      access_token,
    )
    .await
    .with_context(|| {
      format!("Failed to update repo at {repo_path:?}")
    })?;

    let info = RepoInfo {
      last_pulled_at: repo.info.last_pulled_at,
      last_built_at: repo.info.last_built_at,
      built_hash: repo.info.built_hash,
      built_message: repo.info.built_message,
      latest_hash: res.commit_hash,
      latest_message: res.commit_message,
    };

    let info = to_document(&info)
      .context("failed to serialize repo info to bson")?;

    db_client()
      .repos
      .update_one(
        doc! { "name": &repo.name },
        doc! { "$set": { "info": info } },
      )
      .await
      .context("failed to update repo info on db")?;

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for CreateRepoWebhook {
  #[instrument(name = "CreateRepoWebhook", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<CreateRepoWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let repo = get_check_permissions::<Repo>(
      &self.repo,
      &args.user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if repo.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = repo.config.repo.split('/');
    let owner = split.next().context("Repo repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo_name =
      split.next().context("Repo repo has no repo after the /")?;

    let github_repos = github.repos();

    // First make sure the webhook isn't already created (inactive ones are ignored)
    let webhooks = github_repos
      .list_all_webhooks(owner, repo_name)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      webhook_secret,
      ..
    } = core_config();

    let webhook_secret = if repo.config.webhook_secret.is_empty() {
      webhook_secret
    } else {
      &repo.config.webhook_secret
    };

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = match self.action {
      RepoWebhookAction::Clone => {
        format!("{host}/listener/github/repo/{}/clone", repo.id)
      }
      RepoWebhookAction::Pull => {
        format!("{host}/listener/github/repo/{}/pull", repo.id)
      }
      RepoWebhookAction::Build => {
        format!("{host}/listener/github/repo/{}/build", repo.id)
      }
    };

    for webhook in webhooks {
      if webhook.active && webhook.config.url == url {
        return Ok(NoData {});
      }
    }

    // Now good to create the webhook
    let request = ReposCreateWebhookRequest {
      active: Some(true),
      config: Some(ReposCreateWebhookRequestConfig {
        url,
        secret: webhook_secret.to_string(),
        content_type: String::from("json"),
        insecure_ssl: None,
        digest: Default::default(),
        token: Default::default(),
      }),
      events: vec![String::from("push")],
      name: String::from("web"),
    };
    github_repos
      .create_webhook(owner, repo_name, &request)
      .await
      .context("failed to create webhook")?;

    if !repo.config.webhook_enabled {
      UpdateRepo {
        id: repo.id,
        config: PartialRepoConfig {
          webhook_enabled: Some(true),
          ..Default::default()
        },
      }
      .resolve(args)
      .await
      .map_err(|e| e.error)
      .context("failed to update repo to enable webhook")?;
    }

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for DeleteRepoWebhook {
  #[instrument(name = "DeleteRepoWebhook", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteRepoWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let repo = get_check_permissions::<Repo>(
      &self.repo,
      user,
      PermissionLevel::Write.into(),
    )
    .await?;

    if repo.config.git_provider != "github.com" {
      return Err(
        anyhow!("Can only manage github.com repo webhooks").into(),
      );
    }

    if repo.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = repo.config.repo.split('/');
    let owner = split.next().context("Repo repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo_name =
      split.next().context("Repo repo has no repo after the /")?;

    let github_repos = github.repos();

    // First make sure the webhook isn't already created (inactive ones are ignored)
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
    let url = match self.action {
      RepoWebhookAction::Clone => {
        format!("{host}/listener/github/repo/{}/clone", repo.id)
      }
      RepoWebhookAction::Pull => {
        format!("{host}/listener/github/repo/{}/pull", repo.id)
      }
      RepoWebhookAction::Build => {
        format!("{host}/listener/github/repo/{}/build", repo.id)
      }
    };

    for webhook in webhooks {
      if webhook.active && webhook.config.url == url {
        github_repos
          .delete_webhook(owner, repo_name, webhook.id)
          .await
          .context("failed to delete webhook")?;
        return Ok(NoData {});
      }
    }

    // No webhook to delete, all good
    Ok(NoData {})
  }
}
