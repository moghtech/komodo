use std::fmt::Write;

use anyhow::Context as _;
use command::{
  KomodoCommandMode, run_komodo_command_with_sanitization,
  run_komodo_shell_command, run_komodo_standard_command,
};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::entities::{
  deployment::{
    Deployment, DeploymentConfig, DeploymentImage,
    conversions_from_str, extract_registry_domain,
  },
  docker::service::SwarmService,
  environment_vars_from_str,
  update::Log,
};
use periphery_client::api::swarm::{
  CreateSwarmService, GetSwarmServiceLog, GetSwarmServiceLogSearch,
  InspectSwarmService, RemoveSwarmServices,
};
use resolver_api::Resolve;
use tracing::Instrument;

use crate::{
  config::periphery_config,
  docker::docker_login,
  helpers::{
    format_log_grep, push_conversions, push_environment,
    push_extra_args, push_labels,
  },
  state::docker_client,
};

impl Resolve<crate::api::Args> for InspectSwarmService {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<SwarmService> {
    let client = docker_client().load();
    let client = client
      .iter()
      .next()
      .context("Could not connect to docker client")?;
    client.inspect_swarm_service(&self.service).await
  }
}

impl Resolve<crate::api::Args> for GetSwarmServiceLog {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetSwarmServiceLog {
      service,
      tail,
      timestamps,
      no_task_ids,
      no_resolve,
      details,
    } = self;
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let no_task_ids = if no_task_ids {
      " --no-task-ids"
    } else {
      Default::default()
    };
    let no_resolve = if no_resolve {
      " --no-resolve"
    } else {
      Default::default()
    };
    let details = if details {
      " --details"
    } else {
      Default::default()
    };
    let command = format!(
      "docker service logs --tail {tail}{timestamps}{no_task_ids}{no_resolve}{details} {service}",
    );
    Ok(
      run_komodo_standard_command(
        "Get Swarm Service Log",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for GetSwarmServiceLogSearch {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let GetSwarmServiceLogSearch {
      service,
      terms,
      combinator,
      invert,
      timestamps,
      no_task_ids,
      no_resolve,
      details,
    } = self;
    let timestamps = if timestamps {
      " --timestamps"
    } else {
      Default::default()
    };
    let no_task_ids = if no_task_ids {
      " --no-task-ids"
    } else {
      Default::default()
    };
    let no_resolve = if no_resolve {
      " --no-resolve"
    } else {
      Default::default()
    };
    let details = if details {
      " --details"
    } else {
      Default::default()
    };
    let grep = format_log_grep(&terms, combinator, invert);
    let command = format!(
      "docker service logs --tail 5000{timestamps}{no_task_ids}{no_resolve}{details} {service} 2>&1 | {grep}",
    );
    Ok(
      run_komodo_shell_command(
        "Search Swarm Service Log",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for RemoveSwarmServices {
  #[instrument(
    "RemoveSwarmServices",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      services = serde_json::to_string(&self.services).unwrap_or_else(|e| e.to_string()),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Log> {
    let mut command = String::from("docker service rm");
    for service in self.services {
      command += " ";
      command += &service;
    }
    Ok(
      run_komodo_standard_command(
        "Remove Swarm Services",
        None,
        command,
      )
      .await,
    )
  }
}

impl Resolve<crate::api::Args> for CreateSwarmService {
  #[instrument(
    "CreateSwarmService",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      deployment = &self.deployment.name,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> Result<Self::Response, Self::Error> {
    let CreateSwarmService {
      mut deployment,
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
        String::from(
          "Deployment does not have build replaced by Core",
        ),
      ));
    };

    let use_with_registry_auth = match docker_login(
      &extract_registry_domain(image)?,
      &deployment.config.image_registry_account,
      registry_token.as_deref(),
    )
    .await
    {
      Ok(res) => res,
      Err(e) => {
        return Ok(Log::error(
          "Docker Login",
          format_serror(
            &e.context("Failed to login to docker registry").into(),
          ),
        ));
      }
    };

    let command = docker_service_create_command(
      &deployment,
      image,
      use_with_registry_auth,
    )
    .context(
      "Unable to generate valid docker service create command",
    )?;

    let span = info_span!("ExecuteDockerServiceCreate");
    let Some(log) = run_komodo_command_with_sanitization(
      "Docker Service Create",
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

fn docker_service_create_command(
  Deployment {
    name,
    config:
      DeploymentConfig {
        volumes,
        ports,
        network,
        command,
        environment,
        labels,
        extra_args,
        ..
      },
    ..
  }: &Deployment,
  image: &str,
  use_with_registry_auth: bool,
) -> anyhow::Result<String> {
  let mut res = format!(
    "docker service create --name {name} --network {network}"
  );

  push_conversions(
    &mut res,
    &conversions_from_str(ports).context("Invalid ports")?,
    "-p",
  )?;

  push_conversions(
    &mut res,
    &conversions_from_str(volumes).context("Invalid volumes")?,
    "--mount",
  )?;

  push_environment(
    &mut res,
    &environment_vars_from_str(environment)
      .context("Invalid environment")?,
  )?;

  push_labels(
    &mut res,
    &environment_vars_from_str(labels).context("Invalid labels")?,
  )?;

  if use_with_registry_auth {
    res += " --with-registry-auth";
  }

  push_extra_args(&mut res, extra_args)?;

  write!(&mut res, " {image}")?;

  if !command.is_empty() {
    write!(&mut res, " {command}")?;
  }

  Ok(res)
}
