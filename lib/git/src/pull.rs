use std::{
  path::{Path, PathBuf},
  sync::OnceLock,
};

use command::{CommandOptions, run_komodo_standard_command};
use formatting::format_serror;
use komodo_client::entities::{
  GitUpdateStrategy, RepoExecutionArgs, RepoExecutionResponse, all_logs_success,
  komodo_timestamp, update::Log,
};
use mogh_cache::TimeoutCache;

use crate::{check_installed, get_commit_hash_log};

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
  access_token: Option<String>,
) -> anyhow::Result<RepoExecutionResponse>
where
  T: Into<RepoExecutionArgs> + std::fmt::Debug,
{
  check_installed().await?;

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
    let mut set_remote = run_komodo_standard_command(
      "Set Git Remote",
      format!("git remote set-url origin {repo_url}"),
      CommandOptions::default().path(res.path.as_ref()),
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
    let fetch = run_komodo_standard_command(
      "Git Fetch",
      "git fetch --all --prune",
      CommandOptions::default().path(res.path.as_ref()),
    )
    .await;
    if !fetch.success {
      res.logs.push(fetch);
      return Ok(res);
    }

    // Protect the shared working tree before destructive checkout/reset.
    // Include untracked files, which may also belong to other stacks.
    if args.git_update_strategy != GitUpdateStrategy::Default {
      let status = run_komodo_standard_command(
        "Check local changes",
        "git status --porcelain --untracked-files=all",
        CommandOptions::default().path(res.path.as_ref()),
      )
      .await;
      let dirty = !status.stdout.trim().is_empty();
      let failed = !status.success;
      res.logs.push(status);
      if failed {
        return Ok(res);
      }
      if dirty {
        match args.git_update_strategy {
          GitUpdateStrategy::Abort => {
            res.logs.push(Log::error(
              "Git update blocked",
              "Local changes detected. Commit or stash them before updating.",
            ));
            return Ok(res);
          }
          GitUpdateStrategy::Stash => {
            // Unique marker lets us restore exactly our own stash, without
            // popping or dropping any stash created by the user.
            let marker = format!(
              "komodo-update-{}-{}",
              std::process::id(),
              komodo_timestamp()
            );
            let stash = run_komodo_standard_command(
              "Stash local changes",
              format!("git stash push --include-untracked -m {marker}"),
              CommandOptions::default().path(res.path.as_ref()),
            )
            .await;
            let stash_ok = stash.success;
            res.logs.push(stash);
            if !stash_ok {
              return Ok(res);
            }
            // Store the unique stash commit, not stash@{0}, which can move.
            let stash_ref = run_komodo_standard_command(
              "Identify Komodo stash",
              "git rev-parse refs/stash",
              CommandOptions::default().path(res.path.as_ref()),
            )
            .await;
            if !stash_ref.success {
              res.logs.push(stash_ref);
              return Ok(res);
            }
            let stash_commit = stash_ref.stdout.trim().to_string();
            res.logs.push(stash_ref);
            // Always attempt restore, even if checkout, pull, or reset fails.
            let checkout = run_komodo_standard_command(
              "Checkout branch",
              format!("git checkout -f {}", args.branch),
              CommandOptions::default().path(res.path.as_ref()),
            )
            .await;
            let mut update_success = checkout.success;
            res.logs.push(checkout);
            if update_success {
              let pull = run_komodo_standard_command(
                "Git pull",
                format!("git pull --rebase --force origin {}", args.branch),
                CommandOptions::default().path(res.path.as_ref()),
              )
              .await;
              update_success = pull.success;
              res.logs.push(pull);
            }
            if update_success {
              if let Some(commit) = &args.commit {
                let reset = run_komodo_standard_command(
                  "Set commit",
                  format!("git reset --hard {commit}"),
                  CommandOptions::default().path(res.path.as_ref()),
                )
                .await;
                update_success = reset.success;
                res.logs.push(reset);
              }
            }
            let restore = run_komodo_standard_command(
              "Restore local changes",
              format!("git stash apply --index {stash_commit}"),
              CommandOptions::default().path(res.path.as_ref()),
            )
            .await;
            let restored = restore.success;
            res.logs.push(restore);
            if restored {
              // Drop only if this stash is still at the top; otherwise keep
              // it for manual recovery instead of deleting someone else's.
              let top = run_komodo_standard_command(
                "Check Komodo stash",
                "git rev-parse refs/stash",
                CommandOptions::default().path(res.path.as_ref()),
              )
              .await;
              let same = top.success && top.stdout.trim() == stash_commit;
              res.logs.push(top);
              if same {
                let drop = run_komodo_standard_command(
                  "Drop restored stash",
                  "git stash drop stash@{0}",
                  CommandOptions::default().path(res.path.as_ref()),
                )
                .await;
                res.logs.push(drop);
              } else {
                res.logs.push(Log::simple(
                  "Keep Komodo stash",
                  format!("Local changes restored; stash {stash_commit} retained because stash order changed."),
                ));
              }
            } else {
              res.logs.push(Log::error(
                "Git stash recovery required",
                format!("Could not restore local changes automatically. Stash {stash_commit} was preserved. Resolve conflicts and recover it manually."),
              ));
            }
            if !update_success || !restored || !all_logs_success(&res.logs) {
              return Ok(res);
            }
            match get_commit_hash_log(&res.path).await {
              Ok((log, hash, message)) => {
                res.logs.push(log);
                res.commit_hash = Some(hash);
                res.commit_message = Some(message);
              }
              Err(e) => res.logs.push(Log::error("Latest Commit", format_serror(&e.into()))),
            }
            return Ok(res);
          }
          GitUpdateStrategy::Default => unreachable!(),
        }
      }
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
      format!("git pull --rebase --force origin {}", args.branch),
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
