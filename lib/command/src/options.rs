use std::{path::Path, time::Duration};

use tokio_util::sync::CancellationToken;

/// Controls for how a command is executed.
///
/// When either timeout or cancel is set, the child is spawned in its own process group so
/// that, on timeout or cancellation, the entire group (the command and any
/// descendants it spawned) is killed together — not just the direct child.
#[derive(Default, Clone)]
pub struct CommandOptions<'a> {
  /// Run the command at a particular path
  pub path: Option<&'a Path>,
  /// Kill the command (and its process group) if this duration elapses
  /// before it finishes.
  pub timeout: Option<Duration>,
  /// Kill the command (and its process group) when this token is
  /// cancelled, allowing cancellation from elsewhere.
  pub cancel: Option<CancellationToken>,
  /// Write this to the command's standard input. Without it,
  /// the command gets no input at all.
  ///
  /// Use this to hand a secret to a command which reads it from stdin,
  /// such as `docker login --password-stdin`. Unlike interpolating the
  /// secret into the command, it never appears in the process arguments,
  /// where any user on the host could read it out of `ps`.
  pub stdin: Option<&'a str>,
}

impl<'a> CommandOptions<'a> {
  pub fn path(mut self, path: impl Into<Option<&'a Path>>) -> Self {
    self.path = path.into();
    self
  }

  pub fn stdin(mut self, stdin: impl Into<Option<&'a str>>) -> Self {
    self.stdin = stdin.into();
    self
  }
}

impl CommandOptions<'_> {
  pub fn timeout(
    mut self,
    timeout: impl Into<Option<Duration>>,
  ) -> Self {
    self.timeout = timeout.into();
    self
  }

  pub fn cancel(
    mut self,
    cancel: impl Into<Option<CancellationToken>>,
  ) -> Self {
    self.cancel = cancel.into();
    self
  }
}
