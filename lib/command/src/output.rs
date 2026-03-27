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
      stderr: format!("{e:#?}"),
    }
  }

  pub fn success(&self) -> bool {
    self.status.success()
  }
}
