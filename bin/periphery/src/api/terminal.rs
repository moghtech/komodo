use std::{sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use futures_util::{Stream, StreamExt, TryStreamExt};
use komodo_client::entities::{
  KOMODO_EXIT_CODE, NoData, optional_string,
  terminal::{Terminal, TerminalStdinMessage, TerminalTarget},
};
use mogh_resolver::Resolve;
use periphery_client::{
  api::terminal::*, transport::EncodedTransportMessage,
};
use tokio_util::{codec::LinesCodecError, sync::CancellationToken};
use transport::channel::{BufferedChannel, Sender};
use uuid::Uuid;

use crate::{
  config::periphery_config,
  state::{
    TerminalChannel, core_connections, terminal_channels,
    terminal_triggers,
  },
  terminal::*,
};

//

impl Resolve<crate::api::Args> for ListTerminals {
  async fn resolve(
    self,
    _: &crate::api::Args,
  ) -> anyhow::Result<Vec<Terminal>> {
    clean_up_terminals().await;
    Ok(list_terminals(self.target.as_ref()).await)
  }
}

//

impl Resolve<crate::api::Args> for CreateServerTerminal {
  #[instrument(
    "CreateServerTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.name,
      command = self.command,
      recreate = format!("{:?}", self.recreate),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Terminal> {
    if periphery_config().disable_terminals {
      return Err(anyhow!(
        "Terminals are disabled in the Periphery config"
      ));
    }
    let existing =
      list_terminals(Some(&TerminalTarget::Server { server: None }))
        .await;
    create_terminal(
      self
        .name
        .and_then(optional_string)
        .unwrap_or_else(|| format!("term-{}", existing.len())),
      TerminalTarget::Server { server: None },
      self.command,
      self.recreate,
    )
    .await
    .map(|terminal| Terminal {
      name: terminal.name.clone(),
      target: TerminalTarget::Server { server: None },
      target_name: None,
      command: terminal.command.clone(),
      stored_size_kb: terminal.history.size_kb(),
      created_at: terminal.created_at,
    })
  }
}

//

impl Resolve<crate::api::Args> for CreateContainerExecTerminal {
  #[instrument(
    "CreateContainerExecTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.name,
      target = format!("{:?}", self.target),
      command = self.command,
      recreate = format!("{:?}", self.recreate),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Terminal> {
    if periphery_config().disable_container_terminals {
      return Err(anyhow!(
        "Container Terminals are disabled in the Periphery config"
      ));
    }
    let CreateContainerExecTerminal {
      name,
      target,
      container,
      command,
      recreate,
    } = self;
    let command = command.unwrap_or_else(|| String::from("sh"));
    if container.contains("&&") || command.contains("&&") {
      return Err(anyhow!(
        "The use of '&&' is forbidden in the container name or command"
      ));
    }
    let existing = list_terminals(Some(&target)).await;
    create_terminal(
      name.and_then(optional_string).unwrap_or_else(|| {
        format!("exec-{container}-{}", existing.len())
      }),
      target,
      Some(format!("docker exec -it {container} {command}")),
      recreate,
    )
    .await
    .map(|terminal| Terminal {
      name: terminal.name.clone(),
      target: terminal.target.clone(),
      target_name: None,
      command: terminal.command.clone(),
      stored_size_kb: terminal.history.size_kb(),
      created_at: terminal.created_at,
    })
  }
}

//

impl Resolve<crate::api::Args> for CreateContainerAttachTerminal {
  #[instrument(
    "CreateContainerAttachTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.name,
      target = format!("{:?}", self.target),
      recreate = format!("{:?}", self.recreate),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Terminal> {
    if periphery_config().disable_container_terminals {
      return Err(anyhow!(
        "Container Terminals are disabled in the Periphery config"
      ));
    }
    let CreateContainerAttachTerminal {
      name,
      target,
      container,
      recreate,
    } = self;
    if container.contains("&&") {
      return Err(anyhow!(
        "The use of '&&' is forbidden in the container name"
      ));
    }
    let existing = list_terminals(Some(&target)).await;
    create_terminal(
      name.and_then(optional_string).unwrap_or_else(|| {
        format!("attach-{container}-{}", existing.len())
      }),
      target,
      Some(format!("docker attach {container} --sig-proxy=false")),
      recreate,
    )
    .await
    .map(|terminal| Terminal {
      name: terminal.name.clone(),
      target: terminal.target.clone(),
      target_name: None,
      command: terminal.command.clone(),
      stored_size_kb: terminal.history.size_kb(),
      created_at: terminal.created_at,
    })
  }
}

//

impl Resolve<crate::api::Args> for DeleteTerminal {
  #[instrument(
    "DeleteTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.terminal,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<NoData> {
    delete_terminal(&self.target, &self.terminal).await;
    Ok(NoData {})
  }
}

//

impl Resolve<crate::api::Args> for DeleteAllTerminals {
  #[instrument(
    "DeleteAllTerminals",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<NoData> {
    delete_all_terminals().await;
    Ok(NoData {})
  }
}

//

impl Resolve<crate::api::Args> for ConnectTerminal {
  #[instrument(
    "ConnectTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.terminal,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Uuid> {
    let connection =
      core_connections().get(&args.core).await.with_context(
        || format!("Failed to find channel for {}", args.core),
      )?;

    clean_up_terminals().await;

    let terminal = get_terminal(&self.terminal, &self.target).await?;

    let channel =
      spawn_terminal_forwarding(connection, terminal).await;

    Ok(channel)
  }
}

//

impl Resolve<crate::api::Args> for DisconnectTerminal {
  #[instrument(
    "DisconnectTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      channel_id = self.channel.to_string(),
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<NoData> {
    terminal_channels().remove(&self.channel).await;
    Ok(NoData {})
  }
}

//

impl Resolve<crate::api::Args> for ExecuteTerminal {
  #[instrument(
    "ExecuteTerminal",
    skip_all,
    fields(
      id = args.id.to_string(),
      core = args.core,
      terminal = self.terminal,
      command = self.command,
    )
  )]
  async fn resolve(
    self,
    args: &crate::api::Args,
  ) -> anyhow::Result<Uuid> {
    let channel =
      core_connections().get(&args.core).await.with_context(
        || format!("Failed to find channel for {}", args.core),
      )?;

    let terminal = get_terminal(&self.terminal, &self.target).await?;

    let channel_id = Uuid::new_v4();
    let cancel = CancellationToken::new();

    let stdout = setup_execute_command_on_terminal(
      channel_id,
      &terminal,
      &self.command,
    )
    .await?;

    // Registered like ConnectTerminal's channel, so DisconnectTerminal
    // and a Core disconnect can cancel the forwarding task below.
    terminal_channels()
      .insert(
        channel_id,
        Arc::new(TerminalChannel {
          sender: terminal.stdin.clone(),
          cancel: cancel.clone(),
        }),
      )
      .await;

    // Cloned out here so the task does not hold the whole terminal,
    // including its scrollback history, for as long as it runs.
    let terminal_cancel = terminal.cancel.clone();

    tokio::spawn(async move {
      forward_execute_command_on_terminal_response(
        &channel.sender,
        channel_id,
        stdout,
        terminal_cancel,
        cancel,
      )
      .await
    });

    Ok(channel_id)
  }
}

#[instrument("SpawnTerminalForwarding", skip_all)]
async fn spawn_terminal_forwarding(
  connection: Arc<BufferedChannel<EncodedTransportMessage>>,
  terminal: Arc<PeripheryTerminal>,
) -> Uuid {
  let channel = Uuid::new_v4();
  let cancel = CancellationToken::new();

  tokio::join!(
    terminal_channels().insert(
      channel,
      Arc::new(TerminalChannel {
        sender: terminal.stdin.clone(),
        cancel: cancel.clone(),
      }),
    ),
    terminal_triggers().insert(channel),
  );

  tokio::spawn(async move {
    handle_terminal_forwarding(
      &connection.sender,
      channel,
      terminal,
      cancel,
    )
    .await
  });

  channel
}

async fn handle_terminal_forwarding(
  sender: &Sender<EncodedTransportMessage>,
  channel: Uuid,
  terminal: Arc<PeripheryTerminal>,
  cancel: CancellationToken,
) {
  // This waits to begin forwarding until Core sends the None byte start trigger.
  // This ensures no messages are lost before channels on both sides are set up.
  if let Err(e) = terminal_triggers().recv(&channel).await {
    warn!(
      "Failed to init terminal | Failed to receive begin trigger | {e:#}"
    );
    terminal_channels().remove(&channel).await;
    return;
  }

  let init_res = async {
    let (a, b) = terminal.history.bytes_parts();
    if !a.is_empty() {
      sender
        .send_terminal(channel, Ok(a.into()))
        .await
        .context("Failed to send history part a")?;
    }
    if !b.is_empty() {
      sender
        .send_terminal(channel, Ok(b.into()))
        .await
        .context("Failed to send history part b")?;
    }
    anyhow::Ok(())
  }
  .await;

  if let Err(e) = init_res {
    // TODO: Handle error
    warn!("Failed to init terminal | {e:#}");
    terminal_channels().remove(&channel).await;
    return;
  }

  // Forward stdout -> WS
  let mut stdout = terminal.stdout.resubscribe();

  loop {
    let res = tokio::select! {
      res = stdout.recv() => res,
      _ = terminal.cancel.cancelled() => {
        let _ = sender.send_terminal_exited(channel).await;
        break
      },
      _ = cancel.cancelled() => {
        break
      }
    };

    let bytes = match res {
      Ok(bytes) => bytes,
      Err(_e) => {
        let _ = sender.send_terminal_exited(channel).await;
        break;
      }
    };

    if let Err(e) =
      sender.send_terminal(channel, Ok(bytes.into())).await
    {
      debug!("Failed to send to WS: {e:?}");
      cancel.cancel();
      break;
    }
  }

  // Clean up
  terminal_channels().remove(&channel).await;
  clean_up_terminals().await;
}

/// This is run before spawning task handler
#[instrument("SetupExecuteTerminal", skip(terminal))]
async fn setup_execute_command_on_terminal(
  channel_id: Uuid,
  terminal: &PeripheryTerminal,
  command: &str,
) -> anyhow::Result<
  impl Stream<Item = Result<String, LinesCodecError>> + 'static,
> {
  // Read the bytes into lines
  // This is done to check the lines for the EOF sentinal
  let mut stdout = tokio_util::codec::FramedRead::new(
    tokio_util::io::StreamReader::new(
      tokio_stream::wrappers::BroadcastStream::new(
        terminal.stdout.resubscribe(),
      )
      .map(|res| res.map_err(std::io::Error::other)),
    ),
    tokio_util::codec::LinesCodec::new(),
  );

  // Build the command as a single physical line: use `\n` escapes that `printf`
  // expands, rather than real newline bytes. A multi-line command is echoed back
  // by the PTY (local echo is on for interactive use) with the sentinels on their
  // own lines, and the reader below matches those *echoed* lines instead of the
  // real command output (issue #1289). A single-line command cannot produce a bare
  // sentinel line in its echo, while `printf` still emits the sentinels on their
  // own lines in the actual output.
  let full_command = format!(
    "printf '\\n{START_OF_OUTPUT}\\n\\n'; {command}; rc=$?; printf '\\n{KOMODO_EXIT_CODE}%d\\n{END_OF_OUTPUT}\\n' \"$rc\"\n"
  );

  terminal
    .stdin
    .send(TerminalStdinMessage::forward(full_command.into()))
    .await
    .context("Failed to send command to terminal stdin")?;

  // Only start the response AFTER the start sentinel is printed
  loop {
    match stdout
      .try_next()
      .await
      .context("Failed to read stdout line")?
    {
      Some(line) if line == START_OF_OUTPUT => break,
      // Keep looping until the start sentinel received.
      Some(_) => {}
      None => {
        return Err(anyhow!(
          "Stdout stream terminated before start sentinel received"
        ));
      }
    }
  }

  terminal_triggers().insert(channel_id).await;

  Ok(stdout)
}

/// How long to wait for Core's begin trigger before giving up, rather
/// than holding the terminal and the channel open indefinitely.
const BEGIN_TRIGGER_TIMEOUT: Duration = Duration::from_secs(30);

/// Ends the response stream without an exit code, which Core's client
/// reports as an early exit. Sent instead of an error, because an error
/// aborts the HTTP body, surfacing as a failed request rather than as a
/// command that stopped early.
///
/// The reason is sent as a line of output first: reacting to the missing
/// exit code is optional for a caller, so on its own it would let a
/// truncated command read as a complete one.
async fn end_execute_stream(
  sender: &Sender<EncodedTransportMessage>,
  channel: Uuid,
  reason: &str,
) {
  let reason =
    format!("\n[komodo] {reason}, no exit code reported\n");
  let _ = sender.send_terminal(channel, Ok(reason.into())).await;
  if let Err(e) = sender
    .send_terminal(channel, Ok(END_OF_OUTPUT.into()))
    .await
  {
    debug!("Failed to send END_OF_OUTPUT | {e:?}");
  }
}

async fn forward_execute_command_on_terminal_response(
  sender: &Sender<EncodedTransportMessage>,
  channel: Uuid,
  mut stdout: impl Stream<Item = Result<String, LinesCodecError>> + Unpin,
  terminal_cancel: CancellationToken,
  cancel: CancellationToken,
) {
  // This waits to begin forwarding until Core sends the Begin byte start trigger.
  // This ensures no messages are lost before channels on both sides are set up.
  let trigger = tokio::select! {
    // Biased: a trigger which already arrived must not lose the race to
    // a cancel that is also ready.
    biased;
    res = terminal_triggers().recv(&channel) => res,
    _ = terminal_cancel.cancelled() => {
      Err(anyhow!("Terminal exited before begin trigger"))
    },
    _ = cancel.cancelled() => {
      Err(anyhow!("Channel cancelled before begin trigger"))
    },
    // Core sends the trigger as soon as it has the channel id, so this
    // only fires if it never will, eg. the connection dropped first.
    _ = tokio::time::sleep(BEGIN_TRIGGER_TIMEOUT) => {
      Err(anyhow!("Timed out waiting for begin trigger"))
    },
  };
  if let Err(e) = trigger {
    warn!("{e:#}");
    // Only removed by `recv` on success, so it needs removing here.
    terminal_triggers().remove(&channel).await;
    terminal_channels().remove(&channel).await;
    // Unless Core is the one that cancelled, in which case the channel
    // is already gone and sending only logs a missing channel there.
    if !cancel.is_cancelled() {
      end_execute_stream(sender, channel, &format!("{e:#}")).await;
    }
    return;
  }

  loop {
    // Core is gone, so nothing is waiting on the rest of the output.
    // Checked here as well as in the `select!`, because a command
    // printing without pause would otherwise starve that branch.
    if cancel.is_cancelled() {
      break;
    }

    // A command that never finishes leaves `stdout` pending forever, so
    // the cancels are the only way out of this loop.
    let next = tokio::select! {
      // Biased, and no equivalent loop-top check for the terminal: on
      // exit it cancels before dropping the PTY, so output is still
      // queued behind a ready token, and the exit code is in it. The
      // stream ends on its own once drained, which is the `None` arm.
      biased;
      next = tokio::task::coop::unconstrained(stdout.next()) => next,
      _ = terminal_cancel.cancelled() => {
        end_execute_stream(sender, channel, "Terminal exited").await;
        break
      },
      _ = cancel.cancelled() => break,
    };

    match next {
      Some(Ok(line)) if line.as_str() == END_OF_OUTPUT => {
        if let Err(e) =
          sender.send_terminal(channel, Ok(line.into())).await
        {
          warn!("Got ws_sender send error on END_OF_OUTPUT | {e:?}");
        }
        break;
      }
      Some(Ok(line)) => {
        if let Err(e) = sender
          .send_terminal(channel, Ok((line + "\n").into()))
          .await
        {
          warn!("Got ws_sender send error | {e:?}");
          break;
        }
      }
      Some(Err(e)) => {
        warn!("Got stdout stream error | {e:?}");
        end_execute_stream(sender, channel, "Output stream failed")
          .await;
        break;
      }
      None => {
        end_execute_stream(sender, channel, "Terminal output ended")
          .await;
        break;
      }
    }
  }

  terminal_channels().remove(&channel).await;
  clean_up_terminals().await;
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A terminal cancels its token just before dropping the PTY, so by the
  /// time forwarding sees the cancellation the remaining output is already
  /// queued. All of it still has to be sent: the exit code is the last
  /// thing in it, and Core's stream does not end until the sentinel
  /// arrives.
  ///
  /// Built on the same stream chain as [setup_execute_command_on_terminal],
  /// with the output split across many chunks, because that is what makes
  /// the forwarding loop yield mid-drain. A single ready chunk would not
  /// exercise it.
  #[tokio::test]
  async fn cancelled_terminal_still_forwards_buffered_output() {
    const LINES: usize = 300;

    let channel = Uuid::new_v4();
    let (sender, mut receiver) =
      transport::channel::channel::<EncodedTransportMessage>();

    let (stdout_sender, stdout_receiver) =
      tokio::sync::broadcast::channel::<bytes::Bytes>(8192);
    let stdout = tokio_util::codec::FramedRead::new(
      tokio_util::io::StreamReader::new(
        tokio_stream::wrappers::BroadcastStream::new(stdout_receiver)
          .map(|res| res.map_err(std::io::Error::other)),
      ),
      tokio_util::codec::LinesCodec::new(),
    );

    let mut output = String::new();
    for i in 0..LINES {
      output.push_str(&format!("line {i}\n"));
    }
    output.push_str(&format!("{KOMODO_EXIT_CODE}0\n"));
    output.push_str(&format!("{END_OF_OUTPUT}\n"));
    for chunk in output.as_bytes().chunks(3) {
      stdout_sender
        .send(bytes::Bytes::copy_from_slice(chunk))
        .unwrap();
    }

    // Core has already sent the begin trigger.
    terminal_triggers().insert(channel).await;
    terminal_triggers().send(&channel).await.unwrap();

    // The terminal is gone before forwarding even starts.
    let terminal_cancel = CancellationToken::new();
    terminal_cancel.cancel();

    forward_execute_command_on_terminal_response(
      &sender,
      channel,
      stdout,
      terminal_cancel,
      CancellationToken::new(),
    )
    .await;

    drop(sender);
    let mut sent = 0;
    while receiver.recv().await.is_ok() {
      sent += 1;
    }
    // Every line, plus the exit code and the sentinel.
    assert_eq!(
      sent,
      LINES + 2,
      "output buffered before the terminal was cancelled got dropped"
    );
  }
}
