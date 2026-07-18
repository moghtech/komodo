use std::{path::PathBuf, process::Stdio, sync::OnceLock};

use komodo_client::{
  entities::{komodo_timestamp, update::Log},
  parsers::parse_multiline_command,
};
use tokio::process::Command;

mod options;
mod output;

pub use options::*;
pub use output::*;

/// Commands are run directly, and cannot include '&&'
pub async fn run_komodo_standard_command(
  stage: &str,
  command: impl Into<String>,
  options: CommandOptions<'_>,
) -> Log {
  let command = command.into();
  let start_ts = komodo_timestamp();
  let output = run_standard_command(&command, options).await;
  output_into_log(stage, command, start_ts, output)
}

/// Commands are wrapped in 'sh -c', and can include '&&'
pub async fn run_komodo_shell_command(
  stage: &str,
  command: impl Into<String>,
  options: CommandOptions<'_>,
) -> Log {
  let command = command.into();
  let start_ts = komodo_timestamp();
  let output = run_shell_command(&command, options).await;
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
  command: impl AsRef<str>,
  options: CommandOptions<'_>,
) -> Option<Log> {
  let command = parse_multiline_command(command);
  if command.is_empty() {
    return None;
  }
  Some(run_komodo_shell_command(stage, command, options).await)
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
  command: impl AsRef<str>,
  options: CommandOptions<'_>,
  mode: KomodoCommandMode,
  replacers: &[(String, String)],
) -> Option<Log> {
  let mut log = match mode {
    KomodoCommandMode::Standard => {
      run_komodo_standard_command(stage, command.as_ref(), options)
        .await
        .into()
    }
    KomodoCommandMode::Shell => {
      run_komodo_shell_command(stage, command.as_ref(), options)
        .await
        .into()
    }
    KomodoCommandMode::Multiline => {
      run_komodo_multiline_command(stage, command, options).await
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

/// Commands are run directly, and cannot include '&&'.
///
/// See [CommandOptions] for the available `timeout` / `cancel` controls.
pub async fn run_standard_command(
  command: &str,
  options: CommandOptions<'_>,
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

  run_command(cmd, options).await
}

fn shell() -> &'static str {
  static DEFAULT_SHELL: OnceLock<String> = OnceLock::new();
  DEFAULT_SHELL.get_or_init(|| {
    if PathBuf::from("/bin/bash").exists() {
      String::from("/bin/bash")
    } else if PathBuf::from("/usr/bin/bash").exists() {
      String::from("/usr/bin/bash")
    } else if PathBuf::from("/bin/sh").exists() {
      String::from("/bin/sh")
    } else if PathBuf::from("/usr/bin/sh").exists() {
      String::from("/usr/bin/sh")
    } else {
      // try to use sh wherever it is on host by name.
      String::from("sh")
    }
  })
}

/// Commands are wrapped in 'sh -c', and can include '&&'.
///
/// See [CommandOptions] for the available `timeout` / `cancel` controls.
pub async fn run_shell_command(
  command: &str,
  options: CommandOptions<'_>,
) -> CommandOutput {
  let mut cmd = Command::new(shell());

  cmd
    .args(["-c", command])
    .kill_on_drop(true)
    .stdin(Stdio::null());

  run_command(cmd, options).await
}

/// Runs the command to completion, returning its output.
///
/// With an empty `options`, this is just `cmd.output()`.
///
/// With a `timeout` and/or `cancel` token, the child is spawned in its own
/// process group (via `process_group(0)`, so the child's pid is also its
/// process group id). If the timeout elapses or the token is cancelled
/// before the command finishes, the entire process group is killed with
/// `SIGKILL` — not just the direct child — so any descendants the command
/// spawned (e.g. processes started by a `sh -c` wrapper) are torn down too.
/// `kill_on_drop(true)` remains set as a backstop to reap the direct child.
async fn run_command(
  mut cmd: Command,
  options: CommandOptions<'_>,
) -> CommandOutput {
  let CommandOptions {
    path,
    timeout,
    cancel,
  } = options;

  // Attach the path to cmd as current dir
  if let Some(path) = path {
    match path.canonicalize() {
      Ok(path) => {
        cmd.current_dir(path);
      }
      Err(e) => return CommandOutput::from_err(e),
    }
  }

  // Fast path: nothing to wait on besides the command itself.
  if timeout.is_none() && cancel.is_none() {
    return CommandOutput::from(cmd.output().await);
  }

  // Place the child in a new process group so the whole group can be
  // signalled together. `output()` configures stdout/stderr automatically,
  // but since we spawn manually we set them here too.
  cmd
    .process_group(0)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  let child = match cmd.spawn() {
    Ok(child) => child,
    Err(e) => return CommandOutput::from_err(e),
  };

  // Because of `process_group(0)`, the child's pid equals its pgid.
  let pid = child.id();

  // Resolve to `()` only when the relevant control fires; otherwise stay
  // pending forever so it never wins the `select!`.
  let on_timeout = async {
    match timeout {
      Some(timeout) => tokio::time::sleep(timeout).await,
      None => std::future::pending().await,
    }
  };
  let on_cancel = async {
    match &cancel {
      Some(cancel) => cancel.cancelled().await,
      None => std::future::pending().await,
    }
  };

  let killed_reason = tokio::select! {
    output = child.wait_with_output() => {
      return CommandOutput::from(output);
    }
    _ = on_timeout => format!(
      "Command timed out after {:.1}s (process group killed)",
      // `timeout` is `Some` here, since `on_timeout` only fires when set.
      timeout.map(|t| t.as_secs_f64()).unwrap_or_default(),
    ),
    _ = on_cancel => {
      "Command cancelled (process group killed)".to_string()
    }
  };

  kill_process_group(pid);
  CommandOutput::from_err_message(killed_reason)
}

/// Sends `SIGKILL` to the entire process group led by `pid`.
///
/// A negative pid targets the whole group, so a child spawned with
/// `process_group(0)` (group leader) is killed along with all of its
/// descendants.
fn kill_process_group(pid: Option<u32>) {
  let Some(pid) = pid else {
    return;
  };
  // SAFETY: `kill` is a simple syscall with no memory safety concerns;
  // we only signal our own child's process group.
  unsafe {
    libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use tokio_util::sync::CancellationToken;

  use super::*;

  /// On timeout, a backgrounded grandchild (here `sleep 31337`, started
  /// with `&` so it is not the direct child of our spawned shell) must be
  /// killed along with the rest of the process group.
  #[tokio::test]
  async fn timeout_kills_process_group() {
    // Unique sleep duration so we can pgrep for exactly this process.
    let marker = "sleep 31337";
    let out = run_shell_command(
      &format!("{marker} & sleep 31336"),
      CommandOptions::default().timeout(Duration::from_millis(300)),
    )
    .await;

    // The command should have reported a timeout failure.
    assert!(!out.success(), "expected timeout failure, got: {out:?}");
    assert!(
      out.stderr.contains("timed out"),
      "expected timeout error, got: {out:?}"
    );

    // Give the kernel a moment to reap the killed group.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let still_running = std::process::Command::new("pgrep")
      .args(["-f", marker])
      .output()
      .expect("pgrep should run");
    let pids = String::from_utf8_lossy(&still_running.stdout);
    let pids = pids.trim();
    assert!(
      pids.is_empty(),
      "backgrounded grandchild survived timeout: pids={pids:?}"
    );
  }

  /// Cancelling the token mid-run must kill the whole process group, just
  /// like a timeout does, and return promptly with a cancellation error.
  #[tokio::test]
  async fn cancel_kills_process_group() {
    let marker = "sleep 41337";
    let cancel = CancellationToken::new();

    // Cancel shortly after the command starts.
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_millis(300)).await;
      cancel_clone.cancel();
    });

    let out = run_shell_command(
      &format!("{marker} & sleep 41336"),
      CommandOptions::default().cancel(cancel),
    )
    .await;

    assert!(!out.success(), "expected cancel failure, got: {out:?}");
    assert!(
      out.stderr.contains("cancelled"),
      "expected cancellation error, got: {out:?}"
    );

    tokio::time::sleep(Duration::from_millis(200)).await;

    let still_running = std::process::Command::new("pgrep")
      .args(["-f", marker])
      .output()
      .expect("pgrep should run");
    let pids = String::from_utf8_lossy(&still_running.stdout);
    let pids = pids.trim();
    assert!(
      pids.is_empty(),
      "backgrounded grandchild survived cancellation: pids={pids:?}"
    );
  }
}
