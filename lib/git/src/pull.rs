use std::{
  path::{Path, PathBuf},
  sync::OnceLock,
};

use cache::TimeoutCache;
use command::run_komodo_command;
use formatting::format_serror;
use komodo_client::entities::{
  RepoExecutionArgs, RepoExecutionResponse, all_logs_success,
  komodo_timestamp, update::Log,
};

use crate::get_commit_hash_log;

/// Wait this long after a pull to allow another pull through
const PULL_TIMEOUT: i64 = 5_000;

fn pull_cache()
-> &'static TimeoutCache<PathBuf, RepoExecutionResponse> {
  static PULL_CACHE: OnceLock<
    TimeoutCache<PathBuf, RepoExecutionResponse>,
  > = OnceLock::new();
  PULL_CACHE.get_or_init(Default::default)
}

/// This will pull in a way that handles edge cases
/// from possible state of the repo. For example, the user
/// can change branch after clone, or even the remote.
#[tracing::instrument(
  level = "debug",
  skip(clone_args, access_token)
)]
#[allow(clippy::too_many_arguments)]
pub async fn pull<T>(
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

  // Acquire the path lock
  let lock = pull_cache().get_lock(res.path.clone()).await;

  // Lock the path lock, prevents simultaneous pulls by
  // ensuring simultaneous pulls will wait for first to finish
  // and checking cached results.
  let mut locked = lock.lock().await;

  // Early return from cache if lasted pulled with PULL_TIMEOUT
  if locked.last_ts + PULL_TIMEOUT > komodo_timestamp() {
    return locked.clone_res();
  }

  let res = async {
    // Check for '.git' path to see if the folder is initialized as a git repo
    let dot_git_path = res.path.join(".git");
    if !dot_git_path.exists() {
      crate::init::init_folder_as_repo(
        &res.path,
        &args,
        access_token.as_deref(),
        &mut res.logs,
      )
      .await;
      if !all_logs_success(&res.logs) {
        return Ok(res);
      }
    }

    // Set remote url
    let mut set_remote = run_komodo_command(
      "Set Git Remote",
      res.path.as_ref(),
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
    res.logs.push(set_remote);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    // First fetch remote branches before checkout
    let fetch = run_komodo_command(
      "Git Fetch",
      res.path.as_ref(),
      "git fetch --all --prune",
    )
    .await;
    if !fetch.success {
      res.logs.push(fetch);
      return Ok(res);
    }

    let checkout = run_komodo_command(
      "Checkout branch",
      res.path.as_ref(),
      format!("git checkout -f {}", args.branch),
    )
    .await;
    res.logs.push(checkout);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    let pull_log = run_komodo_command(
      "Git pull",
      res.path.as_ref(),
      format!("git pull --rebase --force origin {}", args.branch),
    )
    .await;
    res.logs.push(pull_log);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    if let Some(commit) = args.commit {
      let reset_log = run_komodo_command(
        "Set commit",
        res.path.as_ref(),
        format!("git reset --hard {commit}"),
      )
      .await;
      res.logs.push(reset_log);
      if !all_logs_success(&res.logs) {
        return Ok(res);
      }
    }

    match get_commit_hash_log(&res.path).await {
      Ok((log, hash, message)) => {
        res.logs.push(log);
        res.commit_hash = Some(hash);
        res.commit_message = Some(message);
      }
      Err(e) => {
        res.logs.push(Log::simple(
          "Latest Commit",
          format_serror(
            &e.context("Failed to get latest commit").into(),
          ),
        ));
      }
    };

    anyhow::Ok(res)
  }
  .await;

  // Set the cache with results. Any other calls waiting on the lock will
  // then immediately also use this same result.
  locked.set(&res, komodo_timestamp());

  res
}
