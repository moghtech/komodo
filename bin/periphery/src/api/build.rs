use std::{
  fmt::Write,
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use command::{
  run_komodo_command, run_komodo_command_multiline,
  run_komodo_command_with_interpolation,
};
use formatting::format_serror;
use komodo_client::{
  entities::{
    EnvironmentVar, Version,
    build::{Build, BuildConfig},
    environment_vars_from_str, get_image_name, optional_string,
    to_path_compatible_name,
    update::Log,
  },
  parsers::QUOTE_PATTERN,
};
use periphery_client::api::build::{
  self, GetDockerfileContentsOnHost,
  GetDockerfileContentsOnHostResponse, PruneBuilders, PruneBuildx,
  WriteDockerfileContentsToHost,
};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  config::periphery_config,
  docker::docker_login,
  helpers::{parse_extra_args, parse_labels},
};

impl Resolve<super::Args> for GetDockerfileContentsOnHost {
  #[instrument(name = "GetDockerfileContentsOnHost", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<GetDockerfileContentsOnHostResponse> {
    let GetDockerfileContentsOnHost {
      name,
      build_path,
      dockerfile_path,
    } = self;

    let root = periphery_config()
      .build_dir()
      .join(to_path_compatible_name(&name));
    let build_dir =
      root.join(&build_path).components().collect::<PathBuf>();

    if !build_dir.exists() {
      fs::create_dir_all(&build_dir)
        .await
        .context("Failed to initialize build directory")?;
    }

    let full_path = build_dir
      .join(&dockerfile_path)
      .components()
      .collect::<PathBuf>();

    let contents =
      fs::read_to_string(&full_path).await.with_context(|| {
        format!("Failed to read dockerfile contents at {full_path:?}")
      })?;

    Ok(GetDockerfileContentsOnHostResponse {
      contents,
      path: full_path.display().to_string(),
    })
  }
}

impl Resolve<super::Args> for WriteDockerfileContentsToHost {
  #[instrument(
    name = "WriteDockerfileContentsToHost",
    skip_all,
    fields(
      stack = &self.name,
      build_path = &self.build_path,
      dockerfile_path = &self.dockerfile_path,
    )
  )]
  async fn resolve(self, _: &super::Args) -> serror::Result<Log> {
    let WriteDockerfileContentsToHost {
      name,
      build_path,
      dockerfile_path,
      contents,
    } = self;
    let full_path = periphery_config()
      .build_dir()
      .join(to_path_compatible_name(&name))
      .join(&build_path)
      .join(dockerfile_path)
      .components()
      .collect::<PathBuf>();
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
      if !parent.exists() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to initialize dockerfile parent directory {parent:?}"))?;
      }
    }
    fs::write(&full_path, contents).await.with_context(|| {
      format!("Failed to write dockerfile contents to {full_path:?}")
    })?;
    Ok(Log::simple(
      "Write dockerfile to host",
      format!("dockerfile contents written to {full_path:?}"),
    ))
  }
}

impl Resolve<super::Args> for build::Build {
  #[instrument(name = "Build", skip_all, fields(build = self.build.name.to_string()))]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<Log>> {
    let build::Build {
      build,
      repo: linked_repo,
      registry_token,
      additional_tags,
      replacers: mut core_replacers,
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
          repo,
          files_on_host,
          dockerfile,
          pre_build,
          ..
        },
      ..
    } = &build;

    if !*files_on_host
      && repo.is_empty()
      && linked_repo.is_none()
      && dockerfile.is_empty()
    {
      return Err(anyhow!("Build must be files on host mode, have a repo attached, or have dockerfile contents set to build").into());
    }

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

    let build_path = if let Some(repo) = &linked_repo {
      periphery_config()
        .repo_dir()
        .join(to_path_compatible_name(&repo.name))
        .join(build_path)
    } else {
      periphery_config()
        .build_dir()
        .join(to_path_compatible_name(name))
        .join(build_path)
    }
    .components()
    .collect::<PathBuf>();

    let dockerfile_path = optional_string(dockerfile_path)
      .unwrap_or("Dockerfile".to_owned());

    // Write UI defined Dockerfile to host
    if !*files_on_host
      && repo.is_empty()
      && linked_repo.is_none()
      && !dockerfile.is_empty()
    {
      let dockerfile = if *skip_secret_interp {
        dockerfile.to_string()
      } else {
        let (dockerfile, replacers) = svi::interpolate_variables(
          dockerfile,
          &periphery_config().secrets,
          svi::Interpolator::DoubleBrackets,
          true,
        ).context("Failed to interpolate variables into UI defined dockerfile")?;
        core_replacers.extend(replacers);
        dockerfile
      };

      let full_dockerfile_path = build_path
        .join(&dockerfile_path)
        .components()
        .collect::<PathBuf>();

      // Ensure parent directory exists
      if let Some(parent) = full_dockerfile_path.parent() {
        if !parent.exists() {
          tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to initialize dockerfile parent directory {parent:?}"))?;
        }
      }

      fs::write(&full_dockerfile_path, dockerfile).await.with_context(|| {
        format!(
          "Failed to write dockerfile contents to {full_dockerfile_path:?}"
        )
      })?;

      logs.push(Log::simple(
        "Write Dockerfile",
        format!(
          "Dockerfile contents written to {full_dockerfile_path:?}"
        ),
      ));
    };

    // Pre Build
    if !pre_build.is_none() {
      let pre_build_path = build_path.join(&pre_build.path);
      if let Some(log) = if !skip_secret_interp {
        run_komodo_command_with_interpolation(
          "Pre Build",
          Some(pre_build_path.as_path()),
          &pre_build.command,
          true,
          &periphery_config().secrets,
          &core_replacers,
        )
        .await
      } else {
        run_komodo_command_multiline(
          "Pre Build",
          Some(pre_build_path.as_path()),
          &pre_build.command,
        )
        .await
      } {
        let success = log.success;
        logs.push(log);
        if !success {
          return Ok(logs);
        }
      };
    }

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
      &build_path,
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
        build_path.as_ref(),
        command,
      )
      .await;
      logs.push(build_log);
    } else if let Some(log) = run_komodo_command_with_interpolation(
      "Docker Build",
      build_path.as_ref(),
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
