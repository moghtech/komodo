use std::path::Path;

use komodo_client::entities::{
  RepoExecutionArgs, RepoExecutionResponse,
};
use tokio_util::sync::CancellationToken;

/// This is a mix of clone / pull.
///   - If the folder doesn't exist, it will clone the repo.
///     - Second variable in tuple will be `true`
///   - If it does, it will ensure the remote is correct,
///     ensure the correct branch is (force) checked out,
///     force pull the repo, and switch to specified hash if provided.
pub async fn pull_or_clone<T>(
  clone_args: T,
  root_repo_dir: &Path,
  access_token: Option<String>,
) -> anyhow::Result<(RepoExecutionResponse, bool)>
where
  T: Into<RepoExecutionArgs> + std::fmt::Debug,
{
  pull_or_clone_with_cancel(
    clone_args,
    root_repo_dir,
    access_token,
    None,
  )
  .await
}

pub async fn pull_or_clone_with_cancel<T>(
  clone_args: T,
  root_repo_dir: &Path,
  access_token: Option<String>,
  cancel: Option<CancellationToken>,
) -> anyhow::Result<(RepoExecutionResponse, bool)>
where
  T: Into<RepoExecutionArgs> + std::fmt::Debug,
{
  let args: RepoExecutionArgs = clone_args.into();
  let folder_path = args.path(root_repo_dir);

  if folder_path.exists() {
    crate::pull_with_cancel(args, root_repo_dir, access_token, cancel)
      .await
      .map(|r| (r, false))
  } else {
    crate::clone_with_cancel(
      args,
      root_repo_dir,
      access_token,
      cancel,
    )
    .await
    .map(|r| (r, true))
  }
}
