use std::fmt::Write;

use anyhow::Context;
use command::{
  KomodoCommandMode, run_komodo_command_with_sanitization,
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
use periphery_client::api::container::{
  RemoveContainer, RunContainer,
};
use resolver_api::Resolve;
use tracing::Instrument;

use crate::{
  config::periphery_config,
  docker::{docker_login, pull_image},
  helpers::{
    push_conversions, push_environment, push_extra_args, push_labels,
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

    let _ = (RemoveContainer {
      name: deployment.name.clone(),
      signal: stop_signal,
      time: stop_time,
    })
    .resolve(args)
    .await;
    debug!("container stopped and removed");

    let command = docker_run_command(&deployment, image)
      .context("Unable to generate valid docker run command")?;

    let span = info_span!("ExecuteDockerRun");
    let Some(log) = run_komodo_command_with_sanitization(
      "Docker Run",
      None,
      command,
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
  Deployment {
    name,
    config:
      DeploymentConfig {
        volumes,
        ports,
        network,
        command,
        restart,
        environment,
        labels,
        extra_args,
        ..
      },
    ..
  }: &Deployment,
  image: &str,
) -> anyhow::Result<String> {
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

  push_environment(
    &mut res,
    &environment_vars_from_str(environment)
      .context("Invalid environment")?,
  )?;

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
