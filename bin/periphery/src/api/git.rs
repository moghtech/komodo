use std::path::PathBuf;
use anyhow::{anyhow, Context};
use git::GitRes;
use komodo_client::entities::{update::Log, CloneArgs, LatestCommit};
use periphery_client::api::git::{
  CloneRepo, DeleteRepo, GetLatestCommit, PullOrCloneRepo, PullRepo,
  RenameRepo, RepoActionResponse,
};
use resolver_api::Resolve;
use tokio::fs;

use crate::config::periphery_config;

impl Resolve<super::Args> for GetLatestCommit {
  #[instrument(name = "CloneRepo", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<LatestCommit> {
    let repo_path = match self.path {
      Some(p) => PathBuf::from(p),
      None => periphery_config().repo_dir.join(self.name.as_deref().unwrap_or_default())
    };
    if !repo_path.is_dir() {
      return Err(
        anyhow!("Repo path {} is not directory. is it cloned?", repo_path.display().to_string()).into(),
      );
    }
    Ok(git::get_commit_hash_info(&repo_path).await?)
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
  ) -> serror::Result<RepoActionResponse> {
    let CloneRepo {
      args,
      git_token,
      environment,
      env_file_path,
      skip_secret_interp,
      replacers,
    } = self;
    let CloneArgs {
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
    git::clone(
      args,
      &periphery_config().repo_dir,
      token,
      &environment,
      &env_file_path,
      (!skip_secret_interp).then_some(&periphery_config().secrets),
      &replacers,
    )
    .await
    .map(
      |GitRes {
         logs,
         hash,
         message,
         env_file_path,
       }| {
        RepoActionResponse {
          logs,
          commit_hash: hash,
          commit_message: message,
          env_file_path,
        }
      },
    )
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
  ) -> serror::Result<RepoActionResponse> {
    let PullRepo {
      args,
      git_token,
      environment,
      env_file_path,
      skip_secret_interp,
      replacers,
    } = self;
    let CloneArgs {
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
    git::pull(
      args,
      &periphery_config().repo_dir,
      token,
      &environment,
      &env_file_path,
      (!skip_secret_interp).then_some(&periphery_config().secrets),
      &replacers,
    )
    .await
    .map(
      |GitRes {
         logs,
         hash,
         message,
         env_file_path,
       }| {
        RepoActionResponse {
          logs,
          commit_hash: hash,
          commit_message: message,
          env_file_path,
        }
      },
    )
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
  ) -> serror::Result<RepoActionResponse> {
    let PullOrCloneRepo {
      args,
      git_token,
      environment,
      env_file_path,
      skip_secret_interp,
      replacers,
    } = self;
    let CloneArgs {
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
    git::pull_or_clone(
      args,
      &periphery_config().repo_dir,
      token,
      &environment,
      &env_file_path,
      (!skip_secret_interp).then_some(&periphery_config().secrets),
      &replacers,
    )
    .await
    .map(
      |GitRes {
         logs,
         hash,
         message,
         env_file_path,
       }| {
        RepoActionResponse {
          logs,
          commit_hash: hash,
          commit_message: message,
          env_file_path,
        }
      },
    )
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
    let renamed = fs::rename(
      periphery_config().repo_dir.join(&curr_name),
      periphery_config().repo_dir.join(&new_name),
    )
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
    let DeleteRepo { name } = self;
    // If using custom clone path, it will be passed by core instead of name.
    // So the join will resolve to just the absolute path.
    let deleted =
      fs::remove_dir_all(periphery_config().repo_dir.join(&name))
        .await;
    let msg = match deleted {
      Ok(_) => format!("Deleted Repo {name}"),
      Err(_) => format!("No Repo at {name} to delete"),
    };
    Ok(Log::simple("Delete Repo on Host", msg))
  }
}
