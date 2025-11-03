use std::{
  fmt::Write,
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use command::run_komodo_standard_command;
use formatting::format_serror;
use komodo_client::entities::{
  FileContents, RepoExecutionArgs,
  repo::Repo,
  stack::{AdditionalEnvFile, Stack, StackRemoteFileContents},
  to_path_compatible_name,
  update::Log,
};
use periphery_client::api::{
  compose::ComposeUpResponse, git::PullOrCloneRepo,
};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  api::Args, config::periphery_config, docker::docker_login,
};

use super::docker_compose;

#[instrument(
  "MaybeLoginRegistry",
  skip_all,
  fields(stack = stack.name)
)]
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

pub fn env_file_args(
  env_file_path: Option<&str>,
  additional_env_files: &[AdditionalEnvFile],
) -> anyhow::Result<String> {
  let mut res = String::new();

  // Add additional env files (except komodo's own, which comes last)
  for file in additional_env_files.iter().filter(|f| {
    env_file_path.map_or(true, |komodo_path| f.path.as_str() != komodo_path)
  }) {
    let path = &file.path;
    write!(res, " --env-file {path}").with_context(|| {
      format!("Failed to write --env-file arg for {path}")
    })?;
  }

  // Add komodo's env file last for highest priority
  if let Some(file) = env_file_path {
    write!(res, " --env-file {file}").with_context(|| {
      format!("Failed to write --env-file arg for {file}")
    })?;
  }

  Ok(res)
}

#[instrument("ComposeDown", skip(res))]
pub async fn compose_down(
  project: &str,
  services: &[String],
  res: &mut ComposeUpResponse,
) -> anyhow::Result<()> {
  let docker_compose = docker_compose();
  let service_args = if services.is_empty() {
    String::new()
  } else {
    format!(" {}", services.join(" "))
  };
  let log = run_komodo_standard_command(
    "Compose Down",
    None,
    format!("{docker_compose} -p {project} down{service_args}"),
  )
  .await;
  let success = log.success;
  res.logs.push(log);
  if !success {
    return Err(anyhow!(
      "Failed to bring down existing container(s) with docker compose down. Stopping run."
    ));
  }

  Ok(())
}

/// Only for git repo based Stacks.
/// Returns path to root directory of the stack repo.
///
/// Both Stack and Repo environment, on clone, on pull are ignored.
#[instrument(
  "PullOrCloneStack",
  skip_all,
  fields(
    stack = stack.name,
    repo = repo.as_ref().map(|repo| &repo.name),
  )
)]
pub async fn pull_or_clone_stack(
  stack: &Stack,
  repo: Option<&Repo>,
  git_token: Option<String>,
  req_args: &Args,
) -> anyhow::Result<PathBuf> {
  if stack.config.files_on_host {
    return Err(anyhow!(
      "Wrong method called for files on host stack"
    ));
  }
  if repo.is_none() && stack.config.repo.is_empty() {
    return Err(anyhow!("Repo is not configured"));
  }

  let (root, mut args) = if let Some(repo) = repo {
    let root = periphery_config()
      .repo_dir()
      .join(to_path_compatible_name(&repo.name))
      .join(&repo.config.path)
      .components()
      .collect::<PathBuf>();
    let args: RepoExecutionArgs = repo.into();
    (root, args)
  } else {
    let root = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&stack.name))
      .join(&stack.config.clone_path)
      .components()
      .collect::<PathBuf>();
    let args: RepoExecutionArgs = stack.into();
    (root, args)
  };
  args.destination = Some(root.display().to_string());

  let git_token = crate::helpers::git_token(git_token, &args)?;

  PullOrCloneRepo {
    args,
    git_token,
    // All the extra pull functions
    // (env, on clone, on pull)
    // are disabled with this method.
    environment: Default::default(),
    env_file_path: Default::default(),
    on_clone: Default::default(),
    on_pull: Default::default(),
    skip_secret_interp: Default::default(),
    replacers: Default::default(),
  }
  .resolve(req_args)
  .await?;

  Ok(root)
}

#[instrument(
  "ValidateStackFiles",
  skip(stack, res),
  fields(stack = stack.name)
)]
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
