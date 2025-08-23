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

  // First validate no missing files
  for (full_path, file) in &file_paths {
    if !full_path.exists() {
      res.missing_files.push(file.path.clone());
    }
  }
  if !res.missing_files.is_empty() {
    res.logs.push(Log::error(
      "Validate Files",
      format_serror(
        &anyhow!(
          "Ensure the run_directory and all file paths are correct."
        )
        .context("A file doesn't exist after writing stack.")
        .into(),
      ),
    ));
    return;
  }

  for (full_path, file) in file_paths {
    let file_contents =
      match fs::read_to_string(&full_path).await.with_context(|| {
        format!("Failed to read file contents at {full_path:?}")
      }) {
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
