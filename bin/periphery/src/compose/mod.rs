use std::{fmt::Write, path::PathBuf};

use anyhow::{Context, anyhow};
use command::run_komodo_command;
use komodo_client::entities::{
  RepoExecutionArgs, repo::Repo, stack::Stack,
  to_path_compatible_name,
};
use periphery_client::api::{
  compose::ComposeUpResponse, git::PullOrCloneRepo,
};
use resolver_api::Resolve;

use crate::config::periphery_config;

pub mod up;
pub mod write;

pub fn docker_compose() -> &'static str {
  if periphery_config().legacy_compose_cli {
    "docker-compose"
  } else {
    "docker compose"
  }
}

pub fn env_file_args(
  env_file_path: Option<&str>,
  additional_env_files: &[String],
) -> anyhow::Result<String> {
  let mut res = String::new();

  for file in additional_env_files.iter().filter(|&path| {
    let Some(komodo_path) = env_file_path else {
      return true;
    };
    // Filter komodo env out of additional env file if its also in there.
    // It will be always be added last / have highest priority.
    path != komodo_path
  }) {
    write!(res, " --env-file {file}").with_context(|| {
      format!("Failed to write --env-file arg for {file}")
    })?;
  }

  // Add this last, so it is applied on top
  if let Some(file) = env_file_path {
    write!(res, " --env-file {file}").with_context(|| {
      format!("Failed to write --env-file arg for {file}")
    })?;
  }

  Ok(res)
}

pub async fn down(
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
  let log = run_komodo_command(
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
pub async fn pull_or_clone_stack(
  stack: &Stack,
  repo: Option<&Repo>,
  git_token: Option<String>,
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
  .resolve(&crate::api::Args)
  .await
  .map_err(|e| e.error)?;

  Ok(root)
}
