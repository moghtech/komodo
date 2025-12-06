use std::fmt::Write;

use anyhow::{Context as _, anyhow};
use command::{
  KomodoCommandMode, run_komodo_command_with_sanitization,
  run_komodo_standard_command,
};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::{
  entities::{
    all_logs_success,
    docker::stack::SwarmStack,
    stack::{ComposeFile, ComposeService, StackServiceNames},
    update::Log,
  },
  parsers::parse_multiline_command,
};
use periphery_client::api::{
  DeployStackResponse,
  swarm::{DeploySwarmStack, InspectSwarmStack, RemoveSwarmStacks},
};
use resolver_api::Resolve;
use tracing::Instrument as _;

use crate::{
  config::periphery_config,
  docker::stack::inspect_swarm_stack,
  helpers::push_extra_args,
  stack::{maybe_login_registry, validate_files, write::write_stack},
};

impl Resolve<crate::api::Args> for InspectSwarmStack {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmStack> {
    inspect_swarm_stack(self.stack).await
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmStacks {
  #[instrument(
    "RemoveSwarmStacks",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stacks = serde_json::to_string(&self.stacks).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let mut command = String::from("docker stack rm");
    // This defaults to true, only need when false
    if !self.detach {
      command += " --detach=false"
    }
    for stack in self.stacks {
      command += " ";
      command += &stack;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Stacks",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for DeploySwarmStack {
  #[instrument(
    "DeploySwarmStack",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      stack = self.stack.name,
      repo = self.repo.as_ref().map(|repo| &repo.name),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> Result<Self::Response, Self::Error> {
    let DeploySwarmStack {
      mut stack,
      repo,
      git_token,
      registry_token,
      mut replacers,
    } = self;

    let mut res = DeployStackResponse::default();

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    // Only interpolate Stack. Repo interpolation will be handled
    // by the CloneRepo / PullOrCloneRepo call.
    interpolator
      .interpolate_stack(&mut stack)?
      .push_logs(&mut res.logs);
    replacers.extend(interpolator.secret_replacers);

    // Env files are not supported by docker stack deploy so are ignored.
    let (run_directory, _) = match write_stack(
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

    let use_with_registry_auth =
      maybe_login_registry(&stack, registry_token, &mut res.logs)
        .await;
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

    let file_args = stack.compose_file_paths().join(" -c ");

    // This will be the last project name, which is the one that needs to be destroyed.
    // Might be different from the current project name, if user renames stack / changes to custom project name.
    let last_project_name = stack.project_name(false);
    let project_name = stack.project_name(true);

    // Uses 'docker stack config' command to extract services (including image)
    // after performing interpolation
    {
      let command = format!("docker stack config -c {file_args}",);
      let span = info_span!("GetStackConfig", command);
      let Some(config_log) = run_komodo_command_with_sanitization(
        "Stack Config",
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
      // Store sanitized stack config output
      res.merged_config = Some(config_log.stdout);
      for (service_name, ComposeService { image, .. }) in
        compose.services
      {
        let image = image.unwrap_or_default();
        res.services.push(StackServiceNames {
          container_name: format!("{project_name}-{service_name}"),
          service_name,
          image,
        });
      }
    }

    if stack.config.destroy_before_deploy
      // Also check if project name changed, which also requires taking down.
      || last_project_name != project_name
    {
      // Take down the existing stack.
      // This one tries to use the previously deployed project name, to ensure the right stack is taken down.
      remove_stack(&last_project_name, &mut res)
        .await
        .context("Failed to destroy existing stack")?;
    }

    // Run stack deploy
    let mut command =
      format!("docker stack deploy --detach=false -c {file_args}");
    if use_with_registry_auth {
      command += " --with-registry-auth";
    }
    push_extra_args(&mut command, &stack.config.extra_args)?;
    write!(&mut command, " {project_name}")?;

    // Apply compose cmd wrapper if configured
    let compose_cmd_wrapper =
      parse_multiline_command(&stack.config.compose_cmd_wrapper);
    if !compose_cmd_wrapper.is_empty() {
      if !compose_cmd_wrapper.contains("[[COMPOSE_COMMAND]]") {
        res.logs.push(Log::error(
          "Compose Command Wrapper",
          "compose_cmd_wrapper is configured but does not contain [[COMPOSE_COMMAND]] placeholder. The placeholder is required to inject the compose command.".to_string(),
        ));
        return Ok(res);
      }
      command =
        compose_cmd_wrapper.replace("[[COMPOSE_COMMAND]]", &command);
    }

    let span = info_span!("ExecuteStackDeploy");
    let Some(log) = run_komodo_command_with_sanitization(
      "Stack Deploy",
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
      let span = info_span!("ExecutePostDeploy");
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

#[instrument("RemoveStack", skip(res))]
async fn remove_stack(
  stack: &str,
  res: &mut DeployStackResponse,
) -> anyhow::Result<()> {
  let log = run_komodo_standard_command(
    "Remove Stack",
    None,
    format!("docker stack rm --detach=false {stack}"),
  )
  .await;
  let success = log.success;
  res.logs.push(log);
  if !success {
    return Err(anyhow!(
      "Failed to remove existing stack with docker stack rm. Stopping run."
    ));
  }
  Ok(())
}
