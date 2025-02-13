use std::{collections::HashMap, path::Path};

use anyhow::Context;
use command::run_komodo_command;
use formatting::format_serror;
use komodo_client::entities::{
  all_logs_success, komodo_timestamp, update::Log, CloneArgs,
  EnvironmentVar,
};
use run_command::async_run_command;

use crate::{get_commit_hash_log, GitRes};

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
  repo_dir: &Path,
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
  let repo_dir = args.path(repo_dir);
  let repo_url = args.remote_url(access_token.as_deref())?;

  let mut logs = clone_inner(
    &repo_url,
    &args.branch,
    &args.commit,
    &repo_dir,
    access_token,
  )
  .await;

  if !all_logs_success(&logs) {
    tracing::warn!(
      "Failed to clone repo at {repo_dir:?} | name: {} | {logs:?}",
      args.name
    );
    return Ok(GitRes {
      logs,
      hash: None,
      message: None,
      env_file_path: None,
    });
  }

  tracing::debug!("repo at {repo_dir:?} cloned");

  let (hash, message) = match get_commit_hash_log(&repo_dir).await {
    Ok((log, hash, message)) => {
      logs.push(log);
      (Some(hash), Some(message))
    }
    Err(e) => {
      logs.push(Log::simple(
        "latest commit",
        format_serror(
          &e.context("failed to get latest commit").into(),
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
      &repo_dir,
      &mut logs,
    )
    .await
  else {
    return Ok(GitRes {
      logs,
      hash,
      message,
      env_file_path: None,
    });
  };

  if let Some(command) = args.on_clone {
    if !command.command.is_empty() {
      let on_clone_path = repo_dir.join(&command.path);
      if let Some(secrets) = secrets {
        let (full_command, mut replacers) =
          svi::interpolate_variables(
            &command.command,
            secrets,
            svi::Interpolator::DoubleBrackets,
            true,
          )
          .context(
            "failed to interpolate secrets into on_clone command",
          )?;
        replacers.extend(core_replacers.to_owned());
        let mut on_clone_log = run_komodo_command(
          "on clone",
          on_clone_path.as_ref(),
          full_command,
          true,
        )
        .await;

        on_clone_log.command =
          svi::replace_in_string(&on_clone_log.command, &replacers);
        on_clone_log.stdout =
          svi::replace_in_string(&on_clone_log.stdout, &replacers);
        on_clone_log.stderr =
          svi::replace_in_string(&on_clone_log.stderr, &replacers);

        tracing::debug!(
          "run repo on_clone command | command: {} | cwd: {:?}",
          on_clone_log.command,
          on_clone_path
        );

        logs.push(on_clone_log);
      } else {
        let on_clone_log = run_komodo_command(
          "on clone",
          on_clone_path.as_ref(),
          &command.command,
          true,
        )
        .await;
        tracing::debug!(
          "run repo on_clone command | command: {} | cwd: {:?}",
          command.command,
          on_clone_path
        );
        logs.push(on_clone_log);
      }
    }
  }
  if let Some(command) = args.on_pull {
    if !command.command.is_empty() {
      let on_pull_path = repo_dir.join(&command.path);
      if let Some(secrets) = secrets {
        let (full_command, mut replacers) =
          svi::interpolate_variables(
            &command.command,
            secrets,
            svi::Interpolator::DoubleBrackets,
            true,
          )
          .context(
            "failed to interpolate secrets into on_pull command",
          )?;
        replacers.extend(core_replacers.to_owned());
        let mut on_pull_log = run_komodo_command(
          "on pull",
          on_pull_path.as_ref(),
          &full_command,
          true,
        )
        .await;

        on_pull_log.command =
          svi::replace_in_string(&on_pull_log.command, &replacers);
        on_pull_log.stdout =
          svi::replace_in_string(&on_pull_log.stdout, &replacers);
        on_pull_log.stderr =
          svi::replace_in_string(&on_pull_log.stderr, &replacers);

        tracing::debug!(
          "run repo on_pull command | command: {} | cwd: {:?}",
          on_pull_log.command,
          on_pull_path
        );

        logs.push(on_pull_log);
      } else {
        let on_pull_log = run_komodo_command(
          "on pull",
          on_pull_path.as_ref(),
          &command.command,
          true,
        )
        .await;
        tracing::debug!(
          "run repo on_pull command | command: {} | cwd: {:?}",
          command.command,
          on_pull_path
        );
        logs.push(on_pull_log);
      }
    }
  }

  Ok(GitRes {
    logs,
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
  let _ = std::fs::remove_dir_all(destination);
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
      false,
    )
    .await;
    logs.push(reset_log);
  }

  logs
}
