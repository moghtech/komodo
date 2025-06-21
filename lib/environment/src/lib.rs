use std::path::{Path, PathBuf};

use anyhow::Context;
use formatting::format_serror;
use komodo_client::entities::{EnvironmentVar, update::Log};

/// If the environment was written and needs to be passed to the compose command,
/// will return the env file PathBuf.
/// Should ensure all logs are successful after calling.
pub async fn write_env_file(
  environment: &[EnvironmentVar],
  folder: &Path,
  env_file_path: &str,
  logs: &mut Vec<Log>,
) -> Option<PathBuf> {
  let env_file_path =
    folder.join(env_file_path).components().collect::<PathBuf>();

  if environment.is_empty() {
    // Still want to return Some(env_file_path) if the path
    // already exists on the host and is a file.
    // This is for "Files on Server" mode when user writes the env file themself.
    if env_file_path.is_file() {
      return Some(env_file_path);
    }
    return None;
  }

  let contents = environment
    .iter()
    .map(|env| format!("{}={}", env.variable, env.value))
    .collect::<Vec<_>>()
    .join("\n");

  if let Some(parent) = env_file_path.parent() {
    if let Err(e) = tokio::fs::create_dir_all(parent)
      .await
      .with_context(|| format!("Failed to initialize environment file parent directory {parent:?}"))
    {
      logs.push(Log::error(
        "Write Environment File",
        format_serror(&e.into()),
      ));
      return None;
    }
  }

  if let Err(e) = tokio::fs::write(&env_file_path, contents)
    .await
    .with_context(|| {
      format!("Failed to write environment file to {env_file_path:?}")
    })
  {
    logs.push(Log::error(
      "Write Environment File",
      format_serror(&e.into()),
    ));
    return None;
  }

  logs.push(Log::simple(
    "Write Environment File",
    format!("Environment file written to {env_file_path:?}"),
  ));

  Some(env_file_path)
}
