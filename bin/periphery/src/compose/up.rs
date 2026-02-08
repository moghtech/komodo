use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::entities::{
  FileContents,
  stack::{Stack, StackRemoteFileContents},
  update::Log,
};
use periphery_client::api::compose::ComposeUpResponse;
use tokio::fs;

use crate::docker::docker_login;

/// Maximum number of retry attempts when reading a file that may have
/// been dynamically created (e.g. by a deploy hook or write_stack).
const FILE_READ_MAX_RETRIES: u32 = 3;
/// Delay between retry attempts.
const FILE_READ_RETRY_DELAY: std::time::Duration =
  std::time::Duration::from_millis(500);

/// Try to read a file, retrying a few times if it does not exist yet.
/// This handles the race condition where the filesystem has not yet
/// synced after a file is dynamically created.
async fn read_file_with_retry(
  path: &Path,
) -> Result<String, anyhow::Error> {
  let mut last_err = None;
  for attempt in 0..=FILE_READ_MAX_RETRIES {
    if attempt > 0 {
      tokio::time::sleep(FILE_READ_RETRY_DELAY).await;
    }
    match fs::read_to_string(path).await {
      Ok(contents) => return Ok(contents),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
        last_err = Some(e);
        // File may not exist yet — retry
        continue;
      }
      Err(e) => {
        return Err(anyhow::Error::new(e).context(format!(
          "Failed to read file contents at {path:?}"
        )));
      }
    }
  }
  Err(
    anyhow::Error::new(last_err.unwrap()).context(format!(
      "Failed to read file contents at {path:?} (file not found after {FILE_READ_MAX_RETRIES} retries)"
    )),
  )
}

pub async fn validate_files(
  stack: &Stack,
  run_directory: &Path,
  res: &mut ComposeUpResponse,
) {
  let file_paths = stack
    .all_file_dependencies()
    .into_iter()
    .map(|file| {
      (
        // This will remove any intermediate uneeded '/./' in the path
        run_directory
          .join(&file.path)
          .components()
          .collect::<PathBuf>(),
        file,
      )
    })
    .collect::<Vec<_>>();

  // First validate no missing files, retrying briefly for files that
  // may still be materialising on disk after write_stack.
  for (full_path, file) in &file_paths {
    let mut exists = full_path.exists();
    if !exists {
      for _ in 0..FILE_READ_MAX_RETRIES {
        tokio::time::sleep(FILE_READ_RETRY_DELAY).await;
        exists = full_path.exists();
        if exists {
          break;
        }
      }
    }
    if !exists {
      res.missing_files.push(file.path.clone());
    }
  }
  if !res.missing_files.is_empty() {
    res.logs.push(Log::error(
      "Validate Files",
      format_serror(
        &anyhow!(
          "Missing files: {}", res.missing_files.join(", ")
        )
        .context("Ensure the run_directory and all file paths are correct.")
        .context("A file doesn't exist after writing stack.")
        .into(),
      ),
    ));
    return;
  }

  for (full_path, file) in file_paths {
    let file_contents = match read_file_with_retry(&full_path).await {
      Ok(res) => res,
      Err(e) => {
        let error = format_serror(&e.into());
        res
          .logs
          .push(Log::error("Read Compose File", error.clone()));
        // This should only happen for repo stacks, ie remote error
        res.remote_errors.push(FileContents {
          path: file.path,
          contents: error,
        });
        return;
      }
    };
    res.file_contents.push(StackRemoteFileContents {
      path: file.path,
      contents: file_contents,
      services: file.services,
      requires: file.requires,
    });
  }
}

pub async fn maybe_login_registry(
  stack: &Stack,
  registry_token: Option<String>,
  logs: &mut Vec<Log>,
) {
  if !stack.config.registry_provider.is_empty()
    && !stack.config.registry_account.is_empty()
    && let Err(e) = docker_login(
      &stack.config.registry_provider,
      &stack.config.registry_account,
      registry_token.as_deref(),
    )
    .await
    .with_context(|| {
      format!(
        "Domain: '{}' | Account: '{}'",
        stack.config.registry_provider, stack.config.registry_account
      )
    })
    .context("Failed to login to image registry")
  {
    logs.push(Log::error(
      "Login to Registry",
      format_serror(&e.into()),
    ));
  }
}
