use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::entities::{
  DefaultRepoFolder, LatestCommit, update::Log,
};
use periphery_client::api::git::{
  CloneRepo, DeleteRepo, GetLatestCommit,
  PeripheryRepoExecutionResponse, PullOrCloneRepo, PullRepo,
  RenameRepo,
};
use resolver_api::Resolve;
use std::path::PathBuf;
use tokio::fs;

use crate::{
  config::periphery_config, helpers::handle_post_repo_execution,
};

impl Resolve<crate::api::Args> for GetLatestCommit {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Option<LatestCommit>> {
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

impl Resolve<crate::api::Args> for CloneRepo {
  #[instrument(
    "CloneRepo",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<PeripheryRepoExecutionResponse> {
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

    let token = crate::helpers::git_token(git_token, &args)?;
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
  }
}

//

impl Resolve<crate::api::Args> for PullRepo {
  #[instrument(
    "PullRepo",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<PeripheryRepoExecutionResponse> {
    let PullRepo {
      args,
      git_token,
      environment,
      env_file_path,
      on_pull,
      skip_secret_interp,
      replacers,
    } = self;

    let token = crate::helpers::git_token(git_token, &args)?;
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
  }
}

//

impl Resolve<crate::api::Args> for PullOrCloneRepo {
  #[instrument(
    "PullOrCloneRepo",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      args = format!("{:?}", self.args),
      skip_secret_interp = self.skip_secret_interp,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<PeripheryRepoExecutionResponse> {
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

    let token = crate::helpers::git_token(git_token, &args)?;
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
  }
}

//

impl Resolve<crate::api::Args> for RenameRepo {
  #[instrument(
    "RenameRepo",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      current_name = self.curr_name,
      new_name = self.new_name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
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

impl Resolve<crate::api::Args> for DeleteRepo {
  #[instrument(
    "DeleteRepo",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      repo = self.name,
      is_build = self.is_build,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
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
) -> anyhow::Result<PathBuf> {
  match default_folder {
    DefaultRepoFolder::Stacks => Ok(periphery_config().stack_dir()),
    DefaultRepoFolder::Builds => Ok(periphery_config().build_dir()),
    DefaultRepoFolder::Repos => Ok(periphery_config().repo_dir()),
    DefaultRepoFolder::NotApplicable => Err(anyhow!(
      "The clone args should not have a default_folder of NotApplicable using this method."
    )),
  }
}
