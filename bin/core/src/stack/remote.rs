use std::{fs, path::PathBuf};

use anyhow::Context;
use formatting::format_serror;
use git::RemoteState;
use komodo_client::entities::{
  FileContents, RepoExecutionArgs,
  all_logs_success,
  repo::Repo,
  stack::{Stack, StackRemoteFileContents},
  update::Log,
};

use crate::{config::core_config, helpers::git_token};

#[derive(Default)]
pub struct RemoteComposeContents {
  pub successful: Vec<StackRemoteFileContents>,
  pub errored: Vec<FileContents>,
  pub hash: Option<String>,
  pub message: Option<String>,
  // pub logs: Vec<Log>,
}

/// Returns Result<(read paths, error paths, logs, short hash, commit message)>
pub async fn get_repo_compose_contents(
  stack: &Stack,
  repo: Option<&Repo>,
  // Collect any files which are missing in the repo.
  mut missing_files: Option<&mut Vec<String>>,
) -> anyhow::Result<RemoteComposeContents> {
  let clone_args: RepoExecutionArgs =
    repo.map(Into::into).unwrap_or(stack.into());
  let (repo_path, logs, hash, message) =
    ensure_remote_repo(clone_args)
      .await
      .context("Failed to clone stack repo")?;

  // Ensure clone / pull successful,
  // propogate error log -> 'errored' and return.
  if let Some(failure) = logs.iter().find(|log| !log.success) {
    return Ok(RemoteComposeContents {
      errored: vec![FileContents {
        path: format!("Failed at: {}", failure.stage),
        contents: failure.combined(),
      }],
      ..Default::default()
    });
  }

  let run_directory = repo_path.join(&stack.config.run_directory);
  // This will remove any intermediate '/./' which can be a problem for some OS.
  let run_directory = run_directory.components().collect::<PathBuf>();

  let mut successful = Vec::new();
  let mut errored = Vec::new();

  for file in stack.all_file_dependencies() {
    let file_path = run_directory.join(&file.path);
    if !file_path.exists()
      && let Some(missing_files) = &mut missing_files
    {
      missing_files.push(file.path.clone());
    }
    // If file does not exist, will show up in err case so the log is handled
    match fs::read_to_string(&file_path).with_context(|| {
      format!("Failed to read file contents from {file_path:?}")
    }) {
      Ok(contents) => successful.push(StackRemoteFileContents {
        path: file.path,
        contents,
        services: file.services,
        requires: file.requires,
      }),
      Err(e) => errored.push(FileContents {
        path: file.path,
        contents: format_serror(&e.into()),
      }),
    }
  }

  Ok(RemoteComposeContents {
    successful,
    errored,
    hash,
    message,
  })
}

/// Returns (destination, logs, hash, message)
pub async fn ensure_remote_repo(
  mut clone_args: RepoExecutionArgs,
) -> anyhow::Result<(PathBuf, Vec<Log>, Option<String>, Option<String>)>
{
  let config = core_config();

  let access_token = if let Some(username) = &clone_args.account {
    git_token(&clone_args.provider, username, |https| {
        clone_args.https = https
      })
      .await
      .with_context(
        || format!("Failed to get git token in call to db. Stopping run. | {} | {username}", clone_args.provider),
      )?
  } else {
    None
  };

  let repo_path =
    clone_args.unique_path(&core_config().repo_directory)?;
  clone_args.destination = Some(repo_path.display().to_string());

  // Probe the remote first so we can distinguish "repo unreachable"
  // from "repo reachable but empty / branch missing" and bootstrap
  // empty remotes locally instead of failing on `git pull`.
  let remote_state = git::check_remote_state(
    &clone_args,
    access_token.as_deref(),
  )
  .await
  .context("Failed to check remote state before clone")?;

  match remote_state {
    RemoteState::Unreachable { stderr } => {
      let log = Log::error(
        "Check Remote",
        format!(
          "Remote repository is not reachable.\n\nVerify:\n  \
           \u{2022} The repository exists at {}/{}\n  \
           \u{2022} The configured git account has access to it\n  \
           \u{2022} The token has not expired\n\n\
           git stderr:\n{stderr}",
          clone_args.provider,
          clone_args.repo.as_deref().unwrap_or(""),
        ),
      );
      Ok((repo_path, vec![log], None, None))
    }
    RemoteState::Empty => {
      let mut logs =
        bootstrap_local_clone(&repo_path, &clone_args, &access_token)
          .await?;
      if !all_logs_success(&logs) {
        return Ok((repo_path, logs, None, None));
      }
      logs.push(Log::simple(
        "Check Remote",
        format!(
          "Remote repository is reachable but empty. \
           Initialized local clone on branch {}. \
           Use Initialize File to write the first commit.",
          clone_args.branch,
        ),
      ));
      Ok((repo_path, logs, None, None))
    }
    RemoteState::BranchMissing { available_branches } => {
      let mut logs =
        bootstrap_local_clone(&repo_path, &clone_args, &access_token)
          .await?;
      if !all_logs_success(&logs) {
        return Ok((repo_path, logs, None, None));
      }
      logs.push(Log::simple(
        "Check Remote",
        format!(
          "Remote repository is reachable but branch '{}' does not \
           exist. Available branches: [{}]. Use Initialize File to \
           create the branch from the first commit.",
          clone_args.branch,
          available_branches.join(", "),
        ),
      ));
      Ok((repo_path, logs, None, None))
    }
    RemoteState::BranchExists => {
      git::pull_or_clone(
        clone_args,
        &config.repo_directory,
        access_token,
      )
      .await
      .context("Failed to clone stack repo")
      .map(|(res, _)| {
        (repo_path, res.logs, res.commit_hash, res.commit_message)
      })
    }
  }
}

/// Ensure the local clone directory exists and is initialized as a git repo
/// pointing at origin + the configured branch. Used when the remote is
/// reachable but has nothing to pull (empty repo or branch missing).
async fn bootstrap_local_clone(
  repo_path: &std::path::Path,
  clone_args: &RepoExecutionArgs,
  access_token: &Option<String>,
) -> anyhow::Result<Vec<Log>> {
  let mut logs = Vec::new();
  if let Some(parent) = repo_path.parent() {
    tokio::fs::create_dir_all(parent).await.with_context(|| {
      format!("Failed to create repo parent directory {parent:?}")
    })?;
  }
  tokio::fs::create_dir_all(repo_path).await.with_context(
    || format!("Failed to create repo directory {repo_path:?}"),
  )?;
  if !repo_path.join(".git").exists() {
    git::init_folder_as_repo(
      repo_path,
      clone_args,
      access_token.as_deref(),
      &mut logs,
    )
    .await;
  }
  Ok(logs)
}
