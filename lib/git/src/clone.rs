use std::{collections::HashMap, path::Path};

use command::{
  run_komodo_command, run_komodo_command_multiline,
  run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::entities::{
  CloneArgs, EnvironmentVar, all_logs_success, komodo_timestamp,
  update::Log,
};
use run_command::async_run_command;

use crate::{GitRes, get_commit_hash_log};

/// Will delete the existing repo folder,
/// clone the repo, get the latest hash / message,
/// and run on_clone / on_pull.
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
pub async fn clone<T>(
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
  let path = args.path(root_repo_dir);
  let repo_url = args.remote_url(access_token.as_deref())?;

  let mut logs = clone_inner(
    &repo_url,
    &args.branch,
    &args.commit,
    &path,
    access_token,
  )
  .await;

  if !all_logs_success(&logs) {
    tracing::warn!(
      "Failed to clone repo at {path:?} | name: {} | {logs:?}",
      args.name
    );
    return Ok(GitRes {
      logs,
      path,
      hash: None,
      message: None,
      env_file_path: None,
    });
  }

  tracing::debug!("repo at {path:?} cloned");

  let (hash, message) = match get_commit_hash_log(&path).await {
    Ok((log, hash, message)) => {
      logs.push(log);
      (Some(hash), Some(message))
    }
    Err(e) => {
      logs.push(Log::simple(
        "Latest Commit",
        format_serror(
          &e.context("Failed to get latest commit").into(),
        ),
      ));
      (None, None)
    }
  };

  let Ok((env_file_path, _replacers)) =
    crate::environment::write_file(
      environment,
      env_file_path,
      secrets,
      &path,
      &mut logs,
    )
    .await
  else {
    return Ok(GitRes {
      logs,
      path,
      hash,
      message,
      env_file_path: None,
    });
  };

  if let Some(command) = args.on_clone {
    let on_clone_path = path.join(&command.path);
    if let Some(log) = if let Some(secrets) = secrets {
      run_komodo_command_with_interpolation(
        "On Clone",
        Some(on_clone_path.as_path()),
        &command.command,
        true,
        secrets,
        core_replacers,
      )
      .await
    } else {
      run_komodo_command_multiline(
        "On Clone",
        Some(on_clone_path.as_path()),
        &command.command,
      )
      .await
    } {
      logs.push(log)
    };
  }
  if let Some(command) = args.on_pull {
    let on_pull_path = path.join(&command.path);
    if let Some(log) = if let Some(secrets) = secrets {
      run_komodo_command_with_interpolation(
        "On Pull",
        Some(on_pull_path.as_path()),
        &command.command,
        true,
        secrets,
        core_replacers,
      )
      .await
    } else {
      run_komodo_command_multiline(
        "On Pull",
        Some(on_pull_path.as_path()),
        &command.command,
      )
      .await
    } {
      logs.push(log)
    };
  }

  Ok(GitRes {
    logs,
    path,
    hash,
    message,
    env_file_path,
  })
}

async fn clone_inner(
  repo_url: &str,
  branch: &str,
  commit: &Option<String>,
  destination: &Path,
  access_token: Option<String>,
) -> Vec<Log> {
  let _ = tokio::fs::remove_dir_all(destination).await;

  // Ensure parent folder exists
  if let Some(parent) = destination.parent() {
    let _ = tokio::fs::create_dir_all(parent).await;
  }

  let command = format!(
    "git clone {repo_url} {} -b {branch}",
    destination.display()
  );
  let start_ts = komodo_timestamp();
  let output = async_run_command(&command).await;
  let success = output.success();
  let (command, stderr) = if let Some(token) = access_token {
    (
      command.replace(&token, "<TOKEN>"),
      output.stderr.replace(&token, "<TOKEN>"),
    )
  } else {
    (command, output.stderr)
  };
  let mut logs = vec![Log {
    stage: "clone repo".to_string(),
    command,
    success,
    stdout: output.stdout,
    stderr,
    start_ts,
    end_ts: komodo_timestamp(),
  }];

  if !logs[0].success {
    return logs;
  }

  if let Some(commit) = commit {
    let reset_log = run_komodo_command(
      "set commit",
      destination,
      format!("git reset --hard {commit}",),
    )
    .await;
    logs.push(reset_log);
  }

  logs
}
