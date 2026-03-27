use std::path::Path;

use anyhow::anyhow;
use command::run_standard_command;
use formatting::{bold, muted};
use komodo_client::entities::{
  LatestCommit, komodo_timestamp, update::Log,
};

mod clone;
mod commit;
mod init;
mod pull;
mod pull_or_clone;

pub use crate::{
  clone::clone,
  commit::{commit_all, commit_file, write_commit_file},
  init::init_folder_as_repo,
  pull::pull,
  pull_or_clone::pull_or_clone,
};

pub async fn get_commit_hash_info(
  repo_dir: &Path,
) -> anyhow::Result<LatestCommit> {
  let hash =
    run_standard_command("git rev-parse --short HEAD", repo_dir)
      .await;
  let hash = if hash.status.success() {
    hash.stdout.trim().to_string()
  } else {
    return Err(anyhow!(
      "Failed to get short hash | {}",
      hash.stderr
    ));
  };
  let message =
    run_standard_command("git log -1 --pretty=%B", repo_dir).await;
  let message = if message.status.success() {
    message.stdout.trim().to_string()
  } else {
    return Err(anyhow!(
      "Failed to get commit message | {}",
      message.stderr
    ));
  };
  Ok(LatestCommit { hash, message })
}
/// returns (Log, commit hash, commit message)
pub async fn get_commit_hash_log(
  repo_dir: &Path,
) -> anyhow::Result<(Log, String, String)> {
  let start_ts = komodo_timestamp();
  let LatestCommit { hash, message } =
    get_commit_hash_info(repo_dir).await?;
  let log = Log {
    stage: "Latest Commit".into(),
    command: String::from(
      "git rev-parse --short HEAD && git log -1 --pretty=%B",
    ),
    stdout: format!(
      "{} {}\n{} {}",
      muted("hash:"),
      bold(&hash),
      muted("message:"),
      bold(&message),
    ),
    stderr: String::new(),
    success: true,
    start_ts,
    end_ts: komodo_timestamp(),
  };
  Ok((log, hash, message))
}

/// Gets the remote url, with `.git` stripped from the end.
pub async fn get_remote_url(path: &Path) -> anyhow::Result<String> {
  let output =
    run_standard_command("git remote show origin", path).await;
  if output.success() {
    Ok(
      output
        .stdout
        .trim()
        .strip_suffix(".git")
        .map(str::to_string)
        .unwrap_or(output.stdout),
    )
  } else {
    Err(anyhow!(
      "Failed to get remote url | stdout: {} | stderr: {}",
      output.stdout,
      output.stderr
    ))
  }
}
