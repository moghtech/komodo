use anyhow::Context;
use command::{
  run_komodo_command, run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::{
  entities::{
    EnvironmentVar,
    deployment::{
      Conversion, Deployment, DeploymentConfig, DeploymentImage,
      RestartMode, conversions_from_str, extract_registry_domain,
    },
    environment_vars_from_str, to_docker_compatible_name,
    update::Log,
  },
  parsers::QUOTE_PATTERN,
};
use periphery_client::api::container::{Deploy, RemoveContainer};
use resolver_api::Resolve;

use crate::{
  config::periphery_config,
  docker::{docker_login, pull_image},
  helpers::{parse_extra_args, parse_labels},
};

impl Resolve<super::Args> for Deploy {
  #[instrument(
    name = "Deploy",
    skip_all,
    fields(
      stack = &self.deployment.name,
      stop_signal = format!("{:?}", self.stop_signal),
      stop_time = self.stop_time,
    )
  )]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let Deploy {
      deployment,
      stop_signal,
      stop_time,
      registry_token,
      replacers: core_replacers,
    } = self;
    let image = if let DeploymentImage::Image { image } =
      &deployment.config.image
    {
      if image.is_empty() {
        return Ok(Log::error(
          "get image",
          String::from("deployment does not have image attached"),
        ));
      }
      image
    } else {
      return Ok(Log::error(
        "get image",
        String::from("deployment does not have image attached"),
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
        "docker login",
        format_serror(
          &e.context("failed to login to docker registry").into(),
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
    .resolve(&super::Args)
    .await;
    debug!("container stopped and removed");

    let command = docker_run_command(&deployment, image)
      .context("Unable to generate valid docker run command")?;
    debug!("docker run command: {command}");

    if deployment.config.skip_secret_interp {
      Ok(run_komodo_command("Docker Run", None, command).await)
    } else {
      match run_komodo_command_with_interpolation(
        "Docker Run",
        None,
        command,
        false,
        &periphery_config().secrets,
        &core_replacers,
      )
      .await
      {
        Some(log) => Ok(log),
        // The None case can not be reached, as the command is always non-empty
        None => unreachable!(),
      }
    }
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
  let name = to_docker_compatible_name(name);
  let ports = parse_conversions(
    &conversions_from_str(ports).context("Invalid ports")?,
    "-p",
  );
  let volumes = parse_conversions(
    &conversions_from_str(volumes).context("Invalid volumes")?,
    "-v",
  );
  let network = parse_network(network);
  let restart = parse_restart(restart);
  let environment = parse_environment(
    &environment_vars_from_str(environment)
      .context("Invalid environment")?,
  );
  let labels = parse_labels(
    &environment_vars_from_str(labels).context("Invalid labels")?,
  );
  let command = parse_command(command);
  let extra_args = parse_extra_args(extra_args);
  let command = format!(
    "docker run -d --name {name}{ports}{volumes}{network}{restart}{environment}{labels}{extra_args} {image}{command}"
  );
  Ok(command)
}

fn parse_conversions(
  conversions: &[Conversion],
  flag: &str,
) -> String {
  conversions
    .iter()
    .map(|p| format!(" {flag} {}:{}", p.local, p.container))
    .collect::<Vec<_>>()
    .join("")
}

fn parse_environment(environment: &[EnvironmentVar]) -> String {
  environment
    .iter()
    .map(|p| {
      if p.value.starts_with(QUOTE_PATTERN)
        && p.value.ends_with(QUOTE_PATTERN)
      {
        // If the value already wrapped in quotes, don't wrap it again
        format!(" --env {}={}", p.variable, p.value)
      } else {
        format!(" --env {}=\"{}\"", p.variable, p.value)
      }
    })
    .collect::<Vec<_>>()
    .join("")
}

fn parse_network(network: &str) -> String {
  format!(" --network {network}")
}

fn parse_restart(restart: &RestartMode) -> String {
  let restart = match restart {
    RestartMode::OnFailure => "on-failure:10".to_string(),
    _ => restart.to_string(),
  };
  format!(" --restart {restart}")
}

fn parse_command(command: &str) -> String {
  if command.is_empty() {
    String::new()
  } else {
    format!(" {command}")
  }
}
