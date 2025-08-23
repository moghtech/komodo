use std::{
  fmt::Write,
  path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::{
  entities::{EnvironmentVar, Version, update::Log},
  parsers::QUOTE_PATTERN,
};

pub async fn write_dockerfile(
  build_path: &Path,
  dockerfile_path: &str,
  dockerfile: &str,
  logs: &mut Vec<Log>,
) {
  if let Err(e) = async {
    if dockerfile.is_empty() {
      return Err(anyhow!("UI Defined dockerfile is empty"));
    }

    let full_dockerfile_path = build_path
      .join(dockerfile_path)
      .components()
      .collect::<PathBuf>();

    // Ensure parent directory exists
    if let Some(parent) = full_dockerfile_path.parent() && !parent.exists() {
      tokio::fs::create_dir_all(parent)
        .await
        .with_context(|| format!("Failed to initialize dockerfile parent directory {parent:?}"))?;
    }

    tokio::fs::write(&full_dockerfile_path, dockerfile).await.with_context(|| {
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

    anyhow::Ok(())
  }.await {
    logs.push(Log::error("Write Dockerfile", format_serror(&e.into())));
  }
}

pub fn image_tags(
  image_names: &[String],
  custom_tag: &str,
  version: &Version,
  additional: &[String],
) -> anyhow::Result<String> {
  let Version { major, minor, .. } = version;
  let custom_tag = if custom_tag.is_empty() {
    String::new()
  } else {
    format!("-{custom_tag}")
  };

  let mut res = String::new();

  for image_name in image_names {
    write!(
      &mut res,
      " -t {image_name}:latest{custom_tag} -t {image_name}:{version}{custom_tag} -t {image_name}:{major}.{minor}{custom_tag} -t {image_name}:{major}{custom_tag}"
    )?;
    for tag in additional {
      write!(&mut res, " -t {image_name}:{tag}{custom_tag}")?;
    }
  }

  Ok(res)
}

pub fn parse_build_args(build_args: &[EnvironmentVar]) -> String {
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
pub async fn parse_secret_args(
  secret_args: &[EnvironmentVar],
  build_dir: &Path,
) -> anyhow::Result<String> {
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
