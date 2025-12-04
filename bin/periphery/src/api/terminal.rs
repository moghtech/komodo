use std::sync::Arc;

use anyhow::{Context, anyhow};
use futures_util::{Stream, StreamExt, TryStreamExt};
use komodo_client::entities::{
  KOMODO_EXIT_CODE, NoData,
  terminal::{Terminal, TerminalStdinMessage, TerminalTarget},
};
use periphery_client::{
  api::terminal::*, transport::EncodedTransportMessage,
};
use resolver_api::Resolve;
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
  ) -> anyhow::Result<NoData> {
    if periphery_config().disable_terminals {
      return Err(anyhow!(
        "Terminals are disabled in the Periphery config"
      ));
    }
    create_terminal(
      self.name,
      TerminalTarget::Server { server: None },
      self.command,
      self.recreate,
    )
    .await
    .map(|_| NoData {})
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
  ) -> anyhow::Result<NoData> {
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
    create_terminal(
      name,
      target,
      Some(format!("docker exec -it {container} {command}")),
      recreate,
    )
    .await
    .map(|_| NoData {})
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
  ) -> anyhow::Result<NoData> {
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
    create_terminal(
      name,
      target,
      Some(format!("docker attach {container} --sig-proxy=false")),
      recreate,
    )
    .await
    .map(|_| NoData {})
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

    let stdout = setup_execute_command_on_terminal(
      channel_id,
      &terminal,
      &self.command,
    )
    .await?;

    tokio::spawn(async move {
      forward_execute_command_on_terminal_response(
        &channel.sender,
        channel_id,
        stdout,
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

  let full_command = format!(
    "printf '\n{START_OF_OUTPUT}\n\n'; {command}; rc=$?; printf '\n{KOMODO_EXIT_CODE}%d\n{END_OF_OUTPUT}\n' \"$rc\"\n"
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

async fn forward_execute_command_on_terminal_response(
  sender: &Sender<EncodedTransportMessage>,
  channel: Uuid,
  mut stdout: impl Stream<Item = Result<String, LinesCodecError>> + Unpin,
) {
  // This waits to begin forwarding until Core sends the Begin byte start trigger.
  // This ensures no messages are lost before channels on both sides are set up.
  if let Err(e) = terminal_triggers().recv(&channel).await {
    warn!("{e:#}");
    return;
  }

  loop {
    match stdout.next().await {
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
        break;
      }
      None => {
        clean_up_terminals().await;
        break;
      }
    }
  }
}
