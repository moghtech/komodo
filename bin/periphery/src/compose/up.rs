use std::{fmt::Write, path::PathBuf};

use anyhow::{Context, anyhow};
use command::{
  run_komodo_command, run_komodo_command_multiline,
  run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::entities::{
  FileContents, all_logs_success,
  repo::Repo,
  stack::{
    ComposeFile, ComposeService, ComposeServiceDeploy, Stack,
    StackServiceNames,
  },
  update::Log,
};
use periphery_client::api::compose::ComposeUpResponse;
use tokio::fs;

use crate::{
  compose::compose_down, config::periphery_config,
  docker::docker_login, helpers::parse_extra_args,
};

use super::{docker_compose, write::write_stack};

/// If this fn returns Err, the caller of `compose_up` has to write result to the log before return.
pub async fn compose_up(
  stack: Stack,
  services: Vec<String>,
  repo: Option<Repo>,
  git_token: Option<String>,
  registry_token: Option<String>,
  res: &mut ComposeUpResponse,
  core_replacers: Vec<(String, String)>,
) -> anyhow::Result<()> {
  // Write the stack to local disk. For repos, will first delete any existing folder to ensure fresh deploy.
  // Will also set additional fields on the reponse.
  // Use the env_file_path in the compose command.
  let (run_directory, env_file_path, periphery_replacers) =
    write_stack(&stack, repo.as_ref(), git_token, &mut *res)
      .await
      .context("Failed to write / clone compose file")?;

  let replacers =
    if let Some(periphery_replacers) = periphery_replacers {
      core_replacers
        .into_iter()
        .chain(periphery_replacers)
        .collect()
    } else {
      core_replacers
    };

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
    return Err(anyhow!(
      "A compose file doesn't exist after writing stack. Ensure the run_directory and file_paths are correct."
    ));
  }

  for (path, full_path) in &file_paths {
    let file_contents = match fs::read_to_string(&full_path)
      .await
      .with_context(|| {
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

  let service_args = if services.is_empty() {
    String::new()
  } else {
    format!(" {}", services.join(" "))
  };

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

  // Pre deploy command
  let pre_deploy_path =
    run_directory.join(&stack.config.pre_deploy.path);
  if let Some(log) = if stack.config.skip_secret_interp {
    run_komodo_command_multiline(
      "Pre Deploy",
      pre_deploy_path.as_ref(),
      &stack.config.pre_deploy.command,
    )
    .await
  } else {
    run_komodo_command_with_interpolation(
      "Pre Deploy",
      pre_deploy_path.as_ref(),
      &stack.config.pre_deploy.command,
      true,
      &periphery_config().secrets,
      &replacers,
    )
    .await
  } {
    res.logs.push(log);
  }
  if !all_logs_success(&res.logs) {
    return Err(anyhow!(
      "Failed at running pre_deploy command, stopping the run."
    ));
  }

  // Uses 'docker compose config' command to extract services (including image)
  // after performing interpolation
  {
    let command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{additional_env_files}{env_file} config",
    );
    let config_log = run_komodo_command(
      "Compose Config",
      run_directory.as_ref(),
      command,
    )
    .await;
    if !config_log.success {
      res.logs.push(config_log);
      return Err(anyhow!(
        "Failed to validate compose files, stopping the run."
      ));
    }
    // Record sanitized compose config output
    res.compose_config =
      Some(svi::replace_in_string(&config_log.stdout, &replacers));
    let compose =
      serde_yaml::from_str::<ComposeFile>(&config_log.stdout)
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
      "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} build{build_extra_args}{service_args}",
    );
    if stack.config.skip_secret_interp {
      let log = run_komodo_command(
        "Compose Build",
        run_directory.as_ref(),
        command,
      )
      .await;
      res.logs.push(log);
    } else if let Some(log) = run_komodo_command_with_interpolation(
      "Compose Build",
      run_directory.as_ref(),
      command,
      false,
      &periphery_config().secrets,
      &replacers,
    )
    .await
    {
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
      "Compose Pull",
      run_directory.as_ref(),
      format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} pull{service_args}",
      ),
    )
    .await;

    res.logs.push(log);

    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed to pull required images, stopping the run."
      ));
    }
  }

  if stack.config.destroy_before_deploy
    // Also check if project name changed, which also requires taking down.
    || last_project_name != project_name
  {
    // Take down the existing containers.
    // This one tries to use the previously deployed service name, to ensure the right stack is taken down.
    compose_down(&last_project_name, &services, res)
      .await
      .context("failed to destroy existing containers")?;
  }

  // Run compose up
  let extra_args = parse_extra_args(&stack.config.extra_args);
  let command = format!(
    "{docker_compose} -p {project_name} -f {file_args}{env_file}{additional_env_files} up -d{extra_args}{service_args}",
  );

  let log = if stack.config.skip_secret_interp {
    run_komodo_command("Compose Up", run_directory.as_ref(), command)
      .await
  } else {
    match run_komodo_command_with_interpolation(
      "Compose Up",
      run_directory.as_ref(),
      command,
      false,
      &periphery_config().secrets,
      &replacers,
    )
    .await
    {
      Some(log) => log,
      // The command is definitely non-empty, the result will never be None.
      None => unreachable!(),
    }
  };

  res.deployed = log.success;

  // push the compose up command logs to keep the correct order
  res.logs.push(log);

  if res.deployed {
    let post_deploy_path =
      run_directory.join(&stack.config.post_deploy.path);
    if let Some(log) = if stack.config.skip_secret_interp {
      run_komodo_command_multiline(
        "Post Deploy",
        post_deploy_path.as_ref(),
        &stack.config.post_deploy.command,
      )
      .await
    } else {
      run_komodo_command_with_interpolation(
        "Post Deploy",
        post_deploy_path.as_ref(),
        &stack.config.post_deploy.command,
        true,
        &periphery_config().secrets,
        &replacers,
      )
      .await
    } {
      res.logs.push(log)
    }
    if !all_logs_success(&res.logs) {
      return Err(anyhow!(
        "Failed at running post_deploy command, stopping the run."
      ));
    }
  }

  Ok(())
}
