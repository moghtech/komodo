use std::{
  collections::HashMap,
  sync::{Arc, OnceLock},
  task::Poll,
};

use anyhow::Context;
use flume::r#async::RecvStream;
use futures::{Stream, StreamExt};
use komodo_client::entities::KOMODO_EXIT_DATA;
use pin_project_lite::pin_project;
use tokio::io::AsyncWriteExt;

const KOMODO_END_OF_OUTPUT: &str = "__KOMODO_END_OF_OUTPUT__";

type TerminalName = String;
type TerminalMap =
  std::sync::RwLock<HashMap<TerminalName, Arc<Terminal>>>;

pub fn delete_terminal(name: &str) -> Option<Arc<Terminal>> {
  terminals().write().unwrap().remove(name)
}

pub fn list_terminals() -> Vec<String> {
  terminals().read().unwrap().keys().cloned().collect()
}

pub async fn run_command_on_terminal(
  terminal_name: String,
  command: String,
) -> anyhow::Result<TerminalStream> {
  let shell = get_or_insert_terminal(terminal_name.clone());
  if let Err(e) = shell
    .run(command)
    .await
    .context("Failed to run command on shell")
  {
    delete_terminal(&terminal_name);
    return Err(e);
  };
  Ok(TerminalStream {
    inner: shell.stdout.clone().into_stream(),
    name: terminal_name,
  })
}

fn terminals() -> &'static TerminalMap {
  static TERMINALS: OnceLock<TerminalMap> = OnceLock::new();
  TERMINALS.get_or_init(Default::default)
}

fn get_or_insert_terminal(name: String) -> Arc<Terminal> {
  if let Some(terminal) = terminals().read().unwrap().get(&name) {
    return terminal.clone();
  }
  let terminal = Arc::new(Terminal::new());
  terminals().write().unwrap().insert(name, terminal.clone());
  terminal
}

fn maybe_delete_terminal(name: &str) {
  info!("running maybe clear");
  let shell = terminals().read().unwrap().get(name).cloned();
  if let Some(shell) = shell {
    match shell.child.lock().unwrap().try_wait() {
      Ok(Some(code)) => {
        warn!("shell {name} has exited with status {code}");
        delete_terminal(name);
      }
      Ok(None) => {}
      Err(e) => {
        error!("Shell has error try_wait | {e:?}");
      }
    }
  }
}

pin_project! {
  pub struct TerminalStream { #[pin] inner: RecvStream<'static, Result<String, String>>, name: String }
}

impl Stream for TerminalStream {
  type Item = Result<String, String>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let this = self.project();
    match this.inner.poll_next(cx) {
      Poll::Ready(None) => {
        // This is if a None comes in before END_OF_OUTPUT.
        // This probably means the terminal has exited early,
        // and needs to be cleaned up
        maybe_delete_terminal(&this.name);
        Poll::Ready(None)
      }
      Poll::Ready(Some(line)) => {
        match line {
          Ok(line) if line == KOMODO_END_OF_OUTPUT => {
            // Stop the stream on end sentinel
            Poll::Ready(None)
          }
          Ok(line) => Poll::Ready(Some(Ok(line + "\n"))),
          Err(e) => Poll::Ready(Some(Err(e))),
        }
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

pub struct Terminal {
  child: std::sync::Mutex<tokio::process::Child>,
  stdin: tokio::sync::Mutex<tokio::process::ChildStdin>,
  // receives lines from stdout
  // use flume channel because you need a receiver that
  // you can clone off and turn into a stream back to client.
  stdout: flume::Receiver<Result<String, String>>,
}

impl Terminal {
  fn new() -> Terminal {
    let mut child = tokio::process::Command::new("bash")
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .spawn()
      .expect("Failed to spawn shell");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    let mut stdout = tokio_util::codec::FramedRead::new(
      stdout,
      tokio_util::codec::LinesCodec::new(),
    );

    let (stdout_tx, stdout_rx) = flume::unbounded();

    // spawn stdout forwarding loop
    tokio::spawn(async move {
      // info!("spawned stdout forwarder");
      loop {
        match stdout.next().await {
          Some(line) => {
            // info!("got line: {line:?}");
            if let Err(e) = stdout_tx
              .send_async(
                line
                  .context("Bad line | LinesCodecError")
                  .map_err(|e| format!("{e:#}")),
              )
              .await
              .context("Failed to send line")
            {
              error!("{e:#}");
              break;
            }
          }
          None => {
            // warn!("Stream done");
            break;
          }
        }
      }
    });

    Terminal {
      child: child.into(),
      stdin: stdin.into(),
      stdout: stdout_rx,
    }
  }

  async fn run(
    &self,
    command: impl Into<String>,
  ) -> anyhow::Result<()> {
    let command: String = command.into();
    // The bash wrapping combines stdout and stderr,
    // and attaches the command exit code and END_OF_OUTPUT sentinel.
    let full_command = format!(
      "exec 2>&1; {command}; printf '{KOMODO_EXIT_DATA}%d:%s\n{KOMODO_END_OF_OUTPUT}\n' \"$?\" \"$PWD\"\n"
    );
    // let full_command = format!(
    //   "exec 2>&1; {command}; printf '{KOMODO_EXIT_CODE}%d:%s\n' \"$?\" \"$PWD\"; printf '{KOMODO_END_OF_OUTPUT}'\n"
    // );

    let mut stdin =
      self.stdin.try_lock().context("Shell stdin is busy")?;
    stdin
      .write(full_command.as_bytes())
      .await
      .context("Failed to write command to stdin")?;

    Ok(())
  }
}
