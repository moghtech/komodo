use std::{fmt::Write, path::Path};

use anyhow::{Context, anyhow};
use command::{
  run_komodo_command, run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::{
  entities::{
    EnvironmentVar, Version,
    build::{Build, BuildConfig},
    environment_vars_from_str, get_image_name, optional_string,
    to_komodo_name,
    update::Log,
  },
  parsers::QUOTE_PATTERN,
};
use periphery_client::api::build::{
  self, PruneBuilders, PruneBuildx,
};
use resolver_api::Resolve;

use crate::{
  config::periphery_config,
  docker::docker_login,
  helpers::{parse_extra_args, parse_labels},
};

impl Resolve<super::Args> for build::Build {
  #[instrument(name = "Build", skip_all, fields(build = self.build.name.to_string()))]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let build::Build {
      build,
      registry_token,
      additional_tags,
      replacers: core_replacers,
    } = self;
    let Build {
      name,
      config:
        BuildConfig {
          version,
          image_tag,
          skip_secret_interp,
          build_path,
          dockerfile_path,
          build_args,
          secret_args,
          labels,
          extra_args,
          use_buildx,
          image_registry,
          ..
        },
      ..
    } = &build;

    let mut logs = Vec::new();

    // Maybe docker login
    let should_push = match docker_login(
      &image_registry.domain,
      &image_registry.account,
      registry_token.as_deref(),
    )
    .await
    {
      Ok(should_push) => should_push,
      Err(e) => {
        logs.push(Log::error(
          "Docker Login",
          format_serror(
            &e.context("failed to login to docker registry").into(),
          ),
        ));
        return Ok(logs);
      }
    };

    let name = to_komodo_name(name);

    // Get paths
    let build_dir =
      periphery_config().repo_dir.join(&name).join(build_path);
    let dockerfile_path = match optional_string(dockerfile_path) {
      Some(dockerfile_path) => dockerfile_path.to_owned(),
      None => "Dockerfile".to_owned(),
    };

    // Get command parts
    let image_name =
      get_image_name(&build).context("failed to make image name")?;

    // Add VERSION to build args (if not already there)
    let mut build_args = environment_vars_from_str(build_args)
      .context("Invalid build_args")?;
    if !build_args.iter().any(|a| a.variable == "VERSION") {
      build_args.push(EnvironmentVar {
        variable: String::from("VERSION"),
        value: build.config.version.to_string(),
      });
    }
    let build_args = parse_build_args(&build_args);

    let secret_args = environment_vars_from_str(secret_args)
      .context("Invalid secret_args")?;
    let command_secret_args = parse_secret_args(
      &secret_args,
      &build_dir,
      *skip_secret_interp,
    )
    .await?;

    let labels = parse_labels(
      &environment_vars_from_str(labels).context("Invalid labels")?,
    );
    let extra_args = parse_extra_args(extra_args);
    let buildx = if *use_buildx { " buildx" } else { "" };
    let image_tags =
      image_tags(&image_name, image_tag, version, &additional_tags);
    let maybe_push = if should_push { " --push" } else { "" };

    // Construct command
    let command = format!(
      "docker{buildx} build{build_args}{command_secret_args}{extra_args}{labels}{image_tags}{maybe_push} -f {dockerfile_path} .",
    );

    if *skip_secret_interp {
      let build_log = run_komodo_command(
        "Docker Build",
        build_dir.as_ref(),
        command,
      )
      .await;
      logs.push(build_log);
    } else if let Some(log) = run_komodo_command_with_interpolation(
      "Docker Build",
      build_dir.as_ref(),
      command,
      false,
      &periphery_config().secrets,
      &core_replacers,
    )
    .await
    {
      logs.push(log)
    }

    Ok(logs)
  }
}

fn image_tags(
  image_name: &str,
  custom_tag: &str,
  version: &Version,
  additional: &[String],
) -> String {
  let Version { major, minor, .. } = version;
  let custom_tag = if custom_tag.is_empty() {
    String::new()
  } else {
    format!("-{custom_tag}")
  };
  let additional = additional
    .iter()
    .map(|tag| format!(" -t {image_name}:{tag}{custom_tag}"))
    .collect::<Vec<_>>()
    .join("");
  format!(
    " -t {image_name}:latest{custom_tag} -t {image_name}:{version}{custom_tag} -t {image_name}:{major}.{minor}{custom_tag} -t {image_name}:{major}{custom_tag}{additional}",
  )
}

fn parse_build_args(build_args: &[EnvironmentVar]) -> String {
  build_args
    .iter()
    .map(|p| {
      if p.value.starts_with(QUOTE_PATTERN)
        && p.value.ends_with(QUOTE_PATTERN)
      {
        // If the value already wrapped in quotes, don't wrap it again
        format!(" --build-arg {}={}", p.variable, p.value)
      } else {
        format!(" --build-arg {}=\"{}\"", p.variable, p.value)
      }
    })
    .collect::<Vec<_>>()
    .join("")
}

/// <https://docs.docker.com/build/building/secrets/#using-build-secrets>
async fn parse_secret_args(
  secret_args: &[EnvironmentVar],
  build_dir: &Path,
  skip_secret_interp: bool,
) -> anyhow::Result<String> {
  let periphery_config = periphery_config();
  let mut res = String::new();
  for EnvironmentVar { variable, value } in secret_args {
    // Check edge cases
    if variable.is_empty() {
      return Err(anyhow!("secret variable cannot be empty string"));
    } else if variable.contains('=') {
      return Err(anyhow!(
        "invalid variable {variable}. variable cannot contain '='"
      ));
    }
    // Interpolate in value
    let value = if skip_secret_interp {
      value.to_string()
    } else {
      svi::interpolate_variables(
        value,
        &periphery_config.secrets,
        svi::Interpolator::DoubleBrackets,
        true,
      )
      .context(
        "Failed to interpolate periphery secrets into build secrets",
      )?
      .0
    };
    // Write the value to file to mount
    let path = build_dir.join(variable);
    tokio::fs::write(&path, value).await.with_context(|| {
      format!(
        "Failed to write build secret {variable} to {}",
        path.display()
      )
    })?;
    // Extend the command
    write!(
      &mut res,
      " --secret id={variable},src={}",
      path.display()
    )
    .with_context(|| {
      format!(
        "Failed to format build secret arguments for {variable}"
      )
    })?;
  }
  Ok(res)
}

//

impl Resolve<super::Args> for PruneBuilders {
  #[instrument(name = "PruneBuilders", skip_all)]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker builder prune -a -f");
    Ok(run_komodo_command("Prune Builders", None, command).await)
  }
}

//

impl Resolve<super::Args> for PruneBuildx {
  #[instrument(name = "PruneBuildx", skip_all)]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let command = String::from("docker buildx prune -a -f");
    Ok(run_komodo_command("Prune Buildx", None, command).await)
  }
}
