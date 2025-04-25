use std::{
  collections::{HashMap, VecDeque},
  sync::{Arc, OnceLock},
  time::Duration,
};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use flume::TryRecvError;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio_util::sync::CancellationToken;

type PtyName = String;
type PtyMap = std::sync::RwLock<HashMap<PtyName, Arc<Terminal>>>;

pub fn delete_terminal(name: &str) {
  if let Some(terminal) = terminals().write().unwrap().remove(name) {
    terminal.cancel.cancel();
    terminal.abort();
  }
}

pub fn list_terminals() -> Vec<String> {
  terminals().read().unwrap().keys().cloned().collect()
}

pub fn get_or_insert_terminal(
  name: String,
  shell: String,
) -> anyhow::Result<Arc<Terminal>> {
  if let Some(terminal) = terminals().read().unwrap().get(&name) {
    if terminal.shell != shell {
      return Err(anyhow!(
        "Shell mismatch. Expected {}, got {shell}",
        terminal.shell
      ));
    } else {
      return Ok(terminal.clone());
    }
  }
  let terminal = Arc::new(
    Terminal::new(shell).context("Failed to init terminal")?,
  );
  terminals().write().unwrap().insert(name, terminal.clone());
  Ok(terminal)
}

pub fn clean_up_terminals() {
  terminals().write().unwrap().retain(|_, terminal| {
    if terminal.cancel.is_cancelled() {
      terminal.abort();
      false
    } else {
      true
    }
  });
}

fn terminals() -> &'static PtyMap {
  static TERMINALS: OnceLock<PtyMap> = OnceLock::new();
  TERMINALS.get_or_init(Default::default)
}

#[derive(serde::Deserialize)]
pub struct ResizeDimensions {
  rows: u16,
  cols: u16,
}

pub enum StdinMsg {
  Bytes(Bytes),
  Resize(ResizeDimensions),
}

pub struct Terminal {
  /// The shell that was used as the root command.
  shell: String,
  pub cancel: CancellationToken,

  pub stdin: flume::Sender<StdinMsg>,
  pub stdout: flume::Receiver<Bytes>,

  // need to manually abort this one on cancel
  stdout_task: tokio::task::JoinHandle<()>,

  pub history: Arc<History>,
}

impl Terminal {
  /// shell should be "sh", "bash", "zsh", etc.
  fn new(shell: String) -> anyhow::Result<Terminal> {
    trace!("Creating terminal with shell: {shell}");

    let terminal = native_pty_system()
      .openpty(PtySize::default())
      .context("Failed to open terminal")?;

    let cmd = CommandBuilder::new(&shell);

    let mut child = terminal
      .slave
      .spawn_command(cmd)
      .context("Failed to spawn child command")?;

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
          Ok(Some(code)) => {
            debug!("child exited with code {code}");
            _cancel.cancel();
            break;
          }
          Ok(None) => {
            std::thread::sleep(Duration::from_millis(500));
          }
          Err(e) => {
            debug!("failed to wait for child | {e:?}");
            _cancel.cancel();
            break;
          }
        }
      }
    });

    // WS (channel) -> STDIN TASK
    let (stdin, channel_read) = flume::bounded::<StdinMsg>(8192);
    let _cancel = cancel.clone();
    tokio::task::spawn_blocking(move || {
      loop {
        if _cancel.is_cancelled() {
          trace!("terminal write: cancelled from outside");
          break;
        }
        match channel_read.try_recv() {
          Ok(StdinMsg::Bytes(bytes)) => {
            if let Err(e) = terminal_write.write_all(&bytes) {
              debug!("Failed to write to PTY: {e:?}");
              _cancel.cancel();
              break;
            }
          }
          Ok(StdinMsg::Resize(dimensions)) => {
            if let Err(e) = terminal.master.resize(PtySize {
              cols: dimensions.cols,
              rows: dimensions.rows,
              pixel_width: 0,
              pixel_height: 0,
            }) {
              debug!("Failed to resize | {e:?}");
              _cancel.cancel();
              break;
            };
          }
          Err(TryRecvError::Disconnected) => {
            debug!("WS -> PTY channel read error: Disconnected");
            _cancel.cancel();
            break;
          }
          Err(TryRecvError::Empty) => {}
        }
      }
    });

    let history = Arc::new(History::default());

    // PTY -> WS (channel) TASK
    let (write, stdout) = flume::bounded::<Bytes>(8192);
    let _cancel = cancel.clone();
    let _history = history.clone();
    // This task need to be manually aborted on cancel.
    let stdout_task = tokio::task::spawn_blocking(move || {
      let mut buf = [0u8; 8192];
      loop {
        match terminal_read.read(&mut buf) {
          Ok(0) => {
            // EOF
            trace!("Got PTY read EOF");
            _cancel.cancel();
            break;
          }
          Ok(n) => {
            _history.push(&buf[..n]);
            if let Err(e) =
              write.send(Bytes::copy_from_slice(&buf[..n]))
            {
              debug!("PTY -> WS channel send error: {e:?}");
              _cancel.cancel();
              break;
            }
          }
          Err(e) => {
            debug!("Failed to read for PTY: {e:?}");
            _cancel.cancel();
            break;
          }
        }
      }
    });

    trace!("terminal tasks spawned");

    Ok(Terminal {
      shell,
      cancel,
      stdin,
      stdout,
      stdout_task,
      history,
    })
  }

  fn abort(&self) {
    self.stdout_task.abort();
  }

  pub fn cancel(&self) {
    trace!("Cancel called");
    self.cancel.cancel();
    self.abort();
  }
}

/// 1 MiB
const MAX_BYTES: usize = 1 * 1024 * 1024;

pub struct History {
  buf: std::sync::Mutex<VecDeque<u8>>,
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
    let mut buf = self.buf.lock().unwrap();
    for byte in bytes {
      if buf.len() == MAX_BYTES {
        buf.pop_front();
      }
      buf.push_back(*byte);
    }
  }

  pub fn bytes_parts(&self) -> (Bytes, Bytes) {
    let buf = self.buf.lock().unwrap();
    let (a, b) = buf.as_slices();
    (Bytes::copy_from_slice(a), Bytes::copy_from_slice(b))
  }
}
