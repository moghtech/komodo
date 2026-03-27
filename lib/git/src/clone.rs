use std::{io::ErrorKind, path::Path};

use anyhow::Context;
use command::run_komodo_standard_command;
use formatting::format_serror;
use komodo_client::entities::{
  RepoExecutionArgs, RepoExecutionResponse, all_logs_success,
  update::Log,
};

use crate::get_commit_hash_log;

/// Will delete the existing repo folder,
/// clone the repo, get the latest hash / message,
/// and run on_clone / on_pull.
///
/// Assumes all interpolation is already done and takes the list of replacers
/// for the On Clone command.
pub async fn clone<T>(
  clone_args: T,
  root_repo_dir: &Path,
  access_token: Option<String>,
) -> anyhow::Result<RepoExecutionResponse>
where
  T: Into<RepoExecutionArgs> + std::fmt::Debug,
{
  let args: RepoExecutionArgs = clone_args.into();
  let repo_url = args.remote_url(access_token.as_deref())?;

  let mut res = RepoExecutionResponse {
    path: args.path(root_repo_dir),
    logs: Vec::new(),
    commit_hash: None,
    commit_message: None,
  };

  // Ensure parent folder exists
  if let Some(parent) = res.path.parent()
    && let Err(e) = tokio::fs::create_dir_all(parent)
      .await
      .context("Failed to create clone parent directory.")
  {
    res.logs.push(Log::error(
      "Prepare Repo Root",
      format_serror(&e.into()),
    ));
    return Ok(res);
  }

  match tokio::fs::remove_dir_all(&res.path).await {
    Err(e) if e.kind() != ErrorKind::NotFound => {
      let e: anyhow::Error = e.into();
      res.logs.push(Log::error(
        "Clean Repo Root",
        format_serror(
          &e.context(
            "Failed to remove existing repo root before clone.",
          )
          .into(),
        ),
      ));
      return Ok(res);
    }
    _ => {}
  }

  let command = format!(
    "git clone {repo_url} {} -b {}",
    res.path.display(),
    args.branch
  );

  let mut log =
    run_komodo_standard_command("Clone Repo", None, command).await;

  if let Some(token) = access_token {
    log.command = log.command.replace(&token, "<TOKEN>");
    log.stdout = log.stdout.replace(&token, "<TOKEN>");
    log.stderr = log.stderr.replace(&token, "<TOKEN>");
  }

  res.logs.push(log);

  if !all_logs_success(&res.logs) {
    return Ok(res);
  }

  if let Some(commit) = args.commit {
    let reset_log = run_komodo_standard_command(
      "set commit",
      res.path.as_path(),
      format!("git reset --hard {commit}",),
    )
    .await;
    res.logs.push(reset_log);
  }

  if !all_logs_success(&res.logs) {
    return Ok(res);
  }

  match get_commit_hash_log(&res.path)
    .await
    .context("Failed to get latest commit")
  {
    Ok((log, hash, message)) => {
      res.logs.push(log);
      res.commit_hash = Some(hash);
      res.commit_message = Some(message);
    }
    Err(e) => {
      res
        .logs
        .push(Log::simple("Latest Commit", format_serror(&e.into())));
    }
  };

  Ok(res)
}
