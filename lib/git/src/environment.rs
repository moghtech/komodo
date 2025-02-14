use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use anyhow::Context;
use formatting::format_serror;
use komodo_client::entities::{update::Log, EnvironmentVar};

/// If the environment was written and needs to be passed to the compose command,
/// will return the env file PathBuf.
/// If variables were interpolated, will also return the sanitizing replacers.
pub async fn write_file(
  environment: &[EnvironmentVar],
  env_file_path: &str,
  secrets: Option<&HashMap<String, String>>,
  folder: &Path,
  logs: &mut Vec<Log>,
) -> Result<(Option<PathBuf>, Option<Vec<(String, String)>>), ()> {
  let env_file_path = folder.join(env_file_path);

  if environment.is_empty() {
    // Still want to return Some(env_file_path) if the path
    // already exists on the host and is a file.
    // This is for "Files on Server" mode when user writes the env file themself.
    if env_file_path.is_file() {
      return Ok((Some(env_file_path), None));
    }
    return Ok((None, None));
  }

  let contents = environment
    .iter()
    .map(|env| format!("{}={}", env.variable, env.value))
    .collect::<Vec<_>>()
    .join("\n");

  let (contents, replacers) = if let Some(secrets) = secrets {
    let res = svi::interpolate_variables(
      &contents,
      secrets,
      svi::Interpolator::DoubleBrackets,
      true,
    )
    .context("failed to interpolate secrets into environment");

    let (contents, replacers) = match res {
      Ok(res) => res,
      Err(e) => {
        logs.push(Log::error(
          "Interpolate - Environment",
          format_serror(&e.into()),
        ));
        return Err(());
      }
    };

    if !replacers.is_empty() {
      logs.push(Log::simple(
        "Interpolate - Environment",
        replacers
            .iter()
            .map(|(_, variable)| format!("<span class=\"text-muted-foreground\">replaced:</span> {variable}"))
            .collect::<Vec<_>>()
            .join("\n"),
      ))
    }

    (contents, Some(replacers))
  } else {
    (contents, None)
  };

  if let Err(e) = tokio::fs::write(&env_file_path, contents)
    .await
    .with_context(|| {
      format!("failed to write environment file to {env_file_path:?}")
    })
  {
    logs.push(Log::error(
      "write environment file",
      format_serror(&e.into()),
    ));
    return Err(());
  }

  logs.push(Log::simple(
    "write environment file",
    format!("environment written to {env_file_path:?}"),
  ));

  Ok((Some(env_file_path), replacers))
}

///
/// Will return the env file PathBuf.
pub async fn write_file_simple(
  environment: &[EnvironmentVar],
  env_file_path: &str,
  folder: &Path,
  logs: &mut Vec<Log>,
) -> anyhow::Result<Option<PathBuf>> {
  let env_file_path = folder.join(env_file_path);

  if environment.is_empty() {
    // Still want to return Some(env_file_path) if the path
    // already exists on the host and is a file.
    // This is for "Files on Server" mode when user writes the env file themself.
    if env_file_path.is_file() {
      return Ok(Some(env_file_path));
    }
    return Ok(None);
  }

  let contents = environment
    .iter()
    .map(|env| format!("{}={}", env.variable, env.value))
    .collect::<Vec<_>>()
    .join("\n");

  if let Err(e) = tokio::fs::write(&env_file_path, contents)
    .await
    .with_context(|| {
      format!("Failed to write environment file to {env_file_path:?}")
    })
  {
    logs.push(Log::error(
      "Write Environment file",
      format_serror(&(&e).into()),
    ));
    return Err(e);
  }

  logs.push(Log::simple(
    "Write Environment File",
    format!("Environment written to {env_file_path:?}"),
  ));

  Ok(Some(env_file_path))
}
