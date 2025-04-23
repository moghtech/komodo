use std::{
  collections::HashMap,
  sync::{Arc, OnceLock},
  time::Duration,
};

use anyhow::{Context, anyhow};
use bytes::Bytes;
use flume::TryRecvError;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio_util::sync::CancellationToken;

type PtyName = String;
type PtyMap = std::sync::RwLock<HashMap<PtyName, Arc<Pty>>>;

pub fn delete_pty(name: &str) {
  if let Some(pty) = ptys().write().unwrap().remove(name) {
    pty.cancel.cancel();
    pty.abort();
  }
}

pub fn list_ptys() -> Vec<String> {
  ptys().read().unwrap().keys().cloned().collect()
}

pub fn get_or_insert_pty(
  name: String,
  shell: String,
) -> anyhow::Result<Arc<Pty>> {
  if let Some(pty) = ptys().read().unwrap().get(&name) {
    if pty.shell != shell {
      return Err(anyhow!(
        "Shell mismatch. Expected {}, got {shell}",
        pty.shell
      ));
    } else {
      return Ok(pty.clone());
    }
  }
  let pty = Arc::new(Pty::new(shell).context("Failed to init pty")?);
  ptys().write().unwrap().insert(name, pty.clone());
  Ok(pty)
}

pub fn clean_up_ptys() {
  ptys().write().unwrap().retain(|_, pty| {
    if pty.cancel.is_cancelled() {
      pty.abort();
      false
    } else {
      true
    }
  });
}

fn ptys() -> &'static PtyMap {
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

pub struct Pty {
  /// The shell that was used as the root command.
  shell: String,
  pub cancel: CancellationToken,

  pub stdin: flume::Sender<StdinMsg>,
  pub stdout: flume::Receiver<Bytes>,

  // need to manually abort this one on cancel
  stdout_task: tokio::task::JoinHandle<()>,
}

impl Pty {
  /// shell should be "sh", "bash", "zsh", etc.
  fn new(shell: String) -> anyhow::Result<Pty> {
    trace!("Creating pty with shell: {shell}");

    let pty = native_pty_system()
      .openpty(PtySize::default())
      .context("Failed to open pty")?;

    let cmd = CommandBuilder::new(&shell);

    let mut child = pty
      .slave
      .spawn_command(cmd)
      .context("Failed to spawn child command")?;

    let mut pty_write = pty
      .master
      .take_writer()
      .context("Failed to take pty writer")?;
    let mut pty_read = pty
      .master
      .try_clone_reader()
      .context("Failed to clone pty reader")?;

    let cancel = CancellationToken::new();

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

    let (stdin, channel_read) = flume::bounded::<StdinMsg>(8192);
    let _cancel = cancel.clone();
    tokio::task::spawn_blocking(move || {
      loop {
        if _cancel.is_cancelled() {
          trace!("pty write: cancelled from outside");
          break;
        }
        match channel_read.try_recv() {
          Ok(StdinMsg::Bytes(bytes)) => {
            if let Err(e) = pty_write.write_all(&bytes) {
              debug!("Failed to write to PTY: {e:?}");
              _cancel.cancel();
              break;
            }
          }
          Ok(StdinMsg::Resize(dimensions)) => {
            if let Err(e) = pty.master.resize(PtySize {
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

    let (write, stdout) = flume::bounded::<Bytes>(8192);
    let _cancel = cancel.clone();
    // This task need to be manually aborted on cancel.
    let stdout_task = tokio::task::spawn_blocking(move || {
      let mut buf = [0u8; 8192];
      loop {
        match pty_read.read(&mut buf) {
          Ok(0) => {
            // EOF
            trace!("Got PTY read EOF");
            _cancel.cancel();
            break;
          }
          Ok(n) => {
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

    trace!("pty tasks spawned");

    Ok(Pty {
      shell,
      cancel,
      stdin,
      stdout,
      stdout_task,
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
