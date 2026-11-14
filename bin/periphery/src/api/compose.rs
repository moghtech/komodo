use std::{
  borrow::Cow, fmt::Write, path::PathBuf, sync::LazyLock,
  time::Duration,
};

use anyhow::{Context, anyhow};
use command::{
  CommandOptions, KomodoCommandMode,
  run_komodo_command_with_sanitization, run_komodo_shell_command,
  run_komodo_standard_command,
};
use formatting::format_serror;
use git::write_commit_file;
use interpolate::Interpolator;
use komodo_client::{
  entities::{
    FileContents, RepoExecutionResponse, all_logs_success,
    stack::{AdditionalEnvFile, StackRemoteFileContents},
    to_path_compatible_name,
    update::Log,
  },
  parsers::parse_multiline_command,
};
use mogh_resolver::Resolve;
use periphery_client::api::{DeployStackResponse, compose::*};
use shell_escape::unix::escape;
use tracing::Instrument;

use crate::{
  config::periphery_config,
  docker::compose::{docker_compose, parse_compose_services},
  helpers::{format_extra_args, format_log_grep},
  stack::{
    maybe_login_registry, pull_or_clone_stack, validate_files,
    write::write_stack,
  },
};

impl Resolve<crate::api::Args> for GetComposeLog {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetComposeLog {
      project,
      services,
      tail,
      timestamps,
      redact_env_keys,
    } = self;
    let docker_compose = docker_compose();
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command = format!(
      "{docker_compose} -p {project} logs --tail {tail}{timestamps} -- {}",
      services.join(" ")
    );
    let mut log = run_komodo_standard_command(
      "Get Stack Log",
      command,
      CommandOptions::default().timeout(Duration::from_secs(10)),
    )
    .await;
    crate::secret_lifecycle::redact_compose_project_log(
      &project,
      &redact_env_keys,
      &mut log,
    )
    .await;
    Ok(log)
  }
}

impl Resolve<crate::api::Args> for GetComposeLogSearch {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetComposeLogSearch {
      project,
      services,
      terms,
      combinator,
      invert,
      timestamps,
      redact_env_keys,
    } = self;
    let docker_compose = docker_compose();
    let grep = format_log_grep(&terms, combinator, invert);
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let command = format!(
      "{docker_compose} -p {} logs --tail 5000{timestamps} -- {} 2>&1 | {grep}",
      escape(project.as_str().into()),
      services
        .iter()
        .map(|service| escape(service.into()))
        .collect::<Vec<_>>()
        .join(" ")
    );
    let mut log = run_komodo_shell_command(
      "Search Stack Log",
      command,
      CommandOptions::default().timeout(Duration::from_secs(10)),
    )
    .await;
    crate::secret_lifecycle::redact_compose_project_log(
      &project,
      &redact_env_keys,
      &mut log,
    )
    .await;
    Ok(log)
  }
}

//

impl Resolve<crate::api::Args> for GetComposeContentsOnHost {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<GetComposeContentsOnHostResponse> {
    let GetComposeContentsOnHost {
      name,
      run_directory,
      file_paths,
      exclude_paths,
    } = self;
    let root = periphery_config()
      .stack_dir()
      .join(to_path_compatible_name(&name));
    let run_directory =
      root.join(&run_directory).components().collect::<PathBuf>();

    let mut res = GetComposeContentsOnHostResponse::default();

    for file in file_paths {
      let normalized_rel = file
        .path
        .trim()
        .trim_start_matches("./")
        .trim_end_matches('/');
      if exclude_paths.iter().any(|ex| {
        let ex_norm =
          ex.trim().trim_start_matches("./").trim_end_matches('/');
        !ex_norm.is_empty()
          && (normalized_rel == ex_norm
            || normalized_rel.starts_with(&format!("{ex_norm}/")))
      }) {
        continue;
      }
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

impl Resolve<crate::api::Args> for WriteComposeContentsToHost {
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
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
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
    mogh_secret_file::write_async(&file_path, contents)
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

impl Resolve<crate::api::Args> for WriteCommitComposeContents {
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
    args: &crate::api::Args,
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

impl Resolve<crate::api::Args> for ComposePull {
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
    args: &crate::api::Args,
  ) -> anyhow::Result<ComposePullResponse> {
    let ComposePull {
      mut stack,
      repo,
      services,
      git_token,
      registry_token,
      mut replacers,
      external_env_vars,
      service_env_bindings,
      file_secret_bindings,
      secret_env_keys: _,
    } = self;

    let mut res = ComposePullResponse::default();

    let compose_cmd_wrapper =
      parse_multiline_command(&stack.config.compose_cmd_wrapper);
    let has_external_secrets = !external_env_vars.is_empty()
      || !service_env_bindings.is_empty()
      || !file_secret_bindings.is_empty();
    if has_external_secrets && !compose_cmd_wrapper.is_empty() {
      res.logs.push(Log::error(
        "Secret Provider Conflict",
        String::from(
          "Cannot execute Stack with both external Secret Providers and compose_cmd_wrapper configured.",
        ),
      ));
      return Ok(res);
    }

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    // Only interpolate Stack. Repo interpolation will be handled
    // by the CloneRepo / PullOrCloneRepo call.
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut res.logs);
    replacers.extend(interpolator.secret_replacers);

    let (run_directory, env_file_path) = match write_stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &external_env_vars,
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
    let run_directory = run_directory.canonicalize().with_context(||
      format!("Failed to validate run directory on host after stack write (canonicalize error), path={}", run_directory.to_string_lossy()),
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
      // `--` so a service beginning with `-` is not parsed as a flag.
      format!(" -- {}", services.join(" "))
    };

    let mut file_args = stack.compose_file_paths().join(" -f ");

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    let project_name = stack.project_name(false);

    let prepared_secrets =
      match crate::secret_lifecycle::prepare_stack_secret_generation(
        &project_name,
        &service_env_bindings,
        &file_secret_bindings,
      )
      .await
      {
        Ok(p) => p,
        Err(e) => {
          res.logs.push(Log::error(
            "Prepare External Secrets",
            format_serror(&e.into()),
          ));
          return Ok(res);
        }
      };
    if let Some(prep) = &prepared_secrets {
      write!(
        &mut file_args,
        " -f {}",
        escape(prep.override_compose_path.to_string_lossy())
      )?;
    }

    // If wrapper_include is empty but wrapper is set, use default ["up"] for backward compatibility
    let default_include = vec![String::from("up")];
    let wrapper_include =
      if stack.config.compose_cmd_wrapper_include.is_empty()
        && !compose_cmd_wrapper.is_empty()
      {
        &default_include
      } else {
        &stack.config.compose_cmd_wrapper_include
      };

    let pull_command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull{service_args}",
    );
    let (pull_command, wrapped) = match maybe_wrap_command(
      pull_command,
      &compose_cmd_wrapper,
      wrapper_include,
      "pull",
    ) {
      Ok(result) => result,
      Err(log) => {
        crate::secret_lifecycle::abort_prepared_stack_secret_generation(
          prepared_secrets,
        )
        .await;
        res.logs.push(log);
        return Ok(res);
      }
    };

    let span = info_span!("RunComposePull");
    let mode = if wrapped {
      KomodoCommandMode::Shell
    } else {
      KomodoCommandMode::Standard
    };
    let Some(log) = run_komodo_command_with_sanitization(
      "Compose Pull",
      pull_command,
      CommandOptions::default().path(run_directory.as_path()),
      mode,
      &replacers,
    )
    .instrument(span)
    .await
    else {
      // Only reachable if command is empty,
      // not the case since it is provided above.
      unreachable!()
    };

    crate::secret_lifecycle::abort_prepared_stack_secret_generation(
      prepared_secrets,
    )
    .await;

    res.logs.push(log);

    Ok(res)
  }
}

//

impl Resolve<crate::api::Args> for ComposeUp {
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
    args: &crate::api::Args,
  ) -> anyhow::Result<DeployStackResponse> {
    let ComposeUp {
      mut stack,
      repo,
      services,
      git_token,
      registry_token,
      mut replacers,
      external_env_vars,
      service_env_bindings,
      file_secret_bindings,
      mut secret_env_keys,
    } = self;

    let mut res = DeployStackResponse::default();

    let compose_cmd_wrapper =
      parse_multiline_command(&stack.config.compose_cmd_wrapper);
    let has_external_secrets = !external_env_vars.is_empty()
      || !service_env_bindings.is_empty()
      || !file_secret_bindings.is_empty()
      || !secret_env_keys.is_empty();
    if has_external_secrets && !compose_cmd_wrapper.is_empty() {
      res.logs.push(Log::error(
        "Secret Provider Conflict",
        String::from(
          "Cannot execute Stack with both external Secret Providers and compose_cmd_wrapper configured.",
        ),
      ));
      return Ok(res);
    }

    for ev in &external_env_vars {
      if !secret_env_keys.contains(&ev.variable) {
        secret_env_keys.push(ev.variable.clone());
      }
    }
    for sb in &service_env_bindings {
      if !secret_env_keys.contains(&sb.variable) {
        secret_env_keys.push(sb.variable.clone());
      }
    }

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    // Only interpolate Stack. Repo interpolation will be handled
    // by the CloneRepo / PullOrCloneRepo call.
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut res.logs);
    replacers.extend(interpolator.secret_replacers);

    let (run_directory, env_file_path) = match write_stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &external_env_vars,
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
    let run_directory = run_directory.canonicalize().with_context(||
      format!("Failed to validate run directory on host after stack write (canonicalize error), path={}", run_directory.to_string_lossy()),
    )?;

    validate_files(
      &stack,
      &run_directory,
      !external_env_vars.is_empty() || !secret_env_keys.is_empty(),
      &mut res,
    )
    .await;
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
      let span = info_span!("ExecutePreDeploy");
      if let Some(log) = run_komodo_command_with_sanitization(
        "Pre Deploy",
        &stack.config.pre_deploy.command,
        CommandOptions::default().path(pre_deploy_path.as_path()),
        if stack.config.pre_deploy.shell_mode {
          KomodoCommandMode::Shell
        } else {
          KomodoCommandMode::Multiline
        },
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
      // `--` so a service beginning with `-` is not parsed as a flag.
      format!(" -- {}", services.join(" "))
    };

    let mut file_args = stack.compose_file_paths().join(" -f ");

    // This will be the last project name, which is the one that needs to be destroyed.
    // Might be different from the current project name, if user renames stack / changes to custom project name.
    let last_project_name = stack.project_name(false);
    let project_name = stack.project_name(true);

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    let prepared_secrets =
      match crate::secret_lifecycle::prepare_stack_secret_generation(
        &project_name,
        &service_env_bindings,
        &file_secret_bindings,
      )
      .await
      {
        Ok(p) => p,
        Err(e) => {
          res.logs.push(Log::error(
            "Prepare External Secrets",
            format_serror(&e.into()),
          ));
          return Ok(res);
        }
      };
    if let Some(prep) = &prepared_secrets {
      write!(
        &mut file_args,
        " -f {}",
        escape(prep.override_compose_path.to_string_lossy())
      )?;
    }

    // If wrapper_include is empty but wrapper is set, use default ["up"] for backward compatibility
    let default_include = vec![String::from("up")];
    let wrapper_include =
      if stack.config.compose_cmd_wrapper_include.is_empty()
        && !compose_cmd_wrapper.is_empty()
      {
        &default_include
      } else {
        &stack.config.compose_cmd_wrapper_include
      };

    let deploy_step_res: anyhow::Result<()> = async {
      // Uses 'docker compose config' command to extract services (including image)
      // after performing interpolation
      {
        let command = format!(
          "{docker_compose} -p {project_name} -f {file_args}{env_file_args} config",
        );
        let (command, wrapped) = match maybe_wrap_command(
          command,
          &compose_cmd_wrapper,
          wrapper_include,
          "config",
        ) {
          Ok(result) => result,
          Err(log) => {
            res.logs.push(log);
            return Ok(());
          }
        };
        let span = info_span!("GetComposeConfig", command);
        let mut config_log = if wrapped {
          run_komodo_shell_command(
            "Compose Config",
            command,
            CommandOptions::default().path(run_directory.as_path()),
          )
          .instrument(span)
          .await
        } else {
          run_komodo_standard_command(
            "Compose Config",
            command,
            CommandOptions::default().path(run_directory.as_path()),
          )
          .instrument(span)
          .await
        };

        if !config_log.success {
          config_log.sanitize(&replacers);
          res.logs.push(config_log);
          return Ok(());
        }

        // attach sanitized + structurally redacted merged config in any case.
        let sanitized_merged =
          svi::replace_in_string(&config_log.stdout, &replacers);
        res.merged_config = Some(
          crate::secret_lifecycle::redact_compose_merged_config_yaml(
            &sanitized_merged,
            &secret_env_keys,
          ),
        );

        if let Err(e) = parse_compose_services(
          &config_log.stdout,
          &project_name,
          &mut res.services,
        ) {
          config_log.sanitize(&replacers);
          config_log.stdout =
            crate::secret_lifecycle::redact_compose_merged_config_yaml(
              &config_log.stdout,
              &secret_env_keys,
            );
          res.logs.push(config_log);
          res.logs.push(Log::error(
            "Parse Compose Services",
            format_serror(&e.into()),
          ));
          return Ok(());
        }

        config_log.sanitize(&replacers);
        config_log.stdout =
          crate::secret_lifecycle::redact_compose_merged_config_yaml(
            &config_log.stdout,
            &secret_env_keys,
          );
        res.logs.push(config_log);
      }

      if stack.config.run_build {
        let build_extra_args =
          format_extra_args(&stack.config.build_extra_args);
        let command = format!(
          "{docker_compose} -p {project_name} -f {file_args}{env_file_args} build{build_extra_args}{service_args}",
        );
        let (command, wrapped) = match maybe_wrap_command(
          command,
          &compose_cmd_wrapper,
          wrapper_include,
          "build",
        ) {
          Ok(result) => result,
          Err(log) => {
            res.logs.push(log);
            return Ok(());
          }
        };
        let mode = if wrapped {
          KomodoCommandMode::Shell
        } else {
          KomodoCommandMode::Standard
        };
        let span = info_span!("ExecuteComposeBuild");
        let Some(log) = run_komodo_command_with_sanitization(
          "Compose Build",
          command,
          CommandOptions::default().path(run_directory.as_path()),
          mode,
          &replacers,
        )
        .instrument(span)
        .await
        else {
          unreachable!()
        };
        res.logs.push(log);
        if !all_logs_success(&res.logs) {
          return Ok(());
        }
      }

      // Pull images before deploying
      if stack.config.auto_pull {
        let command = format!(
          "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull{service_args}",
        );
        let (command, wrapped) = match maybe_wrap_command(
          command,
          &compose_cmd_wrapper,
          wrapper_include,
          "pull",
        ) {
          Ok(result) => result,
          Err(log) => {
            res.logs.push(log);
            return Ok(());
          }
        };
        let mode = if wrapped {
          KomodoCommandMode::Shell
        } else {
          KomodoCommandMode::Standard
        };
        let span = info_span!("RunComposePull");
        let Some(log) = run_komodo_command_with_sanitization(
          "Compose Pull",
          command,
          CommandOptions::default().path(run_directory.as_path()),
          mode,
          &replacers,
        )
        .instrument(span)
        .await
        else {
          unreachable!()
        };
        res.logs.push(log);
        if !all_logs_success(&res.logs) {
          return Ok(());
        }
      }

      if stack.config.destroy_before_deploy
        || last_project_name != project_name
      {
        compose_down(&last_project_name, &services, &mut res)
          .await
          .context("Failed to take down existing compose stack")?;
        if last_project_name != project_name {
          let _ = crate::secret_lifecycle::purge_stack_secret_runtime(
            &last_project_name,
          )
          .await;
        }
      }

      // Run compose up
      let extra_args = format_extra_args(&stack.config.extra_args);
      let command = format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} up -d{extra_args}{service_args}",
      );
      let (command, _) = match maybe_wrap_command(
        command,
        &compose_cmd_wrapper,
        wrapper_include,
        "up",
      ) {
        Ok(result) => result,
        Err(log) => {
          res.logs.push(log);
          return Ok(());
        }
      };

      let span = info_span!("ExecuteComposeUp");
      let Some(log) = run_komodo_command_with_sanitization(
        "Compose Up",
        command,
        CommandOptions::default().path(run_directory.as_path()),
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

      Ok(())
    }
    .await;

    if let Err(e) = deploy_step_res {
      crate::secret_lifecycle::abort_prepared_stack_secret_generation(
        prepared_secrets,
      )
      .await;
      return Err(e);
    }

    if res.deployed {
      match crate::secret_lifecycle::commit_stack_secret_generation(
        &project_name,
        prepared_secrets.as_ref(),
      )
      .await
      {
        Ok(active_gen) => {
          res.active_secret_generation = active_gen;
        }
        Err(e) => {
          warn!(
            "Failed to update runtime secret manifest for stack '{project_name}': {e:#}"
          );
        }
      }
    } else {
      crate::secret_lifecycle::abort_prepared_stack_secret_generation(
        prepared_secrets,
      )
      .await;
    }

    if res.deployed && !stack.config.post_deploy.is_none() {
      let post_deploy_path =
        run_directory.join(&stack.config.post_deploy.path);
      let span = info_span!("ExecutePostDeploy");
      if let Some(log) = run_komodo_command_with_sanitization(
        "Post Deploy",
        &stack.config.post_deploy.command,
        CommandOptions::default().path(post_deploy_path.as_path()),
        if stack.config.post_deploy.shell_mode {
          KomodoCommandMode::Shell
        } else {
          KomodoCommandMode::Multiline
        },
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

impl Resolve<crate::api::Args> for ComposeExecution {
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
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let ComposeExecution {
      project,
      command,
      cleanup_runtime_secrets,
    } = self;
    let docker_compose = docker_compose();
    let log = run_komodo_standard_command(
      "Compose Command",
      format!("{docker_compose} -p {project} {command}"),
      CommandOptions::default(),
    )
    .await;
    if log.success && cleanup_runtime_secrets {
      if let Err(e) =
        crate::secret_lifecycle::purge_stack_secret_runtime(&project)
          .await
      {
        warn!(
          "Failed to purge runtime secrets for stack '{project}' after compose command: {e:#}"
        );
      }
    }
    Ok(log)
  }
}

//

impl Resolve<crate::api::Args> for ComposeRun {
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
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let ComposeRun {
      mut stack,
      repo,
      git_token,
      registry_token,
      mut replacers,
      external_env_vars,
      service_env_bindings,
      file_secret_bindings,
      secret_env_keys: _,
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

    let compose_cmd_wrapper =
      parse_multiline_command(&stack.config.compose_cmd_wrapper);
    let has_external_secrets = !external_env_vars.is_empty()
      || !service_env_bindings.is_empty()
      || !file_secret_bindings.is_empty();
    if has_external_secrets && !compose_cmd_wrapper.is_empty() {
      return Ok(Log::error(
        "Secret Provider Conflict",
        String::from(
          "Cannot execute Stack with both external Secret Providers and compose_cmd_wrapper configured.",
        ),
      ));
    }

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut Vec::new());
    replacers.extend(interpolator.secret_replacers);

    let mut res = ComposeRunResponse::default();
    let (run_directory, env_file_path) = match write_stack(
      &stack,
      repo.as_ref(),
      git_token,
      replacers.clone(),
      &external_env_vars,
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

    let run_directory = run_directory.canonicalize().with_context(||
      format!("Failed to validate run directory on host after stack write (canonicalize error), path={}", run_directory.to_string_lossy())
    )?;

    maybe_login_registry(&stack, registry_token, &mut Vec::new())
      .await;

    let docker_compose = docker_compose();

    let mut file_args = if stack.config.file_paths.is_empty() {
      String::from("compose.yaml")
    } else {
      stack.config.file_paths.join(" -f ")
    };

    let env_file_args = env_file_args(
      env_file_path,
      &stack.config.additional_env_files,
    )?;

    let project_name = stack.project_name(true);

    let prepared_secrets =
      match crate::secret_lifecycle::prepare_stack_secret_generation(
        &project_name,
        &service_env_bindings,
        &file_secret_bindings,
      )
      .await
      {
        Ok(p) => p,
        Err(e) => {
          return Ok(Log::error(
            "Prepare External Secrets",
            format_serror(&e.into()),
          ));
        }
      };
    if let Some(prep) = &prepared_secrets {
      write!(
        &mut file_args,
        " -f {}",
        escape(prep.override_compose_path.to_string_lossy())
      )?;
    }

    // Comes straight off the request and interpolated into a
    // command string which is handed to `bash -c`, so must be escaped.
    let service = escape(Cow::Borrowed(&service));

    // If wrapper_include is empty but wrapper is set, use default ["up"] for backward compatibility
    static DEFAULT_INCLUDE: LazyLock<Vec<String>> =
      LazyLock::new(|| vec![String::from("up")]);
    let wrapper_include =
      if stack.config.compose_cmd_wrapper_include.is_empty()
        && !compose_cmd_wrapper.is_empty()
      {
        &DEFAULT_INCLUDE
      } else {
        &stack.config.compose_cmd_wrapper_include
      };

    if pull.unwrap_or_default() {
      let pull_command = format!(
        "{docker_compose} -p {project_name} -f {file_args}{env_file_args} pull -- {service}",
      );
      let (pull_command, wrapped) = match maybe_wrap_command(
        pull_command,
        &compose_cmd_wrapper,
        wrapper_include,
        "pull",
      ) {
        Ok(result) => result,
        Err(log) => {
          crate::secret_lifecycle::abort_prepared_stack_secret_generation(
            prepared_secrets,
          )
          .await;
          return Ok(log);
        }
      };
      let mode = if wrapped {
        KomodoCommandMode::Shell
      } else {
        KomodoCommandMode::Standard
      };
      let Some(pull_log) = run_komodo_command_with_sanitization(
        "Compose Pull",
        pull_command,
        CommandOptions::default().path(run_directory.as_path()),
        mode,
        &replacers,
      )
      .await
      else {
        unreachable!()
      };
      if !pull_log.success {
        crate::secret_lifecycle::abort_prepared_stack_secret_generation(
          prepared_secrets,
        )
        .await;
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
      run_flags
        .push_str(&format!(" --workdir {}", escape(dir.into())));
    }
    if let Some(user) = user.as_ref() {
      run_flags.push_str(&format!(" --user {}", escape(user.into())));
    }
    if let Some(entrypoint) = entrypoint.as_ref() {
      run_flags.push_str(&format!(
        " --entrypoint {}",
        escape(entrypoint.into())
      ));
    }
    if let Some(env) = env {
      for (k, v) in env {
        // Escaped as a single KEY=VALUE token, which is what compose expects.
        run_flags.push_str(&format!(
          " -e {}",
          escape(format!("{k}={v}").into())
        ));
      }
    }

    let command_args = command
      .as_ref()
      .filter(|v| !v.is_empty())
      .map(|argv| {
        let joined = argv
          .iter()
          .map(|s| escape(s.into()).into_owned())
          .collect::<Vec<_>>()
          .join(" ");
        format!(" {joined}")
      })
      .unwrap_or_default();

    let run_command = format!(
      "{docker_compose} -p {project_name} -f {file_args}{env_file_args} run{run_flags} -- {service}{command_args}",
    );
    let (run_command, _) = match maybe_wrap_command(
      run_command,
      &compose_cmd_wrapper,
      wrapper_include,
      "run",
    ) {
      Ok(result) => result,
      Err(log) => {
        crate::secret_lifecycle::abort_prepared_stack_secret_generation(
          prepared_secrets,
        )
        .await;
        return Ok(log);
      }
    };

    let span = info_span!("RunComposeRun", run_command);
    let Some(log) = run_komodo_command_with_sanitization(
      "Compose Run",
      run_command,
      CommandOptions::default().path(run_directory.as_path()),
      KomodoCommandMode::Shell,
      &replacers,
    )
    .instrument(span)
    .await
    else {
      unreachable!()
    };

    crate::secret_lifecycle::abort_prepared_stack_secret_generation(
      prepared_secrets,
    )
    .await;

    Ok(log)
  }
}

fn env_file_args(
  env_file_path: Option<&str>,
  additional_env_files: &[AdditionalEnvFile],
) -> anyhow::Result<String> {
  let mut res = String::new();

  // Add additional env files (except komodo's own, which comes last)
  for file in additional_env_files
    .iter()
    .filter(|f| env_file_path != Some(f.path.as_str()))
  {
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

/// Apply compose_cmd_wrapper to command if the subcommand is in wrapper_include list.
/// Returns Ok((command, wrapped)) where `wrapped` indicates if wrapper was applied.
/// Returns Err(Log) if wrapper is invalid (missing placeholder).
fn maybe_wrap_command(
  command: String,
  wrapper: &str,
  wrapper_include: &[String],
  subcommand: &str,
) -> Result<(String, bool), Log> {
  // Skip wrapping if wrapper is empty or subcommand is not in include list
  if wrapper.is_empty()
    || !wrapper_include.iter().any(|c| c == subcommand)
  {
    return Ok((command, false));
  }

  // Validate wrapper contains placeholder
  if !wrapper.contains("[[COMPOSE_COMMAND]]") {
    return Err(Log::error(
      "Compose Command Wrapper",
      "compose_cmd_wrapper is configured but does not contain [[COMPOSE_COMMAND]] placeholder. The placeholder is required to inject the compose command.".to_string(),
    ));
  }

  Ok((wrapper.replace("[[COMPOSE_COMMAND]]", &command), true))
}

#[instrument("ComposeDown", skip(res))]
async fn compose_down(
  project: &str,
  services: &[String],
  res: &mut DeployStackResponse,
) -> anyhow::Result<()> {
  let docker_compose = docker_compose();
  let service_args = if services.is_empty() {
    String::new()
  } else {
    // `--` so a service beginning with `-` is not parsed as a flag.
    format!(" -- {}", services.join(" "))
  };
  let log = run_komodo_standard_command(
    "Compose Down",
    format!("{docker_compose} -p {project} down{service_args}"),
    CommandOptions::default(),
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
