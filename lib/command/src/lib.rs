use std::path::Path;

use komodo_client::{
  entities::{komodo_timestamp, update::Log},
  parsers::parse_multiline_command,
};
use run_command::{CommandOutput, async_run_command};

pub async fn run_komodo_command(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl AsRef<str>,
) -> Log {
  let command = if let Some(path) = path.into() {
    format!("cd {} && {}", path.display(), command.as_ref())
  } else {
    command.as_ref().to_string()
  };
  let start_ts = komodo_timestamp();
  let output = async_run_command(&command).await;
  output_into_log(stage, command, start_ts, output)
}

/// Parses commands out of multiline string
/// and chains them together with '&&'.
/// Supports full line and end of line comments.
/// See [parse_multiline_command].
///
/// The result may be None if the command is empty after parsing,
/// ie if all the lines are commented out.
pub async fn run_komodo_command_multiline(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl AsRef<str>,
) -> Option<Log> {
  let command = parse_multiline_command(command);
  if command.is_empty() {
    return None;
  }
  Some(run_komodo_command(stage, path, command).await)
}

/// Executes the command, and sanitizes the output to avoid exposing secrets in the log.
///
/// Checks to make sure the command is non-empty after being multiline-parsed.
///
/// If `parse_multiline: true`, parses commands out of multiline string
/// and chains them together with '&&'.
/// Supports full line and end of line comments.
/// See [parse_multiline_command].
pub async fn run_komodo_command_with_sanitization(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl AsRef<str>,
  parse_multiline: bool,
  replacers: &[(String, String)],
) -> Option<Log> {
  let mut log = if parse_multiline {
    run_komodo_command_multiline(stage, path, command).await
  } else {
    run_komodo_command(stage, path, command).await.into()
  }?;

  // Sanitize the command and output
  log.command = svi::replace_in_string(&log.command, replacers);
  log.stdout = svi::replace_in_string(&log.stdout, replacers);
  log.stderr = svi::replace_in_string(&log.stderr, replacers);

  Some(log)
}

pub fn output_into_log(
  stage: &str,
  command: String,
  start_ts: i64,
  output: CommandOutput,
) -> Log {
  let success = output.success();
  Log {
    stage: stage.to_string(),
    stdout: output.stdout,
    stderr: output.stderr,
    command,
    success,
    start_ts,
    end_ts: komodo_timestamp(),
  }
}
