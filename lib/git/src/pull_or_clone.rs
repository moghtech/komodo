use std::{collections::HashMap, path::Path};

use komodo_client::entities::{CloneArgs, EnvironmentVar};

use crate::GitRes;

/// This is a mix of clone / pull.
///   - If the folder doesn't exist, it will clone the repo.
///   - If it does, it will ensure the remote is correct,
///     ensure the correct branch is (force) checked out,
///     force pull the repo, and switch to specified hash if provided.
#[tracing::instrument(
  level = "debug",
  skip(
    clone_args,
    access_token,
    environment,
    secrets,
    core_replacers
  )
)]
pub async fn pull_or_clone<T>(
  clone_args: T,
  root_repo_dir: &Path,
  access_token: Option<String>,
  environment: &[EnvironmentVar],
  env_file_path: &str,
  // if skip_secret_interp is none, make sure to pass None here
  secrets: Option<&HashMap<String, String>>,
  core_replacers: &[(String, String)],
) -> anyhow::Result<GitRes>
where
  T: Into<CloneArgs> + std::fmt::Debug,
{
  let args: CloneArgs = clone_args.into();
  let folder_path = args.path(root_repo_dir);

  if folder_path.exists() {
    crate::pull(
      args,
      root_repo_dir,
      access_token,
      environment,
      env_file_path,
      secrets,
      core_replacers,
    )
    .await
  } else {
    crate::clone(
      args,
      root_repo_dir,
      access_token,
      environment,
      env_file_path,
      secrets,
      core_replacers,
    )
    .await
  }
}
