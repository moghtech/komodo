use std::path::{Path, PathBuf};

use anyhow::Context;
use command::{run_komodo_standard_command, run_standard_command};
use formatting::format_serror;
use komodo_client::entities::{
  RepoExecutionResponse, all_logs_success, update::Log,
};

use crate::get_commit_hash_log;

/// Write file, add, commit, force push.
/// Repo must be cloned.
pub async fn write_commit_file(
  commit_msg: &str,
  repo_dir: &Path,
  // relative to repo root
  relative_file_path: &Path,
  contents: &str,
  branch: &str,
) -> anyhow::Result<RepoExecutionResponse> {
  let mut res = RepoExecutionResponse {
    path: repo_dir.to_path_buf(),
    logs: Vec::new(),
    commit_hash: None,
    commit_message: None,
  };

  // Clean up the path by stripping any redundant `/./`
  let full_file_path = repo_dir
    .join(relative_file_path)
    .components()
    .collect::<PathBuf>();

  if let Some(parent) = full_file_path.parent() {
    tokio::fs::create_dir_all(parent).await.with_context(|| {
      format!("Failed to initialize file parent directory {parent:?}")
    })?;
  }

  tokio::fs::write(&full_file_path, contents)
    .await
    .with_context(|| {
      format!("Failed to write contents to {full_file_path:?}")
    })?;

  res.logs.push(Log::simple(
    "Write file",
    format!("File contents written to {full_file_path:?}"),
  ));

  commit_file_inner(
    commit_msg,
    &mut res,
    repo_dir,
    relative_file_path,
    branch,
  )
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
) -> RepoExecutionResponse {
  let mut res = RepoExecutionResponse {
    path: repo_dir.to_path_buf(),
    logs: Vec::new(),
    commit_hash: None,
    commit_message: None,
  };

  commit_file_inner(commit_msg, &mut res, repo_dir, file, branch)
    .await;

  res
}

pub async fn commit_file_inner(
  commit_msg: &str,
  res: &mut RepoExecutionResponse,
  repo_dir: &Path,
  // relative to repo root
  file: &Path,
  branch: &str,
) {
  ensure_global_git_config_set().await;

  let add_log = run_komodo_standard_command(
    "Add Files",
    repo_dir,
    format!("git add {}", file.display()),
  )
  .await;
  res.logs.push(add_log);
  if !all_logs_success(&res.logs) {
    return;
  }

  let commit_log = run_komodo_standard_command(
    "Commit",
    repo_dir,
    format!(
      r#"git commit -m "[Komodo] {commit_msg}: update {file:?}""#,
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
      res.commit_hash = Some(hash);
      res.commit_message = Some(message);
    }
    Err(e) => {
      res.logs.push(Log::error(
        "Get commit hash",
        format_serror(&e.into()),
      ));
      return;
    }
  };

  let push_log = run_komodo_standard_command(
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
) -> RepoExecutionResponse {
  ensure_global_git_config_set().await;

  let mut res = RepoExecutionResponse {
    path: repo_dir.to_path_buf(),
    logs: Vec::new(),
    commit_hash: None,
    commit_message: None,
  };

  let add_log =
    run_komodo_standard_command("Add Files", repo_dir, "git add -A")
      .await;
  res.logs.push(add_log);
  if !all_logs_success(&res.logs) {
    return res;
  }

  let commit_log = run_komodo_standard_command(
    "Commit",
    repo_dir,
    format!(r#"git commit -m "[Komodo] {message}""#),
  )
  .await;
  res.logs.push(commit_log);
  if !all_logs_success(&res.logs) {
    return res;
  }

  match get_commit_hash_log(repo_dir).await {
    Ok((log, hash, message)) => {
      res.logs.push(log);
      res.commit_hash = Some(hash);
      res.commit_message = Some(message);
    }
    Err(e) => {
      res.logs.push(Log::error(
        "Get commit hash",
        format_serror(&e.into()),
      ));
      return res;
    }
  };

  let push_log = run_komodo_standard_command(
    "Push",
    repo_dir,
    format!("git push --set-upstream origin {branch}"),
  )
  .await;
  res.logs.push(push_log);

  res
}

async fn ensure_global_git_config_set() {
  let res = run_standard_command(
    "git config --global --get user.email",
    None,
  )
  .await;
  if !res.success() {
    let _ = run_standard_command(
      "git config --global user.email komodo@komo.do",
      None,
    )
    .await;
  }
  let res =
    run_standard_command("git config --global --get user.name", None)
      .await;
  if !res.success() {
    let _ = run_standard_command(
      "git config --global user.name komodo",
      None,
    )
    .await;
  }
}
