use std::path::{Path, PathBuf};

use anyhow::Context;
use command::run_komodo_command;
use formatting::format_serror;
use komodo_client::entities::{all_logs_success, update::Log};
use run_command::async_run_command;
use tokio::fs;

use crate::{GitRes, get_commit_hash_log};

/// Write file, add, commit, force push.
/// Repo must be cloned.
pub async fn write_commit_file(
  commit_msg: &str,
  repo_dir: &Path,
  // relative to repo root
  file: &Path,
  contents: &str,
  branch: &str,
) -> anyhow::Result<GitRes> {
  // Clean up the path by stripping any redundant `/./`
  let path = repo_dir.join(file).components().collect::<PathBuf>();

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).await.with_context(|| {
      format!("Failed to initialize file parent directory {parent:?}")
    })?;
  }

  fs::write(&path, contents).await.with_context(|| {
    format!("Failed to write contents to {path:?}")
  })?;

  let mut res = GitRes::default();
  res.logs.push(Log::simple(
    "Write file",
    format!("File contents written to {path:?}"),
  ));

  commit_file_inner(commit_msg, &mut res, repo_dir, file, branch)
    .await;

  Ok(res)
}

/// Add file, commit, force push.
/// Repo must be cloned.
pub async fn commit_file(
  commit_msg: &str,
  repo_dir: &Path,
  // relative to repo root
  file: &Path,
  branch: &str,
) -> GitRes {
  let mut res = GitRes::default();
  commit_file_inner(commit_msg, &mut res, repo_dir, file, branch)
    .await;
  res
}

pub async fn commit_file_inner(
  commit_msg: &str,
  res: &mut GitRes,
  repo_dir: &Path,
  // relative to repo root
  file: &Path,
  branch: &str,
) {
  ensure_global_git_config_set().await;

  let add_log = run_komodo_command(
    "Add Files",
    repo_dir,
    format!("git add {}", file.display()),
  )
  .await;
  res.logs.push(add_log);
  if !all_logs_success(&res.logs) {
    return;
  }

  let commit_log = run_komodo_command(
    "Commit",
    repo_dir,
    format!(
      "git commit -m \"[Komodo] {commit_msg}: update {file:?}\"",
    ),
  )
  .await;

  if !commit_log.success {
    // The user may have nothing to commit, but still should continue push the changes
    if !commit_log.stdout.contains("nothing to commit") {
      res.logs.push(commit_log);
      return;
    }
  } else {
    res.logs.push(commit_log);
  }

  match get_commit_hash_log(repo_dir).await {
    Ok((log, hash, message)) => {
      res.logs.push(log);
      res.hash = Some(hash);
      res.message = Some(message);
    }
    Err(e) => {
      res.logs.push(Log::error(
        "Get commit hash",
        format_serror(&e.into()),
      ));
      return;
    }
  };

  let push_log = run_komodo_command(
    "Push",
    repo_dir,
    format!("git push --set-upstream origin {branch}"),
  )
  .await;
  res.logs.push(push_log);
}

/// Add, commit, and force push.
/// Repo must be cloned.
pub async fn commit_all(
  repo_dir: &Path,
  message: &str,
  branch: &str,
) -> GitRes {
  ensure_global_git_config_set().await;

  let mut res = GitRes::default();

  let add_log =
    run_komodo_command("Add Files", repo_dir, "git add -A").await;
  res.logs.push(add_log);
  if !all_logs_success(&res.logs) {
    return res;
  }

  let commit_log = run_komodo_command(
    "Commit",
    repo_dir,
    format!("git commit -m \"[Komodo] {message}\""),
  )
  .await;
  res.logs.push(commit_log);
  if !all_logs_success(&res.logs) {
    return res;
  }

  match get_commit_hash_log(repo_dir).await {
    Ok((log, hash, message)) => {
      res.logs.push(log);
      res.hash = Some(hash);
      res.message = Some(message);
    }
    Err(e) => {
      res.logs.push(Log::error(
        "Get commit hash",
        format_serror(&e.into()),
      ));
      return res;
    }
  };

  let push_log = run_komodo_command(
    "Push",
    repo_dir,
    format!("git push --set-upstream origin {branch}"),
  )
  .await;
  res.logs.push(push_log);

  res
}

async fn ensure_global_git_config_set() {
  let res =
    async_run_command("git config --global --get user.email").await;
  if !res.success() {
    let _ = async_run_command(
      "git config --global user.email komodo@komo.do",
    )
    .await;
  }
  let res =
    async_run_command("git config --global --get user.name").await;
  if !res.success() {
    let _ =
      async_run_command("git config --global user.name komodo").await;
  }
}
