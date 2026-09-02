use std::{
  io,
  os::unix::process::ExitStatusExt,
  process::{ExitStatus, Output},
};

#[derive(Debug, Clone)]
pub struct CommandOutput {
  pub status: ExitStatus,
  pub stdout: String,
  pub stderr: String,
}

impl CommandOutput {
  pub fn from(output: io::Result<Output>) -> Self {
    match output {
      Ok(output) => Self {
        status: output.status,
        stdout: String::from_utf8(output.stdout)
          .unwrap_or("failed to generate stdout".to_string()),
        stderr: String::from_utf8(output.stderr)
          .unwrap_or("failed to generate stderr".to_string()),
      },
      Err(e) => CommandOutput::from_err(e),
    }
  }

  pub fn from_err(e: io::Error) -> Self {
    Self {
      status: ExitStatus::from_raw(1),
      stdout: "".to_string(),
      stderr: format!("{e:?}"),
    }
  }

  pub fn from_err_message(e: String) -> Self {
    Self {
      status: ExitStatus::from_raw(1),
      stdout: "".to_string(),
      stderr: e,
    }
  }

  /// Keeps what the command printed before it was killed, and reports
  /// the reason it was killed on stderr.
  pub fn from_killed(output: Output, reason: String) -> Self {
    let mut stderr = tail(
      String::from_utf8(output.stderr)
        .unwrap_or("failed to generate stderr".to_string()),
    );
    if !stderr.is_empty() && !stderr.ends_with('\n') {
      stderr.push('\n');
    }
    stderr.push_str(&reason);
    Self {
      // Keeps the real wait status, but never reports success: the
      // child may have exited 0 between the deadline firing and the
      // signal landing.
      status: if output.status.success() {
        ExitStatus::from_raw(1)
      } else {
        output.status
      },
      stdout: tail(
        String::from_utf8(output.stdout)
          .unwrap_or("failed to generate stdout".to_string()),
      ),
      stderr,
    }
  }

  pub fn success(&self) -> bool {
    self.status.success()
  }
}

/// A command killed after printing without pause has no natural bound on
/// its output, so cap what is kept from it. The tail is the useful end:
/// it is where the command got to before it was killed.
const MAX_KILLED_OUTPUT: usize = 1024 * 1024;

fn tail(output: String) -> String {
  if output.len() <= MAX_KILLED_OUTPUT {
    return output;
  }
  let mut start = output.len() - MAX_KILLED_OUTPUT;
  while !output.is_char_boundary(start) {
    start += 1;
  }
  format!("[earlier output truncated]\n{}", &output[start..])
}
