use std::path::PathBuf;

use anyhow::{Context, anyhow};
use formatting::format_serror;
use git::environment;
use komodo_client::entities::{
  CloneArgs, EnvironmentVar, FileContents, all_logs_success,
  environment_vars_from_str, repo::Repo, stack::Stack,
  to_path_compatible_name, update::Log,
};
use periphery_client::api::{
  compose::ComposeUpResponse,
  git::{CloneRepo, PullOrCloneRepo, RepoActionResponse},
};
use resolver_api::Resolve;
use tokio::fs;

use crate::config::periphery_config;

pub trait WriteStackRes {
  fn logs(&mut self) -> &mut Vec<Log>;
  fn add_remote_error(&mut self, _contents: FileContents) {}
  fn set_commit_hash(&mut self, _hash: Option<String>) {}
  fn set_commit_message(&mut self, _message: Option<String>) {}
}

impl WriteStackRes for &mut ComposeUpResponse {
  fn logs(&mut self) -> &mut Vec<Log> {
    &mut self.logs
  }
  fn add_remote_error(&mut self, contents: FileContents) {
    self.remote_errors.push(contents);
  }
  fn set_commit_hash(&mut self, hash: Option<String>) {
    self.commit_hash = hash;
  }
  fn set_commit_message(&mut self, message: Option<String>) {
    self.commit_message = message;
  }
}

/// Either writes the stack file_contents to a file, or clones the repo.
/// Performs variable replacement on env and writes file.
/// Returns (run_directory, env_file_path, periphery_replacers)
pub async fn write_stack<'a>(
  stack: &'a Stack,
  repo: Option<&Repo>,
  git_token: Option<String>,
  mut res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&'a str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  let (env_interpolated, env_replacers) =
    if stack.config.skip_secret_interp {
      (stack.config.environment.clone(), None)
    } else {
      let (environment, replacers) = svi::interpolate_variables(
        &stack.config.environment,
        &periphery_config().secrets,
        svi::Interpolator::DoubleBrackets,
        true,
      )
      .context(
        "Failed to interpolate Periphery secrets into Environment",
      )?;
      (environment, Some(replacers))
    };
  match &env_replacers {
    Some(replacers) if !replacers.is_empty() => {
      res.logs().push(Log::simple(
      "Interpolate - Environment (Periphery)",
      replacers
        .iter()
        .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
        .collect::<Vec<_>>()
        .join("\n"),
      ))
    }
    _ => {}
  }

  let env_vars = environment_vars_from_str(&env_interpolated)
    .context("Invalid environment variables")?;

  if stack.config.files_on_host {
    write_stack_files_on_host(stack, env_vars, env_replacers, res)
      .await
  } else if let Some(repo) = repo {
    write_stack_linked_repo(
      stack,
      repo,
      git_token,
      env_vars,
      env_replacers,
      res,
    )
    .await
  } else if !stack.config.repo.is_empty() {
    write_stack_inline_repo(
      stack,
      git_token,
      env_vars,
      env_replacers,
      res,
    )
    .await
  } else {
    write_stack_ui_defined(stack, env_vars, env_replacers, res).await
  }
}

async fn write_stack_files_on_host(
  stack: &Stack,
  env_vars: Vec<EnvironmentVar>,
  env_replacers: Option<Vec<(String, String)>>,
  mut res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  let run_directory = periphery_config()
    .stack_dir()
    .join(to_path_compatible_name(&stack.name))
    .join(&stack.config.run_directory)
    .components()
    .collect::<PathBuf>();
  let env_file_path = environment::write_file_simple(
    &env_vars,
    &stack.config.env_file_path,
    run_directory.as_ref(),
    res.logs(),
  )
  .await?;
  Ok((
    run_directory,
    // Env file paths are expected to be already relative to run directory,
    // so need to pass original env_file_path here.
    env_file_path
      .is_some()
      .then_some(&stack.config.env_file_path),
    env_replacers,
  ))
}

async fn write_stack_linked_repo<'a>(
  stack: &'a Stack,
  repo: &Repo,
  git_token: Option<String>,
  env_vars: Vec<EnvironmentVar>,
  env_replacers: Option<Vec<(String, String)>>,
  res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&'a str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  let root = periphery_config()
    .repo_dir()
    .join(to_path_compatible_name(&repo.name))
    .join(&repo.config.path)
    .components()
    .collect::<PathBuf>();

  let mut args: CloneArgs = repo.into();
  // Set the clone destination to the one created for this run
  args.destination = Some(root.display().to_string());

  write_stack_repo(
    stack,
    args,
    root,
    git_token,
    env_vars,
    env_replacers,
    res,
  )
  .await
}

async fn write_stack_inline_repo(
  stack: &Stack,
  git_token: Option<String>,
  env_vars: Vec<EnvironmentVar>,
  env_replacers: Option<Vec<(String, String)>>,
  res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  let root = periphery_config()
    .stack_dir()
    .join(to_path_compatible_name(&stack.name))
    .join(&stack.config.clone_path)
    .components()
    .collect::<PathBuf>();

  let mut args: CloneArgs = stack.into();
  // Set the clone destination to the one created for this run
  args.destination = Some(root.display().to_string());

  write_stack_repo(
    stack,
    args,
    root,
    git_token,
    env_vars,
    env_replacers,
    res,
  )
  .await
}

async fn write_stack_repo(
  stack: &Stack,
  args: CloneArgs,
  root: PathBuf,
  git_token: Option<String>,
  env_vars: Vec<EnvironmentVar>,
  env_replacers: Option<Vec<(String, String)>>,
  mut res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  let git_token = match git_token {
    Some(token) => Some(token),
    None => {
      if let Some(account) = &args.account {
        match crate::helpers::git_token(
          &args.account.as_deref().unwrap_or("github.com"),
          account,
        ) {
          Ok(token) => Some(token.to_string()),
          Err(e) => {
            let error = format_serror(&e.into());
            res
              .logs()
              .push(Log::error("no git token", error.clone()));
            res.add_remote_error(FileContents {
              path: Default::default(),
              contents: error,
            });
            return Err(anyhow!(
              "failed to find required git token, stopping run"
            ));
          }
        }
      } else {
        None
      }
    }
  };

  let env_file_path = root
    .join(&stack.config.run_directory)
    .join(if stack.config.env_file_path.is_empty() {
      ".env"
    } else {
      &stack.config.env_file_path
    })
    .components()
    .collect::<PathBuf>()
    .display()
    .to_string();

  let clone_or_pull_res = if stack.config.reclone {
    CloneRepo {
      args,
      git_token,
      environment: env_vars,
      env_file_path,
      // Env has already been interpolated above
      skip_secret_interp: true,
      replacers: Default::default(),
    }
    .resolve(&crate::api::Args)
    .await
  } else {
    PullOrCloneRepo {
      args,
      git_token,
      environment: env_vars,
      env_file_path,
      // Env has already been interpolated above
      skip_secret_interp: true,
      replacers: Default::default(),
    }
    .resolve(&crate::api::Args)
    .await
  };

  let RepoActionResponse {
    logs,
    commit_hash,
    commit_message,
    env_file_path,
    path: _,
  } = match clone_or_pull_res {
    Ok(res) => res,
    Err(e) => {
      let error = format_serror(
        &e.error.context("Failed to pull stack repo").into(),
      );
      res
        .logs()
        .push(Log::error("Pull Stack Repo", error.clone()));
      res.add_remote_error(FileContents {
        path: Default::default(),
        contents: error,
      });
      return Err(anyhow!("Failed to pull stack repo, stopping run"));
    }
  };

  res.logs().extend(logs);
  res.set_commit_hash(commit_hash);
  res.set_commit_message(commit_message);

  if !all_logs_success(res.logs()) {
    return Err(anyhow!("Stopped after repo pull failure"));
  }

  Ok((
    root
      .join(&stack.config.run_directory)
      .components()
      .collect(),
    env_file_path
      .is_some()
      .then_some(&stack.config.env_file_path),
    env_replacers,
  ))
}

async fn write_stack_ui_defined(
  stack: &Stack,
  env_vars: Vec<EnvironmentVar>,
  env_replacers: Option<Vec<(String, String)>>,
  mut res: impl WriteStackRes,
) -> anyhow::Result<(
  // run_directory
  PathBuf,
  // env_file_path
  Option<&str>,
  // periphery_replacers
  Option<Vec<(String, String)>>,
)> {
  if stack.config.file_contents.trim().is_empty() {
    return Err(anyhow!(
      "Must either input compose file contents directly, or use files on host / git repo options."
    ));
  }

  let run_directory = periphery_config()
    .stack_dir()
    .join(to_path_compatible_name(&stack.name))
    .components()
    .collect::<PathBuf>();

  // Ensure run directory exists
  fs::create_dir_all(&run_directory).await.with_context(|| {
    format!(
      "failed to create stack run directory at {run_directory:?}"
    )
  })?;
  let env_file_path = environment::write_file_simple(
    &env_vars,
    &stack.config.env_file_path,
    run_directory.as_ref(),
    res.logs(),
  )
  .await?;
  let file_path = run_directory
    .join(
      stack
        .config
        .file_paths
        // only need the first one, or default
        .first()
        .map(String::as_str)
        .unwrap_or("compose.yaml"),
    )
    .components()
    .collect::<PathBuf>();

  let (file_contents, file_replacers) = if !stack
    .config
    .skip_secret_interp
  {
    let (contents, replacers) = svi::interpolate_variables(
      &stack.config.file_contents,
      &periphery_config().secrets,
      svi::Interpolator::DoubleBrackets,
      true,
    )
    .context("failed to interpolate secrets into file contents")?;
    if !replacers.is_empty() {
      res.logs().push(Log::simple(
      "Interpolate - Compose file (Periphery)",
      replacers
          .iter()
          .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
          .collect::<Vec<_>>()
          .join("\n"),
      ));
    }
    (contents, Some(replacers))
  } else {
    (stack.config.file_contents.clone(), None)
  };

  fs::write(&file_path, &file_contents)
    .await
    .with_context(|| {
      format!("Failed to write compose file to {file_path:?}")
    })?;

  Ok((
    run_directory,
    env_file_path
      .is_some()
      .then_some(&stack.config.env_file_path),
    match (env_replacers, file_replacers) {
      (Some(env_replacers), Some(file_replacers)) => Some(
        env_replacers.into_iter().chain(file_replacers).collect(),
      ),
      (Some(env_replacers), None) => Some(env_replacers),
      (None, Some(file_replacers)) => Some(file_replacers),
      (None, None) => None,
    },
  ))
}
