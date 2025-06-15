use std::{
  collections::{HashMap, VecDeque},
  pin::Pin,
  sync::{Arc, OnceLock},
  task::Poll,
  time::Duration,
};

use anyhow::{Context, anyhow};
use axum::http::StatusCode;
use bytes::Bytes;
use futures::Stream;
use komodo_client::{
  api::write::TerminalRecreateMode,
  entities::{komodo_timestamp, server::TerminalInfo},
};
use pin_project_lite::pin_project;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use rand::Rng;
use serror::AddStatusCodeError;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

type PtyName = String;
type PtyMap = tokio::sync::RwLock<HashMap<PtyName, Arc<Terminal>>>;
type StdinSender = mpsc::Sender<StdinMsg>;
type StdoutReceiver = broadcast::Receiver<Bytes>;

pub async fn create_terminal(
  name: String,
  command: String,
  recreate: TerminalRecreateMode,
) -> anyhow::Result<()> {
  trace!(
    "CreateTerminal: {name} | command: {command} | recreate: {recreate:?}"
  );
  let mut terminals = terminals().write().await;
  use TerminalRecreateMode::*;
  if matches!(recreate, Never | DifferentCommand) {
    if let Some(terminal) = terminals.get(&name) {
      if terminal.command == command {
        return Ok(());
      } else if matches!(recreate, Never) {
        return Err(anyhow!(
          "Terminal {name} already exists, but has command {} instead of {command}",
          terminal.command
        ));
      }
    }
  }
  if let Some(prev) = terminals.insert(
    name,
    Terminal::new(command)
      .await
      .context("Failed to init terminal")?
      .into(),
  ) {
    prev.cancel();
  }
  Ok(())
}

pub async fn delete_terminal(name: &str) {
  if let Some(terminal) = terminals().write().await.remove(name) {
    terminal.cancel.cancel();
  }
}

pub async fn list_terminals() -> Vec<TerminalInfo> {
  let mut terminals = terminals()
    .read()
    .await
    .iter()
    .map(|(name, terminal)| TerminalInfo {
      name: name.to_string(),
      command: terminal.command.clone(),
      stored_size_kb: terminal.history.size_kb(),
    })
    .collect::<Vec<_>>();
  terminals.sort_by(|a, b| a.name.cmp(&b.name));
  terminals
}

pub async fn get_terminal(
  name: &str,
) -> anyhow::Result<Arc<Terminal>> {
  terminals()
    .read()
    .await
    .get(name)
    .cloned()
    .with_context(|| format!("No terminal at {name}"))
}

pub async fn clean_up_terminals() {
  terminals()
    .write()
    .await
    .retain(|_, terminal| !terminal.cancel.is_cancelled());
}

pub async fn delete_all_terminals() {
  terminals()
    .write()
    .await
    .drain()
    .for_each(|(_, terminal)| terminal.cancel());
  // The terminals poll cancel every 500 millis, need to wait for them
  // to finish cancelling.
  tokio::time::sleep(Duration::from_millis(100)).await;
}

fn terminals() -> &'static PtyMap {
  static TERMINALS: OnceLock<PtyMap> = OnceLock::new();
  TERMINALS.get_or_init(Default::default)
}

#[derive(Clone, serde::Deserialize)]
pub struct ResizeDimensions {
  rows: u16,
  cols: u16,
}

#[derive(Clone)]
pub enum StdinMsg {
  Bytes(Bytes),
  Resize(ResizeDimensions),
}

pub struct Terminal {
  /// The command that was used as the root command, eg `shell`
  command: String,

  pub cancel: CancellationToken,

  pub stdin: StdinSender,
  pub stdout: StdoutReceiver,

  pub history: Arc<History>,
}

impl Terminal {
  async fn new(command: String) -> anyhow::Result<Terminal> {
    trace!("Creating terminal with command: {command}");

    let terminal = native_pty_system()
      .openpty(PtySize::default())
      .context("Failed to open terminal")?;

    let mut command_split = command.split(' ').map(|arg| arg.trim());
    let cmd =
      command_split.next().context("Command cannot be empty")?;

    let mut cmd = CommandBuilder::new(cmd);

    for arg in command_split {
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
    // Theres only one consumer here, so use mpsc
    let (stdin, mut channel_read) =
      tokio::sync::mpsc::channel::<StdinMsg>(8192);
    let _cancel = cancel.clone();
    tokio::task::spawn_blocking(move || {
      loop {
        if _cancel.is_cancelled() {
          trace!("terminal write: cancelled from outside");
          break;
        }
        match channel_read.blocking_recv() {
          Some(StdinMsg::Bytes(bytes)) => {
            if let Err(e) = terminal_write.write_all(&bytes) {
              debug!("Failed to write to PTY: {e:?}");
              _cancel.cancel();
              break;
            }
          }
          Some(StdinMsg::Resize(dimensions)) => {
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
          None => {
            debug!("WS -> PTY channel read error: Disconnected");
            _cancel.cancel();
            break;
          }
        }
      }
    });

    let history = Arc::new(History::default());

    // PTY -> WS (channel) TASK
    // Uses broadcast to output to multiple client simultaneously
    let (write, stdout) =
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
      command,
      cancel,
      stdin,
      stdout,
      history,
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

/// Execute Sentinels
pub const START_OF_OUTPUT: &str = "__KOMODO_START_OF_OUTPUT__";
pub const END_OF_OUTPUT: &str = "__KOMODO_END_OF_OUTPUT__";

pin_project! {
  pub struct TerminalStream<S> { #[pin] pub stdout: S }
}

impl<S> Stream for TerminalStream<S>
where
  S:
    Stream<Item = Result<String, tokio_util::codec::LinesCodecError>>,
{
  // Axum expects a stream of results
  type Item = Result<String, String>;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let this = self.project();
    match this.stdout.poll_next(cx) {
      Poll::Ready(None) => {
        // This is if a None comes in before END_OF_OUTPUT.
        // This probably means the terminal has exited early,
        // and needs to be cleaned up
        tokio::spawn(async move { clean_up_terminals().await });
        Poll::Ready(None)
      }
      Poll::Ready(Some(line)) => {
        match line {
          Ok(line) if line.as_str() == END_OF_OUTPUT => {
            // Stop the stream on end sentinel
            Poll::Ready(None)
          }
          Ok(line) => Poll::Ready(Some(Ok(line + "\n"))),
          Err(e) => Poll::Ready(Some(Err(format!("{e:?}")))),
        }
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

/// Tokens valid for 3 seconds
const TOKEN_VALID_FOR_MS: i64 = 3_000;

pub fn auth_tokens() -> &'static AuthTokens {
  static AUTH_TOKENS: OnceLock<AuthTokens> = OnceLock::new();
  AUTH_TOKENS.get_or_init(Default::default)
}

#[derive(Default)]
pub struct AuthTokens {
  map: std::sync::Mutex<HashMap<String, i64>>,
}

impl AuthTokens {
  pub fn create_auth_token(&self) -> String {
    let mut lock = self.map.lock().unwrap();
    // clear out any old tokens here (prevent unbounded growth)
    let ts = komodo_timestamp();
    lock.retain(|_, valid_until| *valid_until > ts);
    let token: String = rand::rng()
      .sample_iter(&rand::distr::Alphanumeric)
      .take(30)
      .map(char::from)
      .collect();
    lock.insert(token.clone(), ts + TOKEN_VALID_FOR_MS);
    token
  }

  pub fn check_token(&self, token: String) -> serror::Result<()> {
    let Some(valid_until) = self.map.lock().unwrap().remove(&token)
    else {
      return Err(
        anyhow!("Terminal auth token not found")
          .status_code(StatusCode::UNAUTHORIZED),
      );
    };
    if komodo_timestamp() <= valid_until {
      Ok(())
    } else {
      Err(
        anyhow!("Terminal token is expired")
          .status_code(StatusCode::UNAUTHORIZED),
      )
    }
  }
}
