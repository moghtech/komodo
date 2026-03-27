use std::{collections::VecDeque, sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use encoding::{Decode as _, WithChannel};
use komodo_client::entities::{
  komodo_timestamp,
  terminal::{
    Terminal, TerminalMessage, TerminalRecreateMode,
    TerminalStdinMessage, TerminalTarget,
  },
};
use periphery_client::transport::EncodedTerminalMessage;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
  config::periphery_config,
  state::{terminal_channels, terminal_triggers, terminals},
};

pub async fn handle_message(message: EncodedTerminalMessage) {
  let WithChannel {
    channel: channel_id,
    data,
  } = match message.decode() {
    Ok(res) => res,
    Err(e) => {
      warn!(
        "Received invalid Terminal bytes | Channel decode | {e:#}"
      );
      return;
    }
  };

  let data = match data {
    Ok(data) => data,
    Err(_) => {
      // This means Core should disconnect.
      terminal_channels().remove(&channel_id).await;
      return;
    }
  };

  let message = match TerminalMessage::from_raw(data)
    .into_stdin_message()
  {
    Err(e) => {
      warn!(
        "Received invalid Terminal bytes | TerminalMessage decode | {e:#}"
      );
      return;
    }
    // Send 'begin' trigger for Terminal Executions
    Ok(TerminalStdinMessage::Begin) => {
      if let Err(e) = terminal_triggers().send(&channel_id).await {
        warn!("{e:#}")
      }
      return;
    }
    Ok(message) => message,
  };

  let Some(channel) = terminal_channels().get(&channel_id).await
  else {
    warn!("No terminal channel for {channel_id}");
    return;
  };

  if let Err(e) = channel.sender.send(message).await {
    warn!("No receiver for {channel_id} | {e:?}");
  };
}

#[instrument("CreateTerminalInner", skip_all, fields(name))]
pub async fn create_terminal(
  name: String,
  target: TerminalTarget,
  command: Option<String>,
  recreate: TerminalRecreateMode,
) -> anyhow::Result<Arc<PeripheryTerminal>> {
  let command = command.unwrap_or_else(|| {
    periphery_config().default_terminal_command.clone()
  });
  trace!(
    "CreateTerminal: {name} | command: {command} | recreate: {recreate:?}"
  );
  let terminals = terminals();
  use TerminalRecreateMode::*;
  if matches!(recreate, Never | DifferentCommand)
    && let Some(terminal) = terminals
      .find(|terminal| {
        terminal.target.matches_on_server(&target)
          && terminal.name == name
      })
      .await
  {
    if terminal.command == command {
      return Ok(terminal.clone());
    } else if matches!(recreate, Never) {
      return Err(anyhow!(
        "Terminal {name} already exists, but has command {} instead of {command}",
        terminal.command
      ));
    }
  }
  let terminal = Arc::new(
    PeripheryTerminal::new(name.clone(), target.clone(), command)
      .await
      .context("Failed to init terminal")?,
  );
  if let Some(prev) = terminals
    .insert(
      |terminal| {
        terminal.target.matches_on_server(&target)
          && terminal.name == name
      },
      terminal.clone(),
    )
    .await
  {
    prev.cancel();
  }
  Ok(terminal)
}

#[instrument("DeleteTerminalInner")]
pub async fn delete_terminal(target: &TerminalTarget, name: &str) {
  if let Some(terminal) = terminals()
    .remove(|terminal| {
      terminal.target.matches_on_server(target)
        && name == terminal.name.as_str()
    })
    .await
  {
    terminal.cancel.cancel();
  }
}

pub async fn list_terminals(
  target: Option<&TerminalTarget>,
) -> Vec<Terminal> {
  let mut terminals = terminals()
    .list()
    .await
    .iter()
    .filter(|terminal| {
      // If no target passed, returns all
      let Some(target) = target else {
        return true;
      };
      match (target, &terminal.target) {
        (
          TerminalTarget::Server { .. },
          TerminalTarget::Server { .. },
        ) => true,
        (
          TerminalTarget::Container {
            container: target_container,
            ..
          },
          TerminalTarget::Container { container, .. },
        ) => target_container == container,
        (
          TerminalTarget::Stack {
            stack: target_stack,
            service: target_service,
          },
          TerminalTarget::Stack { stack, service },
        ) => {
          target_stack == stack
            // If no service passed, only match on stack
            && (target_service.is_none() || target_service == service)
        }
        (
          TerminalTarget::Deployment {
            deployment: target_deployment,
          },
          TerminalTarget::Deployment { deployment },
        ) => target_deployment == deployment,
        _ => false,
      }
    })
    .map(|terminal| Terminal {
      name: terminal.name.clone(),
      target: terminal.target.clone(),
      command: terminal.command.clone(),
      stored_size_kb: terminal.history.size_kb(),
      created_at: terminal.created_at,
    })
    .collect::<Vec<_>>();
  terminals.sort_by(|a, b| a.name.cmp(&b.name));
  terminals
}

pub async fn get_terminal(
  name: &str,
  target: &TerminalTarget,
) -> anyhow::Result<Arc<PeripheryTerminal>> {
  terminals()
    .find(|terminal| {
      terminal.target.matches_on_server(target)
        && terminal.name.as_str() == name
    })
    .await
    .with_context(|| format!("No terminal for {target:?} at {name}"))
}

pub async fn clean_up_terminals() {
  terminals()
    .retain(|terminal| !terminal.cancel.is_cancelled())
    .await;
}

pub async fn delete_all_terminals() {
  terminals()
    .retain(|terminal| {
      terminal.cancel();
      false
    })
    .await;
  // The terminals poll cancel every 500 millis, need to wait for them
  // to finish cancelling.
  tokio::time::sleep(Duration::from_millis(500)).await;
}

pub type StdinSender = mpsc::Sender<TerminalStdinMessage>;
pub type StdoutReceiver = broadcast::Receiver<Bytes>;

pub struct PeripheryTerminal {
  /// The name of the terminal.
  pub name: String,
  /// The target resource of the Terminal.
  pub target: TerminalTarget,
  /// The command used to init the shell.
  pub command: String,
  /// When the Terminal was created.
  pub created_at: i64,

  pub cancel: CancellationToken,
  pub stdin: StdinSender,
  pub stdout: StdoutReceiver,
  pub history: Arc<History>,
}

impl PeripheryTerminal {
  async fn new(
    name: String,
    target: TerminalTarget,
    command: String,
  ) -> anyhow::Result<PeripheryTerminal> {
    trace!("Creating Terminal | Command: {command}");

    let terminal = native_pty_system()
      .openpty(PtySize::default())
      .context("Failed to open terminal")?;

    let mut lexed = shlex::split(&command)
      .context("Invalid command: empty")?
      .into_iter();

    let cmd = lexed.next().context("Command cannot be empty")?;

    let mut cmd = CommandBuilder::new(cmd);

    for arg in lexed {
      cmd.arg(arg);
    }

    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let mut child = terminal
      .slave
      .spawn_command(cmd)
      .context("Failed to spawn child command")?;

    // Check the child didn't stop immediately (after a little wait) with error
    tokio::time::sleep(Duration::from_millis(100)).await;
    if let Some(status) = child
      .try_wait()
      .context("Failed to check child process exit status")?
    {
      return Err(anyhow!(
        "Child process exited immediately with code {}",
        status.exit_code()
      ));
    }

    let mut terminal_write = terminal
      .master
      .take_writer()
      .context("Failed to take terminal writer")?;
    let mut terminal_read = terminal
      .master
      .try_clone_reader()
      .context("Failed to clone terminal reader")?;

    let cancel = CancellationToken::new();

    // CHILD WAIT TASK
    let _cancel = cancel.clone();
    tokio::task::spawn_blocking(move || {
      loop {
        if _cancel.is_cancelled() {
          trace!("child wait handle cancelled from outside");
          if let Err(e) = child.kill() {
            debug!("Failed to kill child | {e:?}");
          }
          break;
        }
        match child.try_wait() {
          Ok(None) => {
            // Continue
            std::thread::sleep(Duration::from_millis(500));
          }
          Ok(Some(code)) => {
            debug!("child exited with code {code}");
            break;
          }
          Err(e) => {
            debug!("failed to wait for child | {e:?}");
            break;
          }
        }
      }
      // Cancel if loop broken
      _cancel.cancel();
    });

    // WS (channel) -> STDIN TASK
    // Theres only one consumer here, so use mpsc
    let (stdin, mut stdin_read) = tokio::sync::mpsc::channel(8192);
    let _cancel = cancel.clone();
    tokio::task::spawn_blocking(move || {
      loop {
        if _cancel.is_cancelled() {
          trace!("terminal write: cancelled from outside");
          break;
        }
        match stdin_read.blocking_recv() {
          // Handled in self::handle_message
          Some(TerminalStdinMessage::Begin) => {}
          Some(TerminalStdinMessage::Forward(bytes)) => {
            if let Err(e) = terminal_write.write_all(&bytes) {
              debug!("Failed to write to PTY: {e:?}");
              break;
            }
          }
          Some(TerminalStdinMessage::Resize(dimensions)) => {
            if let Err(e) = terminal.master.resize(PtySize {
              cols: dimensions.cols,
              rows: dimensions.rows,
              pixel_width: 0,
              pixel_height: 0,
            }) {
              debug!("Failed to resize | {e:?}");
              break;
            };
          }
          None => {
            debug!("WS -> PTY channel read error: Disconnected");
            break;
          }
        }
      }
      // Cancel if loop broken
      _cancel.cancel();
    });

    let history = Arc::new(History::default());

    // PTY -> WS (channel) TASK
    // Uses broadcast to output to multiple client simultaneously
    let (write_stdout, stdout) =
      tokio::sync::broadcast::channel::<Bytes>(8192);
    let _cancel = cancel.clone();
    let _history = history.clone();
    tokio::task::spawn_blocking(move || {
      let mut buf = [0u8; 8192];
      loop {
        if _cancel.is_cancelled() {
          trace!("terminal read: cancelled from outside");
          break;
        }
        match terminal_read.read(&mut buf) {
          Ok(0) => break, // EOF
          Ok(n) => {
            let slice = &buf[..n];
            _history.push(slice);
            if let Err(e) =
              write_stdout.send(Bytes::copy_from_slice(slice))
            {
              debug!("PTY -> WS channel send error: {e:?}");
              break;
            }
          }
          Err(e) => {
            debug!("Failed to read for PTY: {e:?}");
            break;
          }
        }
      }
      // Cancel if loop broken
      _cancel.cancel();
    });

    trace!("terminal tasks spawned");

    Ok(PeripheryTerminal {
      name,
      target,
      command,
      cancel,
      stdin,
      stdout,
      history,
      created_at: komodo_timestamp(),
    })
  }

  pub fn cancel(&self) {
    trace!("Cancel called");
    self.cancel.cancel();
  }
}

/// 1 MiB rolling max history size per terminal
const MAX_BYTES: usize = 1024 * 1024;

pub struct History {
  buf: std::sync::RwLock<VecDeque<u8>>,
}

impl Default for History {
  fn default() -> Self {
    History {
      buf: VecDeque::with_capacity(MAX_BYTES).into(),
    }
  }
}

impl History {
  /// Push some bytes, evicting the oldest when full.
  fn push(&self, bytes: &[u8]) {
    let mut buf = self.buf.write().unwrap();
    for byte in bytes {
      if buf.len() == MAX_BYTES {
        buf.pop_front();
      }
      buf.push_back(*byte);
    }
  }

  pub fn bytes_parts(&self) -> (Bytes, Bytes) {
    let buf = self.buf.read().unwrap();
    let (a, b) = buf.as_slices();
    (Bytes::copy_from_slice(a), Bytes::copy_from_slice(b))
  }

  pub fn size_kb(&self) -> f64 {
    self.buf.read().unwrap().len() as f64 / 1024.0
  }
}
