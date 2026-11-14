use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use formatting::format_serror;
use komodo_client::{
  entities::{EnvironmentVar, update::Log},
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

    mogh_secret_file::write_async(&full_dockerfile_path, dockerfile).await.with_context(|| {
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
