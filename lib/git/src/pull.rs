use std::{
  path::{Path, PathBuf},
  sync::OnceLock,
};

use command::{CommandOptions, run_komodo_standard_command};
use formatting::format_serror;
use komodo_client::entities::{
  GitCredential, RepoExecutionArgs, RepoExecutionResponse,
  all_logs_success, komodo_timestamp, update::Log,
};
use mogh_cache::TimeoutCache;

use crate::{
  check_installed, get_commit_hash_log,
  ssh::{self, SshKeyFile},
};

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
pub async fn pull<T>(
  clone_args: T,
  root_repo_dir: &Path,
  credential: Option<GitCredential>,
) -> anyhow::Result<RepoExecutionResponse>
where
  T: Into<RepoExecutionArgs> + std::fmt::Debug,
{
  check_installed().await?;

  let args: RepoExecutionArgs = clone_args.into();
  let repo_url = args.remote_url(credential.as_ref())?;
  let ssh_key = SshKeyFile::maybe_write(
    args
      .ssh
      .then(|| credential.as_ref()?.ssh_key.as_deref())
      .flatten(),
  )
  .await?;
  let git = ssh::git(ssh_key.as_ref());

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
        credential.as_ref(),
        &mut res.logs,
      )
      .await;
      if !all_logs_success(&res.logs) {
        return Ok(res);
      }
    }

    // Set remote url
    let mut set_remote = run_komodo_standard_command(
      "Set Git Remote",
      format!("git remote set-url origin {repo_url}"),
      CommandOptions::default().path(res.path.as_ref()),
    )
    .await;
    // Sanitize the output
    if let Some(token) =
      credential.as_ref().and_then(|c| c.token.as_deref())
    {
      set_remote.command =
        set_remote.command.replace(token, "<TOKEN>");
      set_remote.stdout = set_remote.stdout.replace(token, "<TOKEN>");
      set_remote.stderr = set_remote.stderr.replace(token, "<TOKEN>");
    }
    res.logs.push(set_remote);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    // First fetch remote branches before checkout
    let fetch = run_komodo_standard_command(
      "Git Fetch",
      format!("{git} fetch --all --prune"),
      CommandOptions::default().path(res.path.as_ref()),
    )
    .await;
    if !fetch.success {
      res.logs.push(fetch);
      return Ok(res);
    }

    let checkout = run_komodo_standard_command(
      "Checkout branch",
      format!("git checkout -f {}", args.branch),
      CommandOptions::default().path(res.path.as_ref()),
    )
    .await;
    res.logs.push(checkout);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    let pull_log = run_komodo_standard_command(
      "Git pull",
      format!("{git} pull --rebase --force origin {}", args.branch),
      CommandOptions::default().path(res.path.as_ref()),
    )
    .await;
    res.logs.push(pull_log);
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    if let Some(commit) = args.commit {
      let reset_log = run_komodo_standard_command(
        "Set commit",
        format!("git reset --hard {commit}"),
        CommandOptions::default().path(res.path.as_ref()),
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
