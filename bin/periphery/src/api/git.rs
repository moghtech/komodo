use anyhow::{Context, anyhow};
use axum::http::StatusCode;
use formatting::format_serror;
use komodo_client::entities::{
  DefaultRepoFolder, LatestCommit, RepoExecutionArgs, update::Log,
};
use periphery_client::api::git::{
  CloneRepo, DeleteRepo, GetLatestCommit,
  PeripheryRepoExecutionResponse, PullOrCloneRepo, PullRepo,
  RenameRepo,
};
use resolver_api::Resolve;
use serror::AddStatusCodeError;
use std::path::PathBuf;
use tokio::fs;

use crate::{
  config::periphery_config, git::handle_post_repo_execution,
};

impl Resolve<super::Args> for GetLatestCommit {
  #[instrument(name = "GetLatestCommit", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Option<LatestCommit>> {
    let repo_path = match self.path {
      Some(p) => PathBuf::from(p),
      None => periphery_config().repo_dir().join(self.name),
    };
    // Make sure its a repo, or return null to avoid log spam
    if !repo_path.is_dir() || !repo_path.join(".git").is_dir() {
      return Ok(None);
    }
    Ok(Some(git::get_commit_hash_info(&repo_path).await?))
  }
}

impl Resolve<super::Args> for CloneRepo {
  #[instrument(
    name = "CloneRepo",
    skip_all,
    fields(
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<PeripheryRepoExecutionResponse> {
    let CloneRepo {
      args,
      git_token,
      environment,
      env_file_path,
      on_clone,
      on_pull,
      skip_secret_interp,
      replacers,
    } = self;
    let RepoExecutionArgs {
      provider, account, ..
    } = &args;
    let token = match (account, git_token) {
      (None, _) => None,
      (Some(_), Some(token)) => Some(token),
      (Some(account),  None) => Some(
        crate::helpers::git_token(provider, account).map(ToString::to_string)
          .with_context(
            || format!("Failed to get git token from periphery config | Provider: {provider} | Account: {account}")
          )?,
      ),
    };

    let root_repo_dir = default_folder(args.default_folder)?;

    let res = git::clone(args, &root_repo_dir, token).await?;

    handle_post_repo_execution(
      res,
      environment,
      &env_file_path,
      on_clone,
      on_pull,
      skip_secret_interp,
      replacers,
    )
    .await
    .map_err(Into::into)
  }
}

//

impl Resolve<super::Args> for PullRepo {
  #[instrument(
    name = "PullRepo",
    skip_all,
    fields(
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<PeripheryRepoExecutionResponse> {
    let PullRepo {
      args,
      git_token,
      environment,
      env_file_path,
      on_pull,
      skip_secret_interp,
      replacers,
    } = self;
    let RepoExecutionArgs {
      provider, account, ..
    } = &args;
    let token = match (account, git_token) {
      (None, _) => None,
      (Some(_), Some(token)) => Some(token),
      (Some(account),  None) => Some(
        crate::helpers::git_token(provider, account).map(ToString::to_string)
          .with_context(
            || format!("Failed to get git token from periphery config | provider: {provider} | account: {account}")
          )?,
      ),
    };

    let parent_dir = default_folder(args.default_folder)?;

    let res = git::pull(args, &parent_dir, token).await?;

    handle_post_repo_execution(
      res,
      environment,
      &env_file_path,
      None,
      on_pull,
      skip_secret_interp,
      replacers,
    )
    .await
    .map_err(Into::into)
  }
}

//

impl Resolve<super::Args> for PullOrCloneRepo {
  #[instrument(
    name = "PullOrCloneRepo",
    skip_all,
    fields(
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<PeripheryRepoExecutionResponse> {
    let PullOrCloneRepo {
      args,
      git_token,
      environment,
      env_file_path,
      on_clone,
      on_pull,
      skip_secret_interp,
      replacers,
    } = self;
    let RepoExecutionArgs {
      provider, account, ..
    } = &args;
    let token = match (account, git_token) {
      (None, _) => None,
      (Some(_), Some(token)) => Some(token),
      (Some(account),  None) => Some(
        crate::helpers::git_token(provider, account).map(ToString::to_string)
          .with_context(
            || format!("Failed to get git token from periphery config | provider: {provider} | account: {account}")
          )?,
      ),
    };
    let parent_dir = default_folder(args.default_folder)?;

    let (res, cloned) =
      git::pull_or_clone(args, &parent_dir, token).await?;

    handle_post_repo_execution(
      res,
      environment,
      &env_file_path,
      cloned.then_some(on_clone).flatten(),
      on_pull,
      skip_secret_interp,
      replacers,
    )
    .await
    .map_err(Into::into)
  }
}

//

impl Resolve<super::Args> for RenameRepo {
  #[instrument(name = "RenameRepo")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let RenameRepo {
      curr_name,
      new_name,
    } = self;
    let repo_dir = periphery_config().repo_dir();
    let renamed =
      fs::rename(repo_dir.join(&curr_name), repo_dir.join(&new_name))
        .await;
    let msg = match renamed {
      Ok(_) => String::from("Renamed Repo directory on Server"),
      Err(_) => format!("No Repo cloned at {curr_name} to rename"),
    };
    Ok(Log::simple("Rename Repo on Server", msg))
  }
}

//

impl Resolve<super::Args> for DeleteRepo {
  #[instrument(name = "DeleteRepo")]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let DeleteRepo { name, is_build } = self;
    // If using custom clone path, it will be passed by core instead of name.
    // So the join will resolve to just the absolute path.
    let root = if is_build {
      periphery_config().build_dir()
    } else {
      periphery_config().repo_dir()
    };
    let full_path = root.join(&name);
    let deleted =
      fs::remove_dir_all(&full_path).await.with_context(|| {
        format!("Failed to delete repo at {full_path:?}")
      });
    let log = match deleted {
      Ok(_) => {
        Log::simple("Delete repo", format!("Deleted Repo {name}"))
      }
      Err(e) => Log::error("Delete repo", format_serror(&e.into())),
    };
    Ok(log)
  }
}

//

fn default_folder(
  default_folder: DefaultRepoFolder,
) -> serror::Result<PathBuf> {
  match default_folder {
    DefaultRepoFolder::Stacks => Ok(periphery_config().stack_dir()),
    DefaultRepoFolder::Builds => Ok(periphery_config().build_dir()),
    DefaultRepoFolder::Repos => Ok(periphery_config().repo_dir()),
    DefaultRepoFolder::NotApplicable => {
      Err(
        anyhow!("The clone args should not have a default_folder of NotApplicable using this method.")
          .status_code(StatusCode::BAD_REQUEST)
      )
    }
  }
}
