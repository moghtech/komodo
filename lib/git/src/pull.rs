use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::OnceLock,
};

use cache::TimeoutCache;
use command::{
  run_komodo_command, run_komodo_command_multiline,
  run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::entities::{
  CloneArgs, EnvironmentVar, all_logs_success, komodo_timestamp,
  update::Log,
};

use crate::{GitRes, get_commit_hash_log};

/// Wait this long after a pull to allow another pull through
const PULL_TIMEOUT: i64 = 5_000;

fn pull_cache() -> &'static TimeoutCache<PathBuf, GitRes> {
  static PULL_CACHE: OnceLock<TimeoutCache<PathBuf, GitRes>> =
    OnceLock::new();
  PULL_CACHE.get_or_init(Default::default)
}

/// This will pull in a way that handles edge cases
/// from possible state of the repo. For example, the user
/// can change branch after clone, or even the remote.
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
#[allow(clippy::too_many_arguments)]
pub async fn pull<T>(
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

  // Acquire the path lock
  let lock = pull_cache().get_lock(path.clone()).await;

  // Lock the path lock, prevents simultaneous pulls by
  // ensuring simultaneous pulls will wait for first to finish
  // and checking cached results.
  let mut locked = lock.lock().await;

  // Early return from cache if lasted pulled with PULL_TIMEOUT
  if locked.last_ts + PULL_TIMEOUT > komodo_timestamp() {
    return locked.clone_res();
  }

  let res = async {
    let mut logs = Vec::new();

    // Check for '.git' path to see if the folder is initialized as a git repo
    let dot_git_path = path.join(".git");
    if !dot_git_path.exists() {
      crate::init::init_folder_as_repo(
        &path,
        &args,
        access_token.as_deref(),
        &mut logs,
      )
      .await;
      if !all_logs_success(&logs) {
        return Ok(GitRes {
          logs,
          path,
          hash: None,
          message: None,
          env_file_path: None,
        });
      }
    }

    // Set remote url
    let mut set_remote = run_komodo_command(
      "Set git remote",
      path.as_ref(),
      format!("git remote set-url origin {repo_url}"),
    )
    .await;
    // Sanitize the output
    if let Some(token) = access_token {
      set_remote.command =
        set_remote.command.replace(&token, "<TOKEN>");
      set_remote.stdout =
        set_remote.stdout.replace(&token, "<TOKEN>");
      set_remote.stderr =
        set_remote.stderr.replace(&token, "<TOKEN>");
    }
    logs.push(set_remote);
    if !all_logs_success(&logs) {
      return Ok(GitRes {
        logs,
        path,
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    let checkout = run_komodo_command(
      "Checkout branch",
      path.as_ref(),
      format!("git checkout -f {}", args.branch),
    )
    .await;
    logs.push(checkout);
    if !all_logs_success(&logs) {
      return Ok(GitRes {
        logs,
        path,
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    let pull_log = run_komodo_command(
      "Git pull",
      path.as_ref(),
      format!("git pull --rebase --force origin {}", args.branch),
    )
    .await;
    logs.push(pull_log);
    if !all_logs_success(&logs) {
      return Ok(GitRes {
        logs,
        path,
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    if let Some(commit) = args.commit {
      let reset_log = run_komodo_command(
        "Set commit",
        path.as_ref(),
        format!("git reset --hard {commit}"),
      )
      .await;
      logs.push(reset_log);
    }

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

    anyhow::Ok(GitRes {
      logs,
      path,
      hash,
      message,
      env_file_path,
    })
  }
  .await;

  // Set the cache with results. Any other calls waiting on the lock will
  // then immediately also use this same result.
  locked.set(&res, komodo_timestamp());

  res
}
