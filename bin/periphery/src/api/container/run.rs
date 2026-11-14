use std::fmt::Write;

use anyhow::Context;
use command::{
  CommandOptions, KomodoCommandMode,
  run_komodo_command_with_sanitization,
};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::entities::{
  deployment::{
    Deployment, DeploymentConfig, DeploymentImage, RestartMode,
    conversions_from_str, extract_registry_domain,
  },
  environment_vars_from_str,
  update::Log,
};
use mogh_resolver::Resolve;
use periphery_client::api::{
  ExternalEnvVar,
  container::{RemoveContainer, RunContainer},
};
use tracing::Instrument;

use crate::{
  config::periphery_config,
  docker::{docker_login, pull_image},
  helpers::{
    push_conversions, push_environment, push_external_environment,
    push_extra_args, push_labels,
  },
};

impl Resolve<crate::api::Args> for RunContainer {
  #[instrument(
    "DeployContainer",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      deployment = &self.deployment.name,
      stop_signal = format!("{:?}", self.stop_signal),
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let RunContainer {
      mut deployment,
      stop_signal,
      stop_time,
      registry_token,
      mut replacers,
      external_env_vars,
    } = self;

    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    interpolator.interpolate_deployment(&mut deployment)?;
    replacers.extend(interpolator.secret_replacers);

    let image = if let DeploymentImage::Image { image } =
      &deployment.config.image
    {
      if image.is_empty() {
        return Ok(Log::error(
          "Get Image",
          String::from("Deployment does not have image attached"),
        ));
      }
      image
    } else {
      return Ok(Log::error(
        "Get Image",
        String::from("Deployment does not have image attached"),
      ));
    };

    if let Err(e) = docker_login(
      &extract_registry_domain(image)?,
      &deployment.config.image_registry_account,
      registry_token.as_deref(),
    )
    .await
    {
      return Ok(Log::error(
        "Docker Login",
        format_serror(
          &e.context("Failed to login to docker registry").into(),
        ),
      ));
    }

    let _ = pull_image(image).await;
    debug!("image pulled");

    // Validate and build the complete `docker run` command BEFORE removing the existing container
    // so any validation/escaping failure fails closed without taking down the running container.
    let command =
      docker_run_command(&deployment, image, &external_env_vars)
        .context("Unable to generate valid docker run command")?;

    // Remove the existing container under the previously deployed name,
    // in case the name configuration changed since the last deploy.
    let _ = (RemoveContainer {
      name: deployment.deployed_name().to_string(),
      signal: stop_signal,
      time: stop_time,
    })
    .resolve(args)
    .await;
    debug!("container stopped and removed");

    let span = info_span!("ExecuteDockerRun");
    let Some(log) = run_komodo_command_with_sanitization(
      "Docker Run",
      command,
      CommandOptions::default(),
      KomodoCommandMode::Shell,
      &replacers,
    )
    .instrument(span)
    .await
    else {
      // The none case is only for empty command,
      // this won't be the case given it is populated above.
      unreachable!()
    };

    Ok(log)
  }
}

fn docker_run_command(
  deployment: &Deployment,
  image: &str,
  external_env_vars: &[ExternalEnvVar],
) -> anyhow::Result<String> {
  let name = deployment.custom_name();
  let DeploymentConfig {
    volumes,
    ports,
    network,
    command,
    restart,
    environment,
    labels,
    extra_args,
    ..
  } = &deployment.config;
  let mut res =
    format!("docker run -d --name {name} --network {network}");

  push_conversions(
    &mut res,
    &conversions_from_str(ports).context("Invalid ports")?,
    "-p",
  )?;

  push_conversions(
    &mut res,
    &conversions_from_str(volumes).context("Invalid volumes")?,
    "-v",
  )?;

  let external_keys: std::collections::HashSet<&str> =
    external_env_vars
      .iter()
      .map(|ev| ev.variable.as_str())
      .collect();
  let standard_env: Vec<_> = environment_vars_from_str(environment)
    .context("Invalid environment")?
    .into_iter()
    .filter(|ev| !external_keys.contains(ev.variable.as_str()))
    .collect();
  push_environment(&mut res, &standard_env)?;
  push_external_environment(&mut res, external_env_vars)?;

  push_restart(&mut res, restart)?;

  push_labels(
    &mut res,
    &environment_vars_from_str(labels).context("Invalid labels")?,
  )?;

  push_extra_args(&mut res, extra_args)?;

  write!(&mut res, " {image}")?;

  if !command.is_empty() {
    write!(&mut res, " {command}")?;
  }

  Ok(res)
}

fn push_restart(
  command: &mut String,
  restart: &RestartMode,
) -> anyhow::Result<()> {
  let restart = match restart {
    RestartMode::OnFailure => "on-failure:10",
    _ => restart.as_ref(),
  };
  write!(command, " --restart {restart}")
    .context("Failed to write restart mode")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn docker_run_command_shell_escapes_arbitrary_external_secrets_and_rejects_bad_keys()
   {
    let mut deployment = Deployment {
      name: String::from("api-svc"),
      ..Default::default()
    };
    deployment.config.environment =
      String::from("PORT=8080\nDATABASE_URL=stale\n");

    let tricky = "p@ss#word/123 'sq' \"dq\" $(rm -rf /) `whoami` \\slash\nline2;echo pwned";
    let cmd = docker_run_command(
      &deployment,
      "alpine:3.20",
      &[ExternalEnvVar {
        variable: String::from("DATABASE_URL"),
        value: String::from(tricky),
      }],
    )
    .unwrap();

    let argv = shlex::split(&cmd).expect("valid POSIX shell quoting");
    assert!(argv.contains(&String::from("PORT=8080")));
    assert!(
      !argv.contains(&String::from("DATABASE_URL=stale")),
      "external env must override duplicate standard key"
    );
    assert!(
      argv.contains(&format!("DATABASE_URL={tricky}")),
      "exact literal secret value must arrive as a single --env token"
    );

    // Reject invalid variable identifier before RemoveContainer
    assert!(
      docker_run_command(
        &deployment,
        "alpine:3.20",
        &[ExternalEnvVar {
          variable: String::from("BAD-KEY;rm"),
          value: String::from("v"),
        }],
      )
      .is_err()
    );
  }
}
