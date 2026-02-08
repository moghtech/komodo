use std::path::Path;

use komodo_client::{
  entities::{komodo_timestamp, update::Log},
  parsers::parse_multiline_command,
};
use regex::Regex;
use run_command::{CommandOutput, async_run_command};

/// A smarter replacement function that only replaces variable values
/// when they appear as whole "tokens" — not as substrings of longer
/// alphanumeric words. This prevents partial replacements in stack
/// names, service names, paths, etc.
///
/// For example, if ESPHOME_USER=esp, this will NOT replace "esp" inside
/// "esphome" but WILL replace a standalone "esp" (e.g. after a space,
/// equals sign, or at string boundaries).
///
/// For values that start/end with word characters (\w), word boundaries
/// (\b) are used. For values starting/ending with non-word characters
/// (like paths), simple string replacement is used as a fallback since
/// partial-match issues are unlikely for such values.
pub fn replace_in_string_word_boundary<'a>(
  input: &str,
  replacers: impl IntoIterator<Item = &'a (String, String)>,
) -> String {
  let mut result = input.to_string();

  for (to_replace, replacer) in replacers {
    if to_replace.is_empty() {
      continue;
    }

    let escaped = regex::escape(to_replace);

    // Determine if we need word boundaries on each side.
    // \b only works at transitions between \w and \W characters.
    let first_is_word = to_replace
      .chars()
      .next()
      .map_or(false, |c| c.is_alphanumeric() || c == '_');
    let last_is_word = to_replace
      .chars()
      .last()
      .map_or(false, |c| c.is_alphanumeric() || c == '_');

    let pattern = format!(
      "{}{}{}",
      if first_is_word { r"\b" } else { "" },
      escaped,
      if last_is_word { r"\b" } else { "" },
    );

    if let Ok(re) = Regex::new(&pattern) {
      let replacement_text = format!("<{}>", replacer);
      result =
        re.replace_all(&result, replacement_text.as_str()).to_string();
    } else {
      // Fall back to simple replacement if regex fails
      result = result.replace(to_replace, &format!("<{replacer}>"));
    }
  }

  result
}

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
  log.command = replace_in_string_word_boundary(&log.command, replacers);
  log.stdout = replace_in_string_word_boundary(&log.stdout, replacers);
  log.stderr = replace_in_string_word_boundary(&log.stderr, replacers);

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
