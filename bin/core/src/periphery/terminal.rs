use std::{
  pin::Pin,
  sync::Arc,
  task::{self, Poll},
};

use anyhow::{Context, anyhow};
use futures_util::Stream;
use komodo_client::entities::terminal::{
  TerminalStdinMessageVariant, TerminalTarget,
};
use mogh_cache::CloneCache;
use periphery_client::{
  api::terminal::{ConnectTerminal, END_OF_OUTPUT, ExecuteTerminal},
  transport::EncodedTransportMessage,
};
use transport::channel::{Receiver, Sender};
use uuid::Uuid;

use crate::{
  periphery::PeripheryClient, state::periphery_connections,
};

pub struct ConnectTerminalResponse {
  pub channel: Uuid,
  pub sender: Sender<EncodedTransportMessage>,
  pub receiver: Receiver<anyhow::Result<Vec<u8>>>,
}

impl PeripheryClient {
  #[instrument("ConnectTerminal", skip(self), fields(server_id = self.id))]
  pub async fn connect_terminal(
    &self,
    terminal: String,
    target: TerminalTarget,
  ) -> anyhow::Result<ConnectTerminalResponse> {
    tracing::trace!(
      "request | type: ConnectTerminal | Terminal: {terminal} | Target: {target:?}",
    );

    let connection =
      periphery_connections().get(&self.id).await.with_context(
        || format!("No connection found for server {}", self.id),
      )?;

    let channel = self
      .request(ConnectTerminal { terminal, target })
      .await
      .context("Failed to create terminal connection")?;

    let (sender, receiver) = transport::channel::channel();
    connection.terminals.insert(channel, sender).await;

    if let Err(e) = connection
      .sender
      .send_terminal(
        channel,
        Ok(vec![TerminalStdinMessageVariant::Begin.as_byte()]),
      )
      .await
    {
      connection.terminals.remove(&channel).await;
      return Err(e).context(
        "Failed to send TerminalMessage Begin byte to begin forwarding.",
      );
    }

    Ok(ConnectTerminalResponse {
      channel,
      sender: connection.sender.clone(),
      receiver,
    })
  }

  /// Executes command on specified terminal,
  /// and streams the response ending in [KOMODO_EXIT_CODE][komodo_client::entities::KOMODO_EXIT_CODE]
  /// sentinal value as the expected final line of the stream.
  ///
  /// Example final line:
  /// ```text
  /// __KOMODO_EXIT_CODE:0
  /// ```
  ///
  /// This means the command exited with code 0 (success).
  ///
  /// If this value is NOT the final item before stream closes, it means
  /// the terminal exited mid command, before giving status. Example: running `exit`.
  #[instrument("ExecuteTerminal", skip(self), fields(server_id = self.id))]
  pub async fn execute_terminal(
    &self,
    target: TerminalTarget,
    terminal: String,
    command: String,
  ) -> anyhow::Result<
    impl Stream<Item = anyhow::Result<Vec<u8>>> + 'static,
  > {
    trace!(
      "sending request | type: ExecuteTerminal | {target:?} | terminal name: {terminal} | command: {command}",
    );

    let connection =
      periphery_connections().get(&self.id).await.with_context(
        || format!("No connection found for server {}", self.id),
      )?;

    let channel = self
      .request(ExecuteTerminal {
        terminal,
        target,
        command,
      })
      .await
      .context("Failed to create execute terminal connection")?;

    let (terminal_sender, terminal_receiver) =
      transport::channel::channel();
    connection.terminals.insert(channel, terminal_sender).await;

    if let Err(e) = connection
      .sender
      .send_terminal(
        channel,
        Ok(vec![TerminalStdinMessageVariant::Begin.as_byte()]),
      )
      .await
    {
      // No stream was created, so no `Drop` will clean this up.
      connection.terminals.remove(&channel).await;
      return Err(e).context(
        "Failed to send TerminalTrigger to begin forwarding.",
      );
    }

    Ok(ReceiverStream {
      channel,
      receiver: terminal_receiver,
      channels: connection.terminals.clone(),
      server: self.id.clone(),
      completed: false,
    })
  }
}

pub struct ReceiverStream {
  channel: Uuid,
  channels: Arc<CloneCache<Uuid, Sender<anyhow::Result<Vec<u8>>>>>,
  receiver: Receiver<anyhow::Result<Vec<u8>>>,
  server: String,
  /// Whether the stream reached its end, as opposed to being dropped
  /// part way through.
  completed: bool,
}

impl Stream for ReceiverStream {
  type Item = anyhow::Result<Vec<u8>>;
  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match self.receiver.poll_recv(cx) {
      Poll::Ready(Some(Ok(bytes)))
        if bytes == END_OF_OUTPUT.as_bytes() =>
      {
        self.completed = true;
        self.cleanup();
        Poll::Ready(None)
      }
      Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(bytes))),
      Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
      // Not marked completed: the channel closing early is not the
      // sentinel, so Periphery may still be forwarding.
      Poll::Ready(None) => {
        self.cleanup();
        Poll::Ready(None)
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

impl Drop for ReceiverStream {
  fn drop(&mut self) {
    // A stream dropped before it completes (client disconnect,
    // cancelled Action, timeout) never sees END_OF_OUTPUT, so without
    // this its channel is left in the map forever. Periphery keeps
    // forwarding to it, and Core logs a missing channel for every line.
    self.cleanup();
  }
}

impl ReceiverStream {
  fn cleanup(&self) {
    // Not the prettiest but it should be fine
    let channels = self.channels.clone();
    let channel = self.channel;
    let server = self.server.clone();
    let completed = self.completed;
    // `Drop` can run outside the runtime, where `spawn` would panic.
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
      return;
    };
    handle.spawn(async move {
      channels.remove(&channel).await;
      if completed {
        return;
      }
      // Periphery is still forwarding, so tell it to stop, using the
      // same message shape the interactive terminal sends on client
      // disconnect. Looked up rather than held, so a reconnect in the
      // meantime does not leave this sending on a dead channel.
      let Some(connection) =
        periphery_connections().get(&server).await
      else {
        return;
      };
      let _ = connection
        .sender
        .send_terminal(channel, Err(anyhow!("Stream dropped")))
        .await;
    });
  }
}
