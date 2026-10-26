use anyhow::anyhow;
use axum::{
  extract::{WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use bytes::Bytes;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use komodo_client::entities::{
  ResourceTarget, permission::PermissionLevel,
  update::UpdateListItem, user::User,
};
use mogh_auth_server::request_ip::RequestIp;
use mogh_error::serialize_error;
use serde_json::json;
use std::{future::Future, time::Duration};
use tokio::{select, sync::broadcast};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::helpers::{
  channel::update_channel, query::get_user_permission_on_target,
};

pub async fn handler(
  RequestIp(ip): RequestIp,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  // get a reveiver for internal update messages.
  let receiver = update_channel().receiver.resubscribe();

  // handle http -> ws updgrade
  ws.on_upgrade(move |socket| async move {
    let Some((client_socket, user)) =
      super::user_ws_login(socket,  ip).await
    else {
      return;
    };

    let (ws_sender, ws_reciever) = client_socket.split();

    let cancel = CancellationToken::new();

    let user_id = user.id;
    tokio::spawn(forward_updates(
      receiver,
      ws_sender,
      cancel.clone(),
      move |update: UpdateListItem| {
        let user_id = user_id.clone();
        async move {
          // before sending every update, verify user is still valid.
          // kill the connection is user if found to be invalid.
          let user = match super::check_user_valid(&user_id).await {
            Err(e) => {
              return Forward::Close(Message::text(
                json!({ "type": "INVALID_USER", "msg": serialize_error(&e) }).to_string(),
              ));
            }
            Ok(user) => user,
          };

          // Only send if user has permission on the target resource.
          if user_can_see_update(&user, &update.target).await.is_ok() {
            Forward::Send(Message::text(
              serde_json::to_string(&update).unwrap(),
            ))
          } else {
            Forward::Skip
          }
        }
      },
    ));

    // Handle messages from the client.
    // After login, only handles close message.
    read_client(ws_reciever, cancel).await;
  })
}

/// How long to wait for the transport to accept the close
/// before dropping the socket.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// What to do with an update once the user was checked.
enum Forward {
  /// Send the message to the client.
  Send(Message),
  /// The user can't see this update.
  Skip,
  /// Send the message, then close the connection.
  Close(Message),
}

/// Forwards updates from the broadcast channel to the client,
/// and keeps the connection alive.
///
/// Returns after closing the connection, always cancelling `cancel`,
/// when the client went away, the keep-alive failed, `prepare` asked
/// to close, or the channel can't be followed any more.
/// A receiver that lagged behind the channel missed updates the client
/// can't know about, so the connection is closed instead of silently
/// carrying on, and the client reconnects and refetches its state.
async fn forward_updates<T, S, F, Fut>(
  mut receiver: broadcast::Receiver<T>,
  mut ws_sender: S,
  cancel: CancellationToken,
  mut prepare: F,
) where
  T: Clone,
  S: Sink<Message> + Unpin,
  F: FnMut(T) -> Fut,
  Fut: Future<Output = Forward>,
{
  let mut keep_alive =
    tokio::time::interval(super::WS_KEEP_ALIVE_INTERVAL);
  loop {
    // poll for updates off the receiver / await cancel / send keep-alive ping.
    let update = select! {
      _ = cancel.cancelled() => break,
      _ = keep_alive.tick() => {
        if !send_unless_cancelled(
          &mut ws_sender,
          Message::Ping(Bytes::new()),
          &cancel,
        )
        .await
        {
          break;
        }
        continue;
      }
      update = receiver.recv() => match update {
        Ok(update) => update,
        Err(e) => {
          warn!("Closing update websocket, can't follow the update channel | {e}");
          break;
        }
      }
    };

    let (message, close) = match prepare(update).await {
      Forward::Send(message) => (message, false),
      Forward::Close(message) => (message, true),
      Forward::Skip => continue,
    };
    if !send_unless_cancelled(&mut ws_sender, message, &cancel).await
      || close
    {
      break;
    }
  }
  cancel.cancel();
  close_sender(&mut ws_sender, CLOSE_TIMEOUT).await;
}

/// Sends `message`. Returns false if `cancel` fired first or the send
/// failed, so a stalled client can't keep the task from seeing a cancel.
async fn send_unless_cancelled<S: Sink<Message> + Unpin>(
  ws_sender: &mut S,
  message: Message,
  cancel: &CancellationToken,
) -> bool {
  select! {
    _ = cancel.cancelled() => false,
    res = ws_sender.send(message) => res.is_ok(),
  }
}

/// Closes the sending half, giving up after `timeout`
/// so a stalled transport can't hold the task forever.
async fn close_sender<S: Sink<Message> + Unpin>(
  ws_sender: &mut S,
  timeout: Duration,
) {
  let _ = tokio::time::timeout(timeout, ws_sender.close()).await;
}

/// Reads the messages of the client until it closes the connection,
/// the stream ends or fails, or `cancel` is cancelled.
/// Always cancels `cancel` on return, so the sending task stops too.
async fn read_client<St, E>(
  mut ws_reciever: St,
  cancel: CancellationToken,
) where
  St: Stream<Item = Result<Message, E>> + Unpin,
{
  loop {
    let msg = select! {
      _ = cancel.cancelled() => break,
      msg = ws_reciever.next() => msg,
    };
    match msg {
      Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
      Some(Ok(_)) => {}
    }
  }
  cancel.cancel();
}

async fn user_can_see_update(
  user: &User,
  update_target: &ResourceTarget,
) -> anyhow::Result<()> {
  if user.admin {
    return Ok(());
  }
  let permission =
    get_user_permission_on_target(user, update_target).await?;
  if permission.level > PermissionLevel::None {
    Ok(())
  } else {
    Err(anyhow!(
      "user does not have permissions on {update_target:?}"
    ))
  }
}

#[cfg(test)]
mod tests {
  use std::{
    pin::Pin,
    sync::{
      Arc, Mutex,
      atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
  };

  use futures_util::stream;

  use super::*;

  #[derive(Clone, Default)]
  struct TestSink {
    sent: Arc<Mutex<Vec<Message>>>,
    closed: Arc<AtomicBool>,
    send_polls: Arc<AtomicUsize>,
    stall_close: bool,
    stall_send: bool,
    fail_send: bool,
    fail_text: bool,
  }

  impl Sink<Message> for TestSink {
    type Error = ();

    fn poll_ready(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
      self.send_polls.fetch_add(1, Ordering::SeqCst);
      if self.stall_send {
        return Poll::Pending;
      }
      if self.fail_send {
        return Poll::Ready(Err(()));
      }
      Poll::Ready(Ok(()))
    }

    fn start_send(
      self: Pin<&mut Self>,
      item: Message,
    ) -> Result<(), Self::Error> {
      if self.fail_text && matches!(item, Message::Text(_)) {
        return Err(());
      }
      self.sent.lock().unwrap().push(item);
      Ok(())
    }

    fn poll_flush(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }

    fn poll_close(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
      if self.stall_close {
        return Poll::Pending;
      }
      self.closed.store(true, Ordering::SeqCst);
      Poll::Ready(Ok(()))
    }
  }

  impl TestSink {
    fn texts(&self) -> Vec<String> {
      self
        .sent
        .lock()
        .unwrap()
        .iter()
        .filter_map(|msg| match msg {
          Message::Text(text) => Some(text.to_string()),
          _ => None,
        })
        .collect()
    }

    fn is_closed(&self) -> bool {
      self.closed.load(Ordering::SeqCst)
    }
  }

  async fn send_text(update: u32) -> Forward {
    Forward::Send(Message::text(update.to_string()))
  }

  /// A receiver that missed updates: the channel holds 2, 5 were sent.
  fn lagged_receiver() -> broadcast::Receiver<u32> {
    let (sender, receiver) = broadcast::channel(2);
    for update in 0..5 {
      sender.send(update).unwrap();
    }
    receiver
  }

  /// Waits for `condition`, failing the test instead of hanging.
  async fn wait_for(condition: impl Fn() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
      while !condition() {
        tokio::task::yield_now().await;
      }
    })
    .await
    .expect("condition not met in time");
  }

  #[tokio::test]
  async fn lagged_receiver_closes_connection() {
    let sink = TestSink::default();
    let cancel = CancellationToken::new();

    forward_updates(
      lagged_receiver(),
      sink.clone(),
      cancel.clone(),
      send_text,
    )
    .await;

    assert!(cancel.is_cancelled());
    assert!(sink.is_closed());
    assert!(sink.texts().is_empty());
  }

  #[tokio::test]
  async fn closed_channel_closes_connection() {
    let (sender, receiver) = broadcast::channel::<u32>(2);
    drop(sender);
    let sink = TestSink::default();
    let cancel = CancellationToken::new();

    forward_updates(
      receiver,
      sink.clone(),
      cancel.clone(),
      send_text,
    )
    .await;

    assert!(cancel.is_cancelled());
    assert!(sink.is_closed());
  }

  #[tokio::test(start_paused = true)]
  async fn stalled_close_gives_up_after_timeout() {
    let sink = TestSink {
      stall_close: true,
      ..Default::default()
    };
    let cancel = CancellationToken::new();

    // Paused time auto-advances: only a missing timeout would time out here.
    tokio::time::timeout(
      CLOSE_TIMEOUT + Duration::from_secs(1),
      forward_updates(
        lagged_receiver(),
        sink.clone(),
        cancel.clone(),
        send_text,
      ),
    )
    .await
    .expect("close was not bounded");

    assert!(cancel.is_cancelled());
    assert!(!sink.is_closed());
  }

  #[tokio::test]
  async fn stalled_ping_does_not_block_cancel() {
    // The sink never accepts a message: the first keep-alive ping stalls.
    let (_sender, receiver) = broadcast::channel::<u32>(2);
    let sink = TestSink {
      stall_send: true,
      ..Default::default()
    };
    let cancel = CancellationToken::new();
    let task = tokio::spawn(forward_updates(
      receiver,
      sink.clone(),
      cancel.clone(),
      send_text,
    ));

    wait_for(|| sink.send_polls.load(Ordering::SeqCst) > 0).await;
    cancel.cancel();

    tokio::time::timeout(Duration::from_secs(5), task)
      .await
      .expect("stalled ping blocked the cancel")
      .unwrap();
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn failed_ping_closes_connection() {
    let (_sender, receiver) = broadcast::channel::<u32>(2);
    let sink = TestSink {
      fail_send: true,
      ..Default::default()
    };
    let cancel = CancellationToken::new();

    tokio::time::timeout(
      Duration::from_secs(5),
      forward_updates(
        receiver,
        sink.clone(),
        cancel.clone(),
        send_text,
      ),
    )
    .await
    .expect("failed ping did not end the forwarder");

    assert!(cancel.is_cancelled());
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn failed_update_send_closes_connection() {
    let (sender, receiver) = broadcast::channel::<u32>(2);
    let sink = TestSink {
      fail_text: true,
      ..Default::default()
    };
    let cancel = CancellationToken::new();
    sender.send(1).unwrap();

    // Pings pass, only the update (text) send fails.
    tokio::time::timeout(
      Duration::from_secs(5),
      forward_updates(
        receiver,
        sink.clone(),
        cancel.clone(),
        send_text,
      ),
    )
    .await
    .expect("failed update send did not end the forwarder");

    assert!(cancel.is_cancelled());
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn forwards_allowed_updates_until_cancelled() {
    let (sender, receiver) = broadcast::channel(16);
    let sink = TestSink::default();
    let cancel = CancellationToken::new();
    let task = tokio::spawn(forward_updates(
      receiver,
      sink.clone(),
      cancel.clone(),
      |update: u32| async move {
        if update % 2 == 1 {
          send_text(update).await
        } else {
          Forward::Skip
        }
      },
    ));

    for update in 1..=3 {
      sender.send(update).unwrap();
    }
    wait_for(|| sink.texts().len() == 2).await;
    cancel.cancel();
    task.await.unwrap();

    assert_eq!(sink.texts(), ["1", "3"]);
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn close_request_sends_message_then_closes() {
    let (sender, receiver) = broadcast::channel(16);
    let sink = TestSink::default();
    let cancel = CancellationToken::new();
    sender.send(1u32).unwrap();
    sender.send(2u32).unwrap();

    forward_updates(
      receiver,
      sink.clone(),
      cancel.clone(),
      |_| async { Forward::Close(Message::text("INVALID_USER")) },
    )
    .await;

    assert_eq!(sink.texts(), ["INVALID_USER"]);
    assert!(cancel.is_cancelled());
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn client_eof_stops_forwarder() {
    // The sender stays alive and silent: only the reader can end it.
    let (_sender, receiver) = broadcast::channel::<u32>(2);
    let sink = TestSink::default();
    let cancel = CancellationToken::new();
    let task = tokio::spawn(forward_updates(
      receiver,
      sink.clone(),
      cancel.clone(),
      send_text,
    ));

    read_client(
      stream::iter(Vec::<Result<Message, ()>>::new()),
      cancel,
    )
    .await;

    tokio::time::timeout(Duration::from_secs(5), task)
      .await
      .expect("forwarder outlived the client")
      .unwrap();
    assert!(sink.is_closed());
  }

  #[tokio::test]
  async fn reader_cancels_on_client_end() {
    let ends: Vec<Vec<Result<Message, ()>>> = vec![
      vec![],
      vec![Ok(Message::Close(None))],
      vec![Err(())],
      vec![Ok(Message::text("ignored")), Ok(Message::Close(None))],
    ];
    for messages in ends {
      let cancel = CancellationToken::new();
      read_client(stream::iter(messages), cancel.clone()).await;
      assert!(cancel.is_cancelled());
    }
  }

  #[tokio::test]
  async fn reader_returns_when_cancelled() {
    let cancel = CancellationToken::new();
    let task = tokio::spawn(read_client(
      stream::pending::<Result<Message, ()>>(),
      cancel.clone(),
    ));

    cancel.cancel();

    tokio::time::timeout(Duration::from_secs(5), task)
      .await
      .expect("reader ignored cancel")
      .unwrap();
  }
}
