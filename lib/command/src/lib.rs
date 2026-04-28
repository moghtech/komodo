use std::{
  path::{Path, PathBuf},
  process::Stdio,
  sync::OnceLock,
};

use komodo_client::{
  entities::{komodo_timestamp, update::Log},
  parsers::parse_multiline_command,
};
use tokio::io::AsyncWriteExt;

mod output;

pub use output::*;
use tokio::process::Command;

/// Commands are run directly, and cannot include '&&'
pub async fn run_komodo_standard_command(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl Into<String>,
) -> Log {
  let command = command.into();
  let start_ts = komodo_timestamp();
  let output = run_standard_command(&command, path).await;
  output_into_log(stage, command, start_ts, output)
}

/// Commands are run directly with args passed separately.
pub async fn run_komodo_command_args(
  stage: &str,
  path: impl Into<Option<&Path>>,
  program: &str,
  args: impl IntoIterator<Item = impl AsRef<str>>,
) -> Log {
  let args = args
    .into_iter()
    .map(|arg| arg.as_ref().to_string())
    .collect::<Vec<_>>();
  let command = command_string(program, &args);
  let start_ts = komodo_timestamp();
  let output = run_command_args(program, &args, path, None).await;
  output_into_log(stage, command, start_ts, output)
}

/// Commands are run directly with args passed separately, and stdin piped.
pub async fn run_komodo_command_args_with_stdin(
  stage: &str,
  path: impl Into<Option<&Path>>,
  program: &str,
  args: impl IntoIterator<Item = impl AsRef<str>>,
  stdin: impl AsRef<[u8]>,
) -> Log {
  let args = args
    .into_iter()
    .map(|arg| arg.as_ref().to_string())
    .collect::<Vec<_>>();
  let command = command_string(program, &args);
  let start_ts = komodo_timestamp();
  let output =
    run_command_args(program, &args, path, Some(stdin.as_ref()))
      .await;
  output_into_log(stage, command, start_ts, output)
}

/// Commands are wrapped in 'sh -c', and can include '&&'
pub async fn run_komodo_shell_command(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl Into<String>,
) -> Log {
  let command = command.into();
  let start_ts = komodo_timestamp();
  let output = run_shell_command(&command, path).await;
  output_into_log(stage, command, start_ts, output)
}

/// Parses commands out of multiline string
/// and chains them together with '&&'.
/// Supports full line and end of line comments.
/// See [parse_multiline_command].
///
/// The result may be None if the command is empty after parsing,
/// ie if all the lines are commented out.
pub async fn run_komodo_multiline_command(
  stage: &str,
  path: impl Into<Option<&Path>>,
  command: impl AsRef<str>,
) -> Option<Log> {
  let command = parse_multiline_command(command);
  if command.is_empty() {
    return None;
  }
  Some(run_komodo_shell_command(stage, path, command).await)
}

pub enum KomodoCommandMode {
  Standard,
  Shell,
  Multiline,
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
  mode: KomodoCommandMode,
  replacers: &[(String, String)],
) -> Option<Log> {
  let mut log = match mode {
    KomodoCommandMode::Standard => run_komodo_standard_command(
      stage,
      path,
      command.as_ref().to_string(),
    )
    .await
    .into(),
    KomodoCommandMode::Shell => run_komodo_shell_command(
      stage,
      path,
      command.as_ref().to_string(),
    )
    .await
    .into(),
    KomodoCommandMode::Multiline => {
      run_komodo_multiline_command(stage, path, command).await
    }
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

/// Commands are run directly, and cannot include '&&'
pub async fn run_standard_command(
  command: &str,
  path: impl Into<Option<&Path>>,
) -> CommandOutput {
  let lexed = if let Some(lexed) = shlex::split(command)
    && !lexed.is_empty()
  {
    lexed
  } else {
    return CommandOutput::from_err(std::io::Error::other(
      "Command lexed into empty args",
    ));
  };

  let mut cmd = Command::new(&lexed[0]);

  cmd
    .args(&lexed[1..])
    .kill_on_drop(true)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  if let Some(path) = path.into() {
    match path.canonicalize() {
      Ok(path) => {
        cmd.current_dir(path);
      }
      Err(e) => return CommandOutput::from_err(e),
    }
  }

  CommandOutput::from(cmd.output().await)
}

/// Commands are run directly with args passed separately.
pub async fn run_command_args(
  program: &str,
  args: &[String],
  path: impl Into<Option<&Path>>,
  stdin: Option<&[u8]>,
) -> CommandOutput {
  let mut cmd = Command::new(program);

  cmd
    .args(args)
    .kill_on_drop(true)
    .stdin(if stdin.is_some() {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  if let Some(path) = path.into() {
    match path.canonicalize() {
      Ok(path) => {
        cmd.current_dir(path);
      }
      Err(e) => return CommandOutput::from_err(e),
    }
  }

  let mut child = match cmd.spawn() {
    Ok(child) => child,
    Err(e) => return CommandOutput::from_err(e),
  };

  if let Some(stdin) = stdin {
    match child.stdin.take() {
      Some(mut child_stdin) => {
        if let Err(e) = child_stdin.write_all(stdin).await {
          return CommandOutput::from_err(e);
        }
      }
      None => {
        return CommandOutput::from_err(std::io::Error::other(
          "Failed to open command stdin",
        ));
      }
    }
  }

  CommandOutput::from(child.wait_with_output().await)
}

pub fn command_string(program: &str, args: &[String]) -> String {
  std::iter::once(program.to_string())
    .chain(args.iter().map(|arg| {
      shlex::try_quote(arg)
        .map(|arg| arg.into_owned())
        .unwrap_or_else(|_| arg.clone())
    }))
    .collect::<Vec<_>>()
    .join(" ")
}

fn shell() -> &'static str {
  static DEFAULT_SHELL: OnceLock<String> = OnceLock::new();
  DEFAULT_SHELL.get_or_init(|| {
    if PathBuf::from("/bin/bash").exists() {
      String::from("/bin/bash")
    } else if PathBuf::from("/usr/bin/bash").exists() {
      String::from("/usr/bin/bash")
    } else {
      String::from("/bin/sh")
    }
  })
}

/// Commands are wrapped in 'sh -c', and can include '&&'
pub async fn run_shell_command(
  command: &str,
  path: impl Into<Option<&Path>>,
) -> CommandOutput {
  let mut cmd = Command::new(shell());

  cmd
    .args(["-c", command])
    .kill_on_drop(true)
    .stdin(Stdio::null());

  if let Some(path) = path.into() {
    match path.canonicalize() {
      Ok(path) => {
        cmd.current_dir(path);
      }
      Err(e) => return CommandOutput::from_err(e),
    }
  }

  CommandOutput::from(cmd.output().await)
}
