use std::{fmt::Write, path::PathBuf};

use anyhow::{anyhow, Context};
use command::run_komodo_command;
use formatting::format_serror;
use git::environment;
use komodo_client::entities::{
  all_logs_success, environment_vars_from_str,
  stack::{
    ComposeFile, ComposeService, ComposeServiceDeploy, Stack,
    StackServiceNames,
  },
  to_komodo_name,
  update::Log,
  CloneArgs, FileContents,
};
use periphery_client::api::{
  compose::ComposeUpResponse,
  git::{CloneRepo, PullOrCloneRepo, RepoActionResponse},
};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  config::periphery_config,
  docker::docker_login,
  helpers::{interpolate_variables, parse_extra_args},
};

pub fn docker_compose() -> &'static str {
  if periphery_config().legacy_compose_cli {
    "docker-compose"
  } else {
    "docker compose"
  }
}

/// If this fn returns Err, the caller of `compose_up` has to write result to the log before return.
pub async fn compose_up(
  stack: Stack,
  service: Option<String>,
  git_token: Option<String>,
  registry_token: Option<String>,
  res: &mut ComposeUpResponse,
  core_replacers: Vec<(String, String)>,
) -> anyhow::Result<()> {
  // Write the stack to local disk. For repos, will first delete any existing folder to ensure fresh deploy.
  // Will also set additional fields on the reponse.
  // Use the env_file_path in the compose command.
  let (run_directory, env_file_path) =
    write_stack(&stack, git_token, &mut *res)
      .await
      .context("Failed to write / clone compose file")?;

  // Canonicalize the path to ensure it exists, and is the cleanest path to the run directory.
  let run_directory = run_directory.canonicalize().context(
    "Failed to validate run directory on host after stack write (canonicalize error)",
  )?;

  let file_paths = stack
    .file_paths()
    .iter()
    .map(|path| {
      (
        path,
        // This will remove any intermediate uneeded '/./' in the path
        run_directory.join(path).components().collect::<PathBuf>(),
      )
    })
    .collect::<Vec<_>>();

  for (path, full_path) in &file_paths {
    if !full_path.exists() {
      res.missing_files.push(path.to_string());
    }
  }
  if !res.missing_files.is_empty() {
    return Err(anyhow!("A compose file doesn't exist after writing stack. Ensure the run_directory and file_paths are correct."));
  }

  for (path, full_path) in &file_paths {
    let file_contents =
      match fs::read_to_string(&full_path).await.with_context(|| {
        format!(
          "failed to read compose file contents at {full_path:?}"
        )
      }) {
        Ok(res) => res,
        Err(e) => {
          let error = format_serror(&e.into());
          res
            .logs
            .push(Log::error("read compose file", error.clone()));
          // This should only happen for repo stacks, ie remote error
          res.remote_errors.push(FileContents {
            path: path.to_string(),
            contents: error,
          });
          return Err(anyhow!(
          "failed to read compose file at {full_path:?}, stopping run"
        ));
        }
      };
    res.file_contents.push(FileContents {
      path: path.to_string(),
      contents: file_contents,
    });
  }

  let docker_compose = docker_compose();
  let service_arg = service
    .as_ref()
    .map(|service| format!(" {service}"))
    .unwrap_or_default();

  let file_args = if stack.config.file_paths.is_empty() {
    String::from("compose.yaml")
  } else {
    stack.config.file_paths.join(" -f ")
  };
  // This will be the last project name, which is the one that needs to be destroyed.
  // Might be different from the current project name, if user renames stack / changes to custom project name.
  let last_project_name = stack.project_name(false);
  let project_name = stack.project_name(true);

  // Login to the registry to pull private images, if provider / account are set
  if !stack.config.registry_provider.is_empty()
    && !stack.config.registry_account.is_empty()
  {
    docker_login(
      &stack.config.registry_provider,
      &stack.config.registry_account,
      registry_token.as_deref(),
    )
    .await
    .with_context(|| {
      format!(
        "domain: {} | account: {}",
        stack.config.registry_provider, stack.config.registry_account
      )
    })
    .context("failed to login to image registry")?;
  }

  let env_file = env_file_path
    .map(|path| format!(" --env-file {path}"))
    .unwrap_or_default();

  let additional_env_files = stack
    .config
    .additional_env_files
    .iter()
    .fold(String::new(), |mut output, file| {
      let _ = write!(output, " --env-file {file}");
      output
    });

  // Uses 'docker compose config' command to extract services (including image)
  // after performing interpolation
  {
    let command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{additional_env_files}{env_file} config --format json",
    );
    let config_log = run_komodo_command(
      "compose build",
      run_directory.as_ref(),
      command,
      false,
    )
    .await;
    if !config_log.success {
      res.logs.push(config_log);
      return Err(anyhow!(
        "Failed to validate compose files, stopping the run."
      ));
    }
    let compose =
      serde_json::from_str::<ComposeFile>(&config_log.stdout)
        .context("Failed to parse compose contents")?;
    for (
      service_name,
      ComposeService {
        container_name,
        deploy,
        image,
      },
    ) in compose.services
    {
      let image = image.unwrap_or_default();
      match deploy {
        Some(ComposeServiceDeploy {
          replicas: Some(replicas),
        }) if replicas > 1 => {
          for i in 1..1 + replicas {
            res.services.push(StackServiceNames {
              container_name: format!(
                "{project_name}-{service_name}-{i}"
              ),
              service_name: format!("{service_name}-{i}"),
              image: image.clone(),
            });
          }
        }
        _ => {
          res.services.push(StackServiceNames {
            container_name: container_name.unwrap_or_else(|| {
              format!("{project_name}-{service_name}")
            }),
            service_name,
            image,
          });
        }
      }
    }
  }

  // Build images before deploying.
  // If this fails, do not continue.
  if stack.config.run_build {
    let build_extra_args =
      parse_extra_args(&stack.config.build_extra_args);
    let command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} build{build_extra_args}{service_arg}",
    );
    if stack.config.skip_secret_interp {
      let log = run_komodo_command(
        "compose build",
        run_directory.as_ref(),
        command,
        false,
      )
      .await;
      res.logs.push(log);
    } else {
      let (command, mut replacers) = svi::interpolate_variables(
        &command,
        &periphery_config().secrets,
        svi::Interpolator::DoubleBrackets,
        true,
      ).context("failed to interpolate periphery secrets into stack build command")?;
      replacers.extend(core_replacers.clone());

      let mut log = run_komodo_command(
        "compose build",
        run_directory.as_ref(),
        command,
        false,
      )
      .await;

      log.command = svi::replace_in_string(&log.command, &replacers);
      log.stdout = svi::replace_in_string(&log.stdout, &replacers);
      log.stderr = svi::replace_in_string(&log.stderr, &replacers);

      res.logs.push(log);
    }

    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed to build required images, stopping the run."
      ));
    }
  }

  // Pull images before deploying
  if stack.config.auto_pull {
    // Pull images before destroying to minimize downtime.
    // If this fails, do not continue.
    let log = run_komodo_command(
      "compose pull",
      run_directory.as_ref(),
      format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} pull{service_arg}",
      ),
      false,
    )
    .await;

    res.logs.push(log);

    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed to pull required images, stopping the run."
      ));
    }
  }

  if !stack.config.pre_deploy.command.is_empty() {
    let pre_deploy_path =
      run_directory.join(&stack.config.pre_deploy.path);
    if !stack.config.skip_secret_interp {
      let (full_command, mut replacers) =
        interpolate_variables(&stack.config.pre_deploy.command)
          .context(
            "failed to interpolate secrets into pre_deploy command",
          )?;
      replacers.extend(core_replacers.to_owned());
      let mut pre_deploy_log = run_komodo_command(
        "pre deploy",
        pre_deploy_path.as_ref(),
        &full_command,
        true,
      )
      .await;

      pre_deploy_log.command =
        svi::replace_in_string(&pre_deploy_log.command, &replacers);
      pre_deploy_log.stdout =
        svi::replace_in_string(&pre_deploy_log.stdout, &replacers);
      pre_deploy_log.stderr =
        svi::replace_in_string(&pre_deploy_log.stderr, &replacers);

      tracing::debug!(
        "run Stack pre_deploy command | command: {} | cwd: {:?}",
        pre_deploy_log.command,
        pre_deploy_path
      );

      res.logs.push(pre_deploy_log);
    } else {
      let pre_deploy_log = run_komodo_command(
        "pre deploy",
        pre_deploy_path.as_ref(),
        &stack.config.pre_deploy.command,
        true,
      )
      .await;
      tracing::debug!(
        "run Stack pre_deploy command | command: {} | cwd: {:?}",
        &stack.config.pre_deploy.command,
        pre_deploy_path
      );
      res.logs.push(pre_deploy_log);
    }
    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed at running pre_deploy command, stopping the run."
      ));
    }
  }

  if stack.config.destroy_before_deploy
    // Also check if project name changed, which also requires taking down.
    || last_project_name != project_name
  {
    // Take down the existing containers.
    // This one tries to use the previously deployed service name, to ensure the right stack is taken down.
    compose_down(&last_project_name, service, res)
      .await
      .context("failed to destroy existing containers")?;
  }

  // Run compose up
  let extra_args = parse_extra_args(&stack.config.extra_args);
  let command = format!(
    "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} up -d{extra_args}{service_arg}",
  );

  let log = if stack.config.skip_secret_interp {
    run_komodo_command(
      "compose up",
      run_directory.as_ref(),
      command,
      false,
    )
    .await
  } else {
    let (command, mut replacers) = svi::interpolate_variables(
      &command,
      &periphery_config().secrets,
      svi::Interpolator::DoubleBrackets,
      true,
    ).context("failed to interpolate periphery secrets into stack run command")?;
    replacers.extend(core_replacers.clone());

    let mut log = run_komodo_command(
      "compose up",
      run_directory.as_ref(),
      command,
      false,
    )
    .await;

    log.command = svi::replace_in_string(&log.command, &replacers);
    log.stdout = svi::replace_in_string(&log.stdout, &replacers);
    log.stderr = svi::replace_in_string(&log.stderr, &replacers);

    log
  };

  res.deployed = log.success;

  // push the compose up command logs to keep the correct order
  res.logs.push(log);

  if res.deployed && !stack.config.post_deploy.command.is_empty() {
    let post_deploy_path =
        run_directory.join(&stack.config.post_deploy.path);
    if !stack.config.skip_secret_interp {
      let (full_command, mut replacers) =
          interpolate_variables(&stack.config.post_deploy.command)
              .context(
                "failed to interpolate secrets into post_deploy command",
              )?;
      replacers.extend(core_replacers);
      let mut post_deploy_log = run_komodo_command(
        "post deploy",
        post_deploy_path.as_ref(),
        &full_command,
        true,
      )
          .await;

      post_deploy_log.command =
          svi::replace_in_string(&post_deploy_log.command, &replacers);
      post_deploy_log.stdout =
          svi::replace_in_string(&post_deploy_log.stdout, &replacers);
      post_deploy_log.stderr =
          svi::replace_in_string(&post_deploy_log.stderr, &replacers);

      tracing::debug!(
        "run Stack post_deploy command | command: {} | cwd: {:?}",
        post_deploy_log.command,
        post_deploy_path
      );

      res.logs.push(post_deploy_log);
    } else {
      let post_deploy_log = run_komodo_command(
        "post deploy",
        post_deploy_path.as_ref(),
        &stack.config.post_deploy.command,
        true,
      )
          .await;
      tracing::debug!(
        "run Stack post_deploy command | command: {} | cwd: {:?}",
        &stack.config.post_deploy.command,
        post_deploy_path
      );
      res.logs.push(post_deploy_log);
    }
    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed at running post_deploy command, stopping the run."
      ));
    }
  }

  Ok(())
}

pub trait WriteStackRes {
  fn logs(&mut self) -> &mut Vec<Log>;
  fn add_remote_error(&mut self, _contents: FileContents) {}
  fn set_commit_hash(&mut self, _hash: Option<String>) {}
  fn set_commit_message(&mut self, _message: Option<String>) {}
}

impl<'a> WriteStackRes for &'a mut ComposeUpResponse {
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
/// Returns (run_directory, env_file_path)
pub async fn write_stack(
  stack: &Stack,
  git_token: Option<String>,
  mut res: impl WriteStackRes,
) -> anyhow::Result<(PathBuf, Option<&str>)> {
  let root = periphery_config()
    .stack_dir
    .join(to_komodo_name(&stack.name));
  let run_directory = root.join(&stack.config.run_directory);
  // This will remove any intermediate '/./' in the path, which is a problem for some OS.
  // Cannot use 'canonicalize' yet as directory may not exist.
  let run_directory = run_directory.components().collect::<PathBuf>();

  let env_vars = environment_vars_from_str(&stack.config.environment)
    .context("Invalid environment variables")?;

  if stack.config.files_on_host {
    // =============
    // FILES ON HOST
    // =============
    // Only need to write environment file here (which does nothing if not using this feature)
    let env_file_path = match environment::write_file(
      &env_vars,
      &stack.config.env_file_path,
      stack
        .config
        .skip_secret_interp
        .then_some(&periphery_config().secrets),
      run_directory.as_ref(),
      res.logs(),
    )
    .await
    {
      Ok(path) => path,
      Err(_) => {
        return Err(anyhow!("failed to write environment file"));
      }
    };
    Ok((
      run_directory,
      // Env file paths are already relative to run directory,
      // so need to pass original env_file_path here.
      env_file_path
        .is_some()
        .then_some(&stack.config.env_file_path),
    ))
  } else if stack.config.repo.is_empty() {
    if stack.config.file_contents.trim().is_empty() {
      return Err(anyhow!("Must either input compose file contents directly, or use file one host / git repo options."));
    }
    // ==============
    // UI BASED FILES
    // ==============
    // Ensure run directory exists
    fs::create_dir_all(&run_directory).await.with_context(|| {
      format!(
        "failed to create stack run directory at {run_directory:?}"
      )
    })?;
    let env_file_path = match environment::write_file(
      &env_vars,
      &stack.config.env_file_path,
      stack
        .config
        .skip_secret_interp
        .then_some(&periphery_config().secrets),
      run_directory.as_ref(),
      res.logs(),
    )
    .await
    {
      Ok(path) => path,
      Err(_) => {
        return Err(anyhow!("failed to write environment file"));
      }
    };
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

    let file_contents = if !stack.config.skip_secret_interp {
      let (contents, replacers) = svi::interpolate_variables(
        &stack.config.file_contents,
        &periphery_config().secrets,
        svi::Interpolator::DoubleBrackets,
        true,
      )
      .context("failed to interpolate secrets into file contents")?;
      if !replacers.is_empty() {
        res.logs().push(Log::simple(
        "Interpolate - Compose file",
        replacers
            .iter()
            .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
            .collect::<Vec<_>>()
            .join("\n"),
        ));
      }
      contents
    } else {
      stack.config.file_contents.clone()
    };

    fs::write(&file_path, &file_contents).await.with_context(
      || format!("failed to write compose file to {file_path:?}"),
    )?;

    Ok((
      run_directory,
      env_file_path
        .is_some()
        .then_some(&stack.config.env_file_path),
    ))
  } else {
    // ================
    // REPO BASED FILES
    // ================
    let mut args: CloneArgs = stack.into();
    // Set the clone destination to the one created for this run
    args.destination = Some(root.display().to_string());

    let git_token = match git_token {
      Some(token) => Some(token),
      None => {
        if !stack.config.git_account.is_empty() {
          match crate::helpers::git_token(
            &stack.config.git_provider,
            &stack.config.git_account,
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

    let env_file_path = stack
      .config
      .run_directory
      .parse::<PathBuf>()
      .context("Invalid run_directory")?
      .join(&stack.config.env_file_path)
      .display()
      .to_string();

    let clone_or_pull_res = if stack.config.reclone {
      CloneRepo {
        args,
        git_token,
        environment: env_vars,
        env_file_path,
        skip_secret_interp: stack.config.skip_secret_interp,
        // repo replacer only needed for on_clone / on_pull,
        // which aren't available for stacks
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
        skip_secret_interp: stack.config.skip_secret_interp,
        // repo replacer only needed for on_clone / on_pull,
        // which aren't available for stacks
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
    } = match clone_or_pull_res {
      Ok(res) => res,
      Err(e) => {
        let error = format_serror(
          &e.error.context("failed to pull stack repo").into(),
        );
        res
          .logs()
          .push(Log::error("pull stack repo", error.clone()));
        res.add_remote_error(FileContents {
          path: Default::default(),
          contents: error,
        });
        return Err(anyhow!(
          "failed to pull stack repo, stopping run"
        ));
      }
    };

    res.logs().extend(logs);
    res.set_commit_hash(commit_hash);
    res.set_commit_message(commit_message);

    if !all_logs_success(res.logs()) {
      return Err(anyhow!("Stopped after repo pull failure"));
    }

    Ok((
      run_directory,
      env_file_path
        .is_some()
        .then_some(&stack.config.env_file_path),
    ))
  }
}

async fn compose_down(
  project: &str,
  service: Option<String>,
  res: &mut ComposeUpResponse,
) -> anyhow::Result<()> {
  let docker_compose = docker_compose();
  let service_arg = service
    .as_ref()
    .map(|service| format!(" {service}"))
    .unwrap_or_default();
  let log = run_komodo_command(
    "compose down",
    None,
    format!("{docker_compose} -p {project} down{service_arg}"),
    false,
  )
  .await;
  let success = log.success;
  res.logs.push(log);
  if !success {
    return Err(anyhow!("Failed to bring down existing container(s) with docker compose down. Stopping run."));
  }

  Ok(())
}
