use std::path::PathBuf;

use anyhow::{Context, anyhow};
use command::{
  run_komodo_command, run_komodo_command_with_sanitization,
};
use formatting::format_serror;
use interpolate::Interpolator;
use komodo_client::entities::{
  EnvironmentVar, all_logs_success,
  build::{Build, BuildConfig},
  environment_vars_from_str, get_image_name, optional_string,
  to_path_compatible_name,
  update::Log,
};
use periphery_client::api::build::{
  self, GetDockerfileContentsOnHost,
  GetDockerfileContentsOnHostResponse, PruneBuilders, PruneBuildx,
  WriteDockerfileContentsToHost,
};
use resolver_api::Resolve;
use tokio::fs;

use crate::{
  build::{
    image_tags, parse_build_args, parse_secret_args, write_dockerfile,
  },
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
      mut build,
      repo: linked_repo,
      registry_token,
      additional_tags,
      mut replacers,
    } = self;

    let mut logs = Vec::new();

    // Periphery side interpolation
    let mut interpolator =
      Interpolator::new(None, &periphery_config().secrets);
    interpolator
      .interpolate_build(&mut build)?
      .push_logs(&mut logs);

    replacers.extend(interpolator.secret_replacers);

    let Build {
      name,
      config:
        BuildConfig {
          version,
          image_tag,
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
      write_dockerfile(
        &build_path,
        &dockerfile_path,
        dockerfile,
        &mut logs,
      )
      .await;
      if !all_logs_success(&logs) {
        return Ok(logs);
      }
    };

    // Pre Build
    if !pre_build.is_none() {
      let pre_build_path = build_path.join(&pre_build.path);
      if let Some(log) = run_komodo_command_with_sanitization(
        "Pre Build",
        pre_build_path.as_path(),
        &pre_build.command,
        true,
        &replacers,
      )
      .await
      {
        let success = log.success;
        logs.push(log);
        if !success {
          return Ok(logs);
        }
      }
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
    let command_secret_args =
      parse_secret_args(&secret_args, &build_path).await?;

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

    if let Some(build_log) = run_komodo_command_with_sanitization(
      "Docker Build",
      build_path.as_ref(),
      command,
      false,
      &replacers,
    )
    .await
    {
      logs.push(build_log);
    };

    Ok(logs)
  }
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
