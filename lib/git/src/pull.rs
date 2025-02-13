use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::OnceLock,
};

use anyhow::Context;
use cache::TimeoutCache;
use command::run_komodo_command;
use formatting::format_serror;
use komodo_client::entities::{
  komodo_timestamp, update::Log, CloneArgs, EnvironmentVar,
};

use crate::{get_commit_hash_log, GitRes};

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
  let path = args.path(repo_dir);

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
    let repo_url = args.remote_url(access_token.as_deref())?;

    // Set remote url
    let mut set_remote = run_komodo_command(
      "set git remote",
      path.as_ref(),
      format!("git remote set-url origin {repo_url}"),
      false,
    )
    .await;

    if !set_remote.success {
      if let Some(token) = access_token {
        set_remote.command =
          set_remote.command.replace(&token, "<TOKEN>");
        set_remote.stdout =
          set_remote.stdout.replace(&token, "<TOKEN>");
        set_remote.stderr =
          set_remote.stderr.replace(&token, "<TOKEN>");
      }
      return Ok(GitRes {
        logs: vec![set_remote],
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    let checkout = run_komodo_command(
      "checkout branch",
      path.as_ref(),
      format!("git checkout -f {}", args.branch),
      false,
    )
    .await;

    if !checkout.success {
      return Ok(GitRes {
        logs: vec![checkout],
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    let pull_log = run_komodo_command(
      "git pull",
      path.as_ref(),
      format!("git pull --rebase --force origin {}", args.branch),
      false,
    )
    .await;

    let mut logs = vec![pull_log];

    if !logs[0].success {
      return Ok(GitRes {
        logs,
        hash: None,
        message: None,
        env_file_path: None,
      });
    }

    if let Some(commit) = args.commit {
      let reset_log = run_komodo_command(
        "set commit",
        path.as_ref(),
        format!("git reset --hard {commit}"),
        false,
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
        &path,
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

    if let Some(command) = args.on_pull {
      if !command.command.is_empty() {
        let on_pull_path = path.join(&command.path);
        if let Some(secrets) = secrets {
          let (full_command, mut replacers) =
            match svi::interpolate_variables(
              &command.command,
              secrets,
              svi::Interpolator::DoubleBrackets,
              true,
            )
            .context(
              "failed to interpolate secrets into on_pull command",
            ) {
              Ok(res) => res,
              Err(e) => {
                logs.push(Log::error(
                  "interpolate secrets - on_pull",
                  format_serror(&e.into()),
                ));
                return Ok(GitRes {
                  logs,
                  hash,
                  message,
                  env_file_path: None,
                });
              }
            };
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

    anyhow::Ok(GitRes {
      logs,
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
