use std::{borrow::Cow, path::PathBuf};

use anyhow::{Context, anyhow};
use command::{
  KomodoCommandMode, run_komodo_command_with_sanitization,
  run_komodo_shell_command, run_komodo_standard_command,
};
use formatting::format_serror;
use git::write_commit_file;
use interpolate::Interpolator;
use komodo_client::entities::{
  FileContents, RepoExecutionResponse, all_logs_success,
  stack::{
    ComposeFile, ComposeProject, ComposeService,
    ComposeServiceDeploy, StackRemoteFileContents, StackServiceNames,
  },
  to_path_compatible_name,
  update::Log,
};
use periphery_client::api::compose::*;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use shell_escape::unix::escape;
use tracing::Instrument;

use crate::{
  config::periphery_config,
  helpers::{format_extra_args, format_log_grep},
};

mod helpers;
mod write;

use helpers::*;

fn docker_compose() -> &'static str {
  if periphery_config().legacy_compose_cli {
    "docker-compose"
  } else {
    "docker compose"
  }
}

pub async fn list_compose_projects()
-> anyhow::Result<Vec<ComposeProject>> {
  let docker_compose = docker_compose();
  let res = run_komodo_standard_command(
    "List Projects",
    None,
    format!("{docker_compose} ls --all --format json"),
  )
  .await;

  if !res.success {
    return Err(anyhow!("{}", res.combined()).context(format!(
      "failed to list compose projects using {docker_compose} ls"
    )));
  }

  let res =
    serde_json::from_str::<Vec<DockerComposeLsItem>>(&res.stdout)
      .with_context(|| res.stdout.clone())
      .with_context(|| {
        format!(
          "failed to parse '{docker_compose} ls' response to json"
        )
      })?
      .into_iter()
      .filter(|item| !item.name.is_empty())
      .map(|item| ComposeProject {
        name: item.name,
        status: item.status,
        compose_files: item
          .config_files
          .split(',')
          .map(str::to_string)
          .collect(),
      })
      .collect();

  Ok(res)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeLsItem {
  #[serde(default, alias = "Name")]
  pub name: String,
  #[serde(alias = "Status")]
  pub status: Option<String>,
  /// Comma seperated list of paths
  #[serde(default, alias = "ConfigFiles")]
  pub config_files: String,
}

//

impl Resolve<super::Args> for GetComposeLog {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
    let GetComposeLog {
      project,
      services,
      tail,
      timestamps,
    } = self;
    let docker_compose = docker_compose();
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command = format!(
      "{docker_compose} -p {project} logs --tail {tail}{timestamps} {}",
      services.join(" ")
    );
    Ok(
      run_komodo_standard_command("get stack log", None, command)
        .await,
    )
  }
}

impl Resolve<super::Args> for GetComposeLogSearch {
  async fn resolve(self, _: &super::Args) -> anyhow::Result<Log> {
    let GetComposeLogSearch {
      project,
      services,
      terms,
      combinator,
      invert,
      timestamps,
    } = self;
    let docker_compose = docker_compose();
    let grep = format_log_grep(&terms, combinator, invert);
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command = format!(
      "{docker_compose} -p {project} logs --tail 5000{timestamps} {} 2>&1 | {grep}",
      services.join(" ")
    );
    Ok(
      run_komodo_shell_command("Get stack log grep", None, command)
        .await,
    )
  }
}

//

impl Resolve<super::Args> for GetComposeContentsOnHost {
  async fn resolve(
    self,
    _: &super::Args,
  ) -> anyhow::Result<GetComposeContentsOnHostResponse> {
    let GetComposeContentsOnHost {
      name,
      run_directory,
      file_paths,
    } = self;
    let root = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&name));
    let run_directory =
      root.join(&run_directory).components().collect::<PathBuf>();

    let mut res = GetComposeContentsOnHostResponse::default();

    for file in file_paths {
      let full_path = run_directory
        .join(&file.path)
        .components()
        .collect::<PathBuf>();
      match tokio::fs::read_to_string(&full_path).await.with_context(
        || {
          format!(
            "Failed to read compose file contents at {full_path:?}"
          )
        },
      ) {
        Ok(contents) => {
          // The path we store here has to be the same as incoming file path in the array,
          // in order for WriteComposeContentsToHost to write to the correct path.
          res.contents.push(StackRemoteFileContents {
            path: file.path,
            contents,
            services: file.services,
            requires: file.requires,
          });
        }
        Err(e) => {
          res.errors.push(FileContents {
            path: file.path,
            contents: format_serror(&e.into()),
          });
        }
      }
    }

    Ok(res)
  }
}

//

impl Resolve<super::Args> for WriteComposeContentsToHost {
  #[instrument(
    "WriteComposeContentsToHost",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = self.name,
      run_directory = self.run_directory,
      file_path = self.file_path,
    )
  )]
  async fn resolve(self, args: &super::Args) -> anyhow::Result<Log> {
    let WriteComposeContentsToHost {
      name,
      run_directory,
      file_path,
      contents,
    } = self;
    let file_path = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&name))
      .join(&run_directory)
      .join(file_path)
      .components()
      .collect::<PathBuf>();
    secret_file::write_async(&file_path, contents)
      .await
      .with_context(|| {
        format!(
          "Failed to write compose file contents to {file_path:?}"
        )
      })?;
    Ok(Log::simple(
      "Write contents to host",
      format!("File contents written to {file_path:?}"),
    ))
  }
}

//

impl Resolve<super::Args> for WriteCommitComposeContents {
  #[instrument(
    "WriteCommitComposeContents",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = &self.stack.name,
      username = &self.username,
      file_path = &self.file_path,
    )
  )]
  async fn resolve(
    self,
    args: &super::Args,
  ) -> anyhow::Result<RepoExecutionResponse> {
    let WriteCommitComposeContents {
      stack,
      repo,
      username,
      file_path,
      contents,
      git_token,
    } = self;

    let root =
      pull_or_clone_stack(&stack, repo.as_ref(), git_token, args)
        .await?;

    let file_path = stack
      .config
      .run_directory
      .parse::<PathBuf>()
      .context("Run directory is not a valid path")?
      .join(&file_path);

    let msg = if let Some(username) = username {
      format!("{username}: Write Compose File")
    } else {
      "Write Compose File".to_string()
    };

    write_commit_file(
      &msg,
      &root,
      &file_path,
      &contents,
      &stack.config.branch,
    )
    .await
  }
}

//

impl Resolve<super::Args> for ComposePull {
  #[instrument(
    "ComposePull",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = &self.stack.name,
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    args: &super::Args,
  ) -> anyhow::Result<ComposePullResponse> {
    let ComposePull {
      mut stack,
      repo,
      services,
      git_token,
      registry_token,
      mut replacers,
    } = self;

    let mut res = ComposePullResponse::default();

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    // Only interpolate Stack. Repo interpolation will be handled
    // by the CloneRepo / PullOrCloneRepo call.
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut res.logs);
    replacers.extend(interpolator.secret_replacers);

    let (run_directory, env_file_path) = match write::stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &mut res,
      args,
    )
    .await
    {
      Ok(res) => res,
      Err(e) => {
        res
          .logs
          .push(Log::error("Write Stack", format_serror(&e.into())));
        return Ok(res);
      }
    };

    // Canonicalize the path to ensure it exists, and is the cleanest path to the run directory.
    let run_directory = run_directory.canonicalize().context(
      "Failed to validate run directory on host after stack write (canonicalize error)",
    )?;

    let file_paths = stack
      .all_tracked_file_paths()
      .into_iter()
      .map(|path| {
        (
          // This will remove any intermediate uneeded '/./' in the path
          run_directory.join(&path).components().collect::<PathBuf>(),
          path,
        )
      })
      .collect::<Vec<_>>();

    // Validate files
    for (full_path, path) in &file_paths {
      if !full_path.exists() {
        return Err(anyhow!("Missing compose file at {path}"));
      }
    }

    maybe_login_registry(&stack, registry_token, &mut res.logs).await;
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    let docker_compose = docker_compose();

    let service_args = if services.is_empty() {
      String::new()
    } else {
      format!(" {}", services.join(" "))
    };

    let file_args = stack.compose_file_paths().join(" -f ");

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    let project_name = stack.project_name(false);

    let span = info_span!("RunComposePull");
    let log = run_komodo_standard_command(
      "Compose Pull",
      run_directory.as_ref(),
      format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull{service_args}",
      ),
    )
    .instrument(span)
    .await;

    res.logs.push(log);

    Ok(res)
  }
}

//

impl Resolve<super::Args> for ComposeUp {
  #[instrument(
    "ComposeUp",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = self.stack.name,
      repo = self.repo.as_ref().map(|repo| &repo.name),
      services = format!("{:?}", self.services),
    )
  )]
  async fn resolve(
    self,
    args: &super::Args,
  ) -> anyhow::Result<ComposeUpResponse> {
    let ComposeUp {
      mut stack,
      repo,
      services,
      git_token,
      registry_token,
      mut replacers,
    } = self;

    let mut res = ComposeUpResponse::default();

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    // Only interpolate Stack. Repo interpolation will be handled
    // by the CloneRepo / PullOrCloneRepo call.
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut res.logs);
    replacers.extend(interpolator.secret_replacers);

    let (run_directory, env_file_path) = match write::stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &mut res,
      args,
    )
    .await
    {
      Ok(res) => res,
      Err(e) => {
        res
          .logs
          .push(Log::error("Write Stack", format_serror(&e.into())));
        return Ok(res);
      }
    };

    // Canonicalize the path to ensure it exists, and is the cleanest path to the run directory.
    let run_directory = run_directory.canonicalize().context(
      "Failed to validate run directory on host after stack write (canonicalize error)",
    )?;

    validate_files(&stack, &run_directory, &mut res).await;
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    maybe_login_registry(&stack, registry_token, &mut res.logs).await;
    if !all_logs_success(&res.logs) {
      return Ok(res);
    }

    // Pre deploy
    if !stack.config.pre_deploy.is_none() {
      let pre_deploy_path =
        run_directory.join(&stack.config.pre_deploy.path);
      let span = info_span!("RunPreDeploy");
      if let Some(log) = run_komodo_command_with_sanitization(
        "Pre Deploy",
        pre_deploy_path.as_path(),
        &stack.config.pre_deploy.command,
        KomodoCommandMode::Multiline,
        &replacers,
      )
      .instrument(span)
      .await
      {
        res.logs.push(log);
        if !all_logs_success(&res.logs) {
          return Ok(res);
        }
      };
    }

    let docker_compose = docker_compose();

    let service_args = if services.is_empty() {
      String::new()
    } else {
      format!(" {}", services.join(" "))
    };

    let file_args = stack.compose_file_paths().join(" -f ");

    // This will be the last project name, which is the one that needs to be destroyed.
    // Might be different from the current project name, if user renames stack / changes to custom project name.
    let last_project_name = stack.project_name(false);
    let project_name = stack.project_name(true);

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    // Uses 'docker compose config' command to extract services (including image)
    // after performing interpolation
    {
      let command = format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} config",
      );
      let span = info_span!("GetComposeConfig", command);
      let Some(config_log) = run_komodo_command_with_sanitization(
        "Compose Config",
        run_directory.as_path(),
        command,
        KomodoCommandMode::Standard,
        &replacers,
      )
      .instrument(span)
      .await
      else {
        // Only reachable if command is empty,
        // not the case since it is provided above.
        unreachable!()
      };
      if !config_log.success {
        res.logs.push(config_log);
        return Ok(res);
      }
      let compose =
        serde_yaml_ng::from_str::<ComposeFile>(&config_log.stdout)
          .context("Failed to parse compose contents")?;
      // Record sanitized compose config output
      res.compose_config = Some(config_log.stdout);
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

    if stack.config.run_build {
      let build_extra_args =
        format_extra_args(&stack.config.build_extra_args);
      let command = format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} build{build_extra_args}{service_args}",
      );
      let span = info_span!("RunComposeBuild");
      let Some(log) = run_komodo_command_with_sanitization(
        "Compose Build",
        run_directory.as_path(),
        command,
        KomodoCommandMode::Shell,
        &replacers,
      )
      .instrument(span)
      .await
      else {
        unreachable!()
      };
      res.logs.push(log);
      if !all_logs_success(&res.logs) {
        return Ok(res);
      }
    }

    // Pull images before deploying
    if stack.config.auto_pull {
      // Pull images before destroying to minimize downtime.
      // If this fails, do not continue.
      let command = format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull{service_args}",
      );
      let span = info_span!("RunComposePull");
      let log = run_komodo_standard_command(
        "Compose Pull",
        run_directory.as_ref(),
        command,
      )
      .instrument(span)
      .await;
      res.logs.push(log);
      if !all_logs_success(&res.logs) {
        return Ok(res);
      }
    }

    if stack.config.destroy_before_deploy
      // Also check if project name changed, which also requires taking down.
      || last_project_name != project_name
    {
      // Take down the existing containers.
      // This one tries to use the previously deployed service name, to ensure the right stack is taken down.
      helpers::compose_down(&last_project_name, &services, &mut res)
        .await
        .context("failed to destroy existing containers")?;
    }

    // Run compose up
    let extra_args = format_extra_args(&stack.config.extra_args);
    let command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{env_file_args} up -d{extra_args}{service_args}",
    );

    let span = info_span!("RunComposeUp");
    let Some(log) = run_komodo_command_with_sanitization(
      "Compose Up",
      run_directory.as_path(),
      command,
      KomodoCommandMode::Shell,
      &replacers,
    )
    .instrument(span)
    .await
    else {
      unreachable!()
    };

    res.deployed = log.success;
    res.logs.push(log);

    if res.deployed && !stack.config.post_deploy.is_none() {
      let post_deploy_path =
        run_directory.join(&stack.config.post_deploy.path);
      let span = info_span!("RunPostDeploy");
      if let Some(log) = run_komodo_command_with_sanitization(
        "Post Deploy",
        post_deploy_path.as_path(),
        &stack.config.post_deploy.command,
        KomodoCommandMode::Multiline,
        &replacers,
      )
      .instrument(span)
      .await
      {
        res.logs.push(log);
      };
    }

    Ok(res)
  }
}

//

impl Resolve<super::Args> for ComposeExecution {
  #[instrument(
    "ComposeExecution",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      project = self.project,
      command = self.command,
    )
  )]
  async fn resolve(self, args: &super::Args) -> anyhow::Result<Log> {
    let ComposeExecution { project, command } = self;
    let docker_compose = docker_compose();
    let log = run_komodo_standard_command(
      "Compose Command",
      None,
      format!("{docker_compose} -p {project} {command}"),
    )
    .await;
    Ok(log)
  }
}

//

impl Resolve<super::Args> for ComposeRun {
  #[instrument(
    "ComposeRun",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = self.stack.name,
      repo = self.repo.as_ref().map(|repo| &repo.name),
      service = &self.service
    )
  )]
  async fn resolve(self, args: &super::Args) -> anyhow::Result<Log> {
    let ComposeRun {
      mut stack,
      repo,
      git_token,
      registry_token,
      mut replacers,
      service,
      command,
      no_tty,
      no_deps,
      detach,
      service_ports,
      env,
      workdir,
      user,
      entrypoint,
      pull,
    } = self;

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut Vec::new());
    replacers.extend(interpolator.secret_replacers);

    let mut res = ComposeRunResponse::default();
    let (run_directory, env_file_path) = match write::stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &mut res,
      args,
    )
    .await
    {
      Ok(res) => res,
      Err(e) => {
        return Ok(Log::error(
          "Write Stack",
          format_serror(&e.into()),
        ));
      }
    };

    let run_directory = run_directory.canonicalize().context(
      "Failed to validate run directory on host after stack write (canonicalize error)",
    )?;

    maybe_login_registry(&stack, registry_token, &mut Vec::new())
      .await;

    let docker_compose = docker_compose();

    let file_args = if stack.config.file_paths.is_empty() {
      String::from("compose.yaml")
    } else {
      stack.config.file_paths.join(" -f ")
    };

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    let project_name = stack.project_name(true);

    if pull.unwrap_or_default() {
      let pull_log = run_komodo_standard_command(
        "Compose Pull",
        run_directory.as_ref(),
        format!(
          "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull {service}",
        ),
      )
      .await;
      if !pull_log.success {
        return Ok(pull_log);
      }
    }

    let mut run_flags = String::from(" --rm");
    if detach.unwrap_or_default() {
      run_flags.push_str(" -d");
    }
    if no_tty.unwrap_or_default() {
      run_flags.push_str(" --no-tty");
    }
    if no_deps.unwrap_or_default() {
      run_flags.push_str(" --no-deps");
    }
    if service_ports.unwrap_or_default() {
      run_flags.push_str(" --service-ports");
    }
    if let Some(dir) = workdir.as_ref() {
      run_flags.push_str(&format!(" --workdir {dir}"));
    }
    if let Some(user) = user.as_ref() {
      run_flags.push_str(&format!(" --user {user}"));
    }
    if let Some(entrypoint) = entrypoint.as_ref() {
      run_flags.push_str(&format!(" --entrypoint {entrypoint}"));
    }
    if let Some(env) = env {
      for (k, v) in env {
        run_flags.push_str(&format!(" -e {}={} ", k, v));
      }
    }

    let command_args = command
      .as_ref()
      .filter(|v| !v.is_empty())
      .map(|argv| {
        let joined = argv
          .iter()
          .map(|s| escape(Cow::Borrowed(s)).into_owned())
          .collect::<Vec<_>>()
          .join(" ");
        format!(" {joined}")
      })
      .unwrap_or_default();

    let command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{env_file_args} run{run_flags} {service}{command_args}",
    );

    let span = info_span!("RunComposeRun", command);
    let Some(log) = run_komodo_command_with_sanitization(
      "Compose Run",
      run_directory.as_path(),
      command,
      KomodoCommandMode::Shell,
      &replacers,
    )
    .instrument(span)
    .await
    else {
      unreachable!()
    };

    Ok(log)
  }
}
