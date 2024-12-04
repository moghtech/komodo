use anyhow::{anyhow, Context};
use git::GitRes;
use komodo_client::{
  api::write::*,
  entities::{
    build::{Build, BuildInfo, PartialBuildConfig},
    config::core::CoreConfig,
    permission::PermissionLevel,
    update::Update,
    CloneArgs, NoData,
  },
};
use mongo_indexed::doc;
use mungos::mongodb::bson::to_document;
use octorust::types::{
  ReposCreateWebhookRequest, ReposCreateWebhookRequestConfig,
};
use resolver_api::Resolve;

use crate::{
  config::core_config,
  helpers::git_token,
  resource,
  state::{db_client, github_client},
};

use super::WriteArgs;

impl Resolve<WriteArgs> for CreateBuild {
  #[instrument(name = "CreateBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    Ok(
      resource::create::<Build>(&self.name, self.config, user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for CopyBuild {
  #[instrument(name = "CopyBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    let Build { mut config, .. } =
      resource::get_check_permissions::<Build>(
        &self.id,
        user,
        PermissionLevel::Write,
      )
      .await?;
    // reset version to 0.0.0
    config.version = Default::default();
    Ok(
      resource::create::<Build>(&self.name, config.into(), user)
        .await?,
    )
  }
}

impl Resolve<WriteArgs> for DeleteBuild {
  #[instrument(name = "DeleteBuild", skip(args))]
  async fn resolve(self, args: &WriteArgs) -> serror::Result<Build> {
    Ok(resource::delete::<Build>(&self.id, args).await?)
  }
}

impl Resolve<WriteArgs> for UpdateBuild {
  #[instrument(name = "UpdateBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Build> {
    Ok(resource::update::<Build>(&self.id, self.config, user).await?)
  }
}

impl Resolve<WriteArgs> for RenameBuild {
  #[instrument(name = "RenameBuild", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<Update> {
    Ok(resource::rename::<Build>(&self.id, &self.name, user).await?)
  }
}

impl Resolve<WriteArgs> for RefreshBuildCache {
  #[instrument(
    name = "RefreshBuildCache",
    level = "debug",
    skip(user)
  )]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<NoData> {
    // Even though this is a write request, this doesn't change any config. Anyone that can execute the
    // build should be able to do this.
    let build = resource::get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Execute,
    )
    .await?;

    if build.config.repo.is_empty()
      || build.config.git_provider.is_empty()
    {
      // Nothing to do here
      return Ok(NoData {});
    }

    let config = core_config();

    let mut clone_args: CloneArgs = (&build).into();
    let repo_path =
      clone_args.unique_path(&core_config().repo_directory)?;
    clone_args.destination = Some(repo_path.display().to_string());
    // Don't want to run these on core.
    clone_args.on_clone = None;
    clone_args.on_pull = None;

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

    let GitRes {
      hash: latest_hash,
      message: latest_message,
      ..
    } = git::pull_or_clone(
      clone_args,
      &config.repo_directory,
      access_token,
      &[],
      "",
      None,
      &[],
    )
    .await
    .context("failed to clone build repo")?;

    let info = BuildInfo {
      last_built_at: build.info.last_built_at,
      built_hash: build.info.built_hash,
      built_message: build.info.built_message,
      latest_hash,
      latest_message,
    };

    let info = to_document(&info)
      .context("failed to serialize build info to bson")?;

    db_client()
      .builds
      .update_one(
        doc! { "name": &build.name },
        doc! { "$set": { "info": info } },
      )
      .await
      .context("failed to update build info on db")?;

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for CreateBuildWebhook {
  #[instrument(name = "CreateBuildWebhook", skip(args))]
  async fn resolve(
    self,
    args: &WriteArgs,
  ) -> serror::Result<CreateBuildWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let WriteArgs { user } = args;

    let build = resource::get_check_permissions::<Build>(
      &self.build,
      user,
      PermissionLevel::Write,
    )
    .await?;

    if build.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't create webhook").into(),
      );
    }

    let mut split = build.config.repo.split('/');
    let owner = split.next().context("Build repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
    };

    let repo =
      split.next().context("Build repo has no repo after the /")?;

    let github_repos = github.repos();

    // First make sure the webhook isn't already created (inactive ones are ignored)
    let webhooks = github_repos
      .list_all_webhooks(owner, repo)
      .await
      .context("failed to list all webhooks on repo")?
      .body;

    let CoreConfig {
      host,
      webhook_base_url,
      webhook_secret,
      ..
    } = core_config();

    let webhook_secret = if build.config.webhook_secret.is_empty() {
      webhook_secret
    } else {
      &build.config.webhook_secret
    };

    let host = if webhook_base_url.is_empty() {
      host
    } else {
      webhook_base_url
    };
    let url = format!("{host}/listener/github/build/{}", build.id);

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
      .create_webhook(owner, repo, &request)
      .await
      .context("failed to create webhook")?;

    if !build.config.webhook_enabled {
      UpdateBuild {
        id: build.id,
        config: PartialBuildConfig {
          webhook_enabled: Some(true),
          ..Default::default()
        },
      }
      .resolve(args)
      .await
      .map_err(|e| e.error)
      .context("failed to update build to enable webhook")?;
    }

    Ok(NoData {})
  }
}

impl Resolve<WriteArgs> for DeleteBuildWebhook {
  #[instrument(name = "DeleteBuildWebhook", skip(user))]
  async fn resolve(
    self,
    WriteArgs { user }: &WriteArgs,
  ) -> serror::Result<DeleteBuildWebhookResponse> {
    let Some(github) = github_client() else {
      return Err(
        anyhow!(
          "github_webhook_app is not configured in core config toml"
        )
        .into(),
      );
    };

    let build = resource::get_check_permissions::<Build>(
      &self.build,
      &user,
      PermissionLevel::Write,
    )
    .await?;

    if build.config.git_provider != "github.com" {
      return Err(
        anyhow!("Can only manage github.com repo webhooks").into(),
      );
    }

    if build.config.repo.is_empty() {
      return Err(
        anyhow!("No repo configured, can't delete webhook").into(),
      );
    }

    let mut split = build.config.repo.split('/');
    let owner = split.next().context("Build repo has no owner")?;

    let Some(github) = github.get(owner) else {
      return Err(
        anyhow!("Cannot manage repo webhooks under owner {owner}")
          .into(),
      );
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
        github_repos
          .delete_webhook(owner, repo, webhook.id)
          .await
          .context("failed to delete webhook")?;
        return Ok(NoData {});
      }
    }

    // No webhook to delete, all good
    Ok(NoData {})
  }
}
