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

/// SSL-related error patterns that git may produce when the server's
/// certificate is not trusted.
const SSL_ERROR_PATTERNS: &[&str] = &[
  "SSL certificate problem",
  "server certificate verification failed",
  "unable to access",
];

const SSL_HELP_MESSAGE: &str = concat!(
  "\n\nNote: This error may be caused by an untrusted SSL certificate on the git server. ",
  "To resolve this, either:\n",
  "  1. Add the server's CA certificate to the system trust store, or\n",
  "  2. Set the GIT_SSL_CAINFO environment variable to point to your CA bundle, or\n",
  "  3. Set GIT_SSL_NO_VERIFY=true (not recommended for production) to skip certificate verification.",
);

/// If the stderr contains signs of an SSL/TLS certificate error,
/// append a human-friendly help message.
fn maybe_enrich_ssl_error(stderr: String) -> String {
  let lower = stderr.to_lowercase();
  let is_ssl_error = SSL_ERROR_PATTERNS
    .iter()
    .any(|p| lower.contains(&p.to_lowercase()));
  if is_ssl_error {
    format!("{stderr}{SSL_HELP_MESSAGE}")
  } else {
    stderr
  }
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
    stderr: maybe_enrich_ssl_error(output.stderr),
    command,
    success,
    start_ts,
    end_ts: komodo_timestamp(),
  }
}
