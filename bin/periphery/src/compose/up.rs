use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::entities::{
  FileContents, stack::Stack, update::Log,
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
    .all_file_paths()
    .into_iter()
    .map(|path| {
      (
        // This will remove any intermediate uneeded '/./' in the path
        run_directory.join(&path).components().collect::<PathBuf>(),
        path,
      )
    })
    .collect::<Vec<_>>();

  for (full_path, path) in &file_paths {
    if !full_path.exists() {
      res.missing_files.push(path.to_string());
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

  for (full_path, path) in &file_paths {
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
            path: path.to_string(),
            contents: error,
          });
          return;
        }
      };
    res.file_contents.push(FileContents {
      path: path.to_string(),
      contents: file_contents,
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
