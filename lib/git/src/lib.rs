use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use formatting::{bold, muted};
use komodo_client::entities::{
  LatestCommit, komodo_timestamp, update::Log,
};
use run_command::async_run_command;
use tracing::instrument;

pub mod environment;

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

#[derive(Debug, Default, Clone)]
pub struct GitRes {
  pub logs: Vec<Log>,
  pub path: PathBuf,
  pub hash: Option<String>,
  pub message: Option<String>,
  pub env_file_path: Option<PathBuf>,
}

#[instrument(level = "debug")]
pub async fn get_commit_hash_info(
  repo_dir: &Path,
) -> anyhow::Result<LatestCommit> {
  let command = format!(
    "cd {} && git rev-parse --short HEAD && git rev-parse HEAD && git log -1 --pretty=%B",
    repo_dir.display()
  );
  let output = async_run_command(&command).await;
  let mut split = output.stdout.split('\n');
  let (hash, _, message) = (
    split
      .next()
      .context("Failed to get short commit hash")?
      .to_string(),
    split.next().context("failed to get long commit hash")?,
    split
      .next()
      .context("Failed to get commit message")?
      .to_string(),
  );
  Ok(LatestCommit { hash, message })
}
/// returns (Log, commit hash, commit message)
#[instrument(level = "debug")]
pub async fn get_commit_hash_log(
  repo_dir: &Path,
) -> anyhow::Result<(Log, String, String)> {
  let start_ts = komodo_timestamp();
  let command = format!(
    "cd {} && git rev-parse --short HEAD && git rev-parse HEAD && git log -1 --pretty=%B",
    repo_dir.display()
  );
  let output = async_run_command(&command).await;
  let mut split = output.stdout.split('\n');
  let (short_hash, _, msg) = (
    split
      .next()
      .context("Failed to get short commit hash")?
      .to_string(),
    split.next().context("Failed to get long commit hash")?,
    split
      .next()
      .context("Failed to get commit message")?
      .to_string(),
  );
  let log = Log {
    stage: "Latest Commit".into(),
    command,
    stdout: format!(
      "{} {}\n{} {}",
      muted("hash:"),
      bold(&short_hash),
      muted("message:"),
      bold(&msg),
    ),
    stderr: String::new(),
    success: true,
    start_ts,
    end_ts: komodo_timestamp(),
  };
  Ok((log, short_hash, msg))
}

/// Gets the remote url, with `.git` stripped from the end.
pub async fn get_remote_url(path: &Path) -> anyhow::Result<String> {
  let command =
    format!("cd {} && git remote show origin", path.display());
  let output = async_run_command(&command).await;
  if output.success() {
    Ok(
      output
        .stdout
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
