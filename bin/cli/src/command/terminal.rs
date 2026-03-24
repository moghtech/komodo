use anyhow::{Context, anyhow};
use colored::Colorize;
use komodo_client::{
  api::{
    read::{ListAllDockerContainers, ListServers},
    terminal::InitTerminal,
  },
  entities::{
    config::cli::args::terminal::{Attach, Connect, Exec},
    server::ServerQuery,
    terminal::{
      ContainerTerminalMode, TerminalRecreateMode,
      TerminalResizeMessage, TerminalStdinMessage,
    },
  },
  ws::terminal::TerminalWebsocket,
};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::sync::CancellationToken;

pub async fn handle_connect(
  Connect {
    server,
    name,
    command,
    recreate,
  }: &Connect,
) -> anyhow::Result<()> {
  handle_terminal_forwarding(server, async {
    super::komodo_client()
      .await?
      .connect_server_terminal(
        server.to_string(),
        Some(name.to_string()),
        Some(InitTerminal {
          command: command.clone(),
          recreate: if *recreate {
            TerminalRecreateMode::Always
          } else {
            TerminalRecreateMode::DifferentCommand
          },
          mode: None,
        }),
      )
      .await
  })
  .await
}

pub async fn handle_exec(
  Exec {
    server,
    container,
    shell,
    recreate,
  }: &Exec,
) -> anyhow::Result<()> {
  let server = get_server(server.clone(), container).await?;
  handle_terminal_forwarding(
    &format!("{server}/{container}"),
    async {
      super::komodo_client()
        .await?
        .connect_container_terminal(
          server,
          container.to_string(),
          None,
          Some(InitTerminal {
            command: Some(shell.to_string()),
            recreate: if *recreate {
              TerminalRecreateMode::Always
            } else {
              TerminalRecreateMode::DifferentCommand
            },
            mode: Some(ContainerTerminalMode::Exec),
          }),
        )
        .await
    },
  )
  .await
}

pub async fn handle_attach(
  Attach {
    server,
    container,
    recreate,
  }: &Attach,
) -> anyhow::Result<()> {
  let server = get_server(server.clone(), container).await?;
  handle_terminal_forwarding(
    &format!("{server}/{container}-attach"),
    async {
      super::komodo_client()
        .await?
        .connect_container_terminal(
          server,
          container.to_string(),
          None,
          Some(InitTerminal {
            command: None,
            recreate: if *recreate {
              TerminalRecreateMode::Always
            } else {
              TerminalRecreateMode::DifferentCommand
            },
            mode: Some(ContainerTerminalMode::Attach),
          }),
        )
        .await
    },
  )
  .await
}

async fn get_server(
  server: Option<String>,
  container: &str,
) -> anyhow::Result<String> {
  if let Some(server) = server {
    return Ok(server);
  }

  let client = super::komodo_client().await?;

  let mut containers = client
    .read(ListAllDockerContainers {
      servers: Default::default(),
      containers: vec![container.to_string()],
    })
    .await?;

  if containers.is_empty() {
    return Err(anyhow!(
      "Did not find any container matching {container}"
    ));
  }

  if containers.len() == 1 {
    return containers
      .pop()
      .context("Shouldn't happen")?
      .server_id
      .context("Container doesn't have server_id");
  }

  let servers = containers
    .into_iter()
    .flat_map(|container| container.server_id)
    .collect::<Vec<_>>();

  let servers = client
    .read(ListServers {
      query: ServerQuery::builder().names(servers).build(),
    })
    .await?
    .into_iter()
    .map(|server| format!("\t- {}", server.name.bold()))
    .collect::<Vec<_>>()
    .join("\n");

  Err(anyhow!(
    "Multiple containers matching '{}' on Servers:\n{servers}",
    container.bold(),
  ))
}

async fn handle_terminal_forwarding<
  C: Future<Output = anyhow::Result<TerminalWebsocket>>,
>(
  label: &str,
  connect: C,
) -> anyhow::Result<()> {
  // Need to forward multiple sources into ws write
  let (write_tx, mut write_rx) =
    tokio::sync::mpsc::channel::<TerminalStdinMessage>(1024);

  // ================
  //  SETUP RESIZING
  // ================

  // Subscribe to SIGWINCH for resize messages
  let mut sigwinch = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::window_change(),
  )
  .context("failed to register SIGWINCH handler")?;

  // Send first resize messsage, bailing if it fails to get the size.
  write_tx.send(resize_message()?).await?;

  let cancel = CancellationToken::new();

  let forward_resize = async {
    while future_or_cancel(sigwinch.recv(), &cancel)
      .await
      .flatten()
      .is_some()
    {
      if let Ok(resize_message) = resize_message()
        && write_tx.send(resize_message).await.is_err()
      {
        break;
      }
    }
    cancel.cancel();
  };

  let forward_stdin = async {
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 8192];
    while let Some(Ok(n)) =
      future_or_cancel(stdin.read(&mut buf), &cancel).await
    {
      // EOF
      if n == 0 {
        break;
      }
      let bytes = &buf[..n];
      // Check for disconnect sequence (alt + q)
      if bytes == [197, 147] {
        break;
      }
      // Forward bytes
      if write_tx
        .send(TerminalStdinMessage::Forward(bytes.to_vec()))
        .await
        .is_err()
      {
        break;
      };
    }
    cancel.cancel();
  };

  // =====================
  //  CONNECT AND FORWARD
  // =====================

  let (mut ws_write, mut ws_read) = connect.await?.split();

  let forward_write = async {
    while let Some(message) =
      future_or_cancel(write_rx.recv(), &cancel).await.flatten()
    {
      if let Err(e) = ws_write.send_stdin_message(message).await {
        cancel.cancel();
        return Some(e);
      };
    }
    cancel.cancel();
    None
  };

  let forward_read = async {
    let mut stdout = tokio::io::stdout();

    // Write connection message
    if let Err(e) = write_connection_message(&mut stdout, label)
      .await
      .context("Failed to write text to stdout")
    {
      cancel.cancel();
      return Some(e);
    }

    while let Some(msg) =
      future_or_cancel(ws_read.receive_stdout(), &cancel).await
    {
      let bytes = match msg {
        Ok(Some(bytes)) => bytes,
        Ok(None) => break,
        Err(e) => {
          cancel.cancel();
          return Some(e.context("Websocket read error"));
        }
      };
      if let Err(e) = stdout
        .write_all(&bytes)
        .await
        .context("Failed to write text to stdout")
      {
        cancel.cancel();
        return Some(e);
      }
      let _ = stdout.flush().await;
    }
    cancel.cancel();
    None
  };

  let guard = RawModeGuard::enable_raw_mode()?;

  let (_, _, write_error, read_error) = tokio::join!(
    forward_resize,
    forward_stdin,
    forward_write,
    forward_read
  );

  drop(guard);

  if let Some(e) = write_error {
    eprintln!("\nFailed to forward stdin | {e:#}");
  }

  if let Some(e) = read_error {
    eprintln!("\nFailed to forward stdout | {e:#}");
  }

  println!("\n\n{} {}", "connection".bold(), "closed".red().bold());

  // It doesn't seem to exit by itself after the raw mode stuff.
  std::process::exit(0)
}

async fn write_connection_message(
  stdout: &mut tokio::io::Stdout,
  label: &str,
) -> anyhow::Result<()> {
  // Use message without ansi for correct length
  let message_clean = format!("# Connected to {label} (km) #");
  let bounder = "=".repeat(message_clean.chars().count());

  let message = format!(
    "# {} to {} {} #",
    "Connected".green().bold(),
    label.bold(),
    "(km)".dimmed()
  );

  stdout
    .write_all(
      format!("\n{bounder}\r\n{message}\r\n{bounder}\r\n").as_bytes(),
    )
    .await?;
  let _ = stdout.flush().await;

  Ok(())
}

fn resize_message() -> anyhow::Result<TerminalStdinMessage> {
  let (cols, rows) = crossterm::terminal::size()
    .context("Failed to get terminal size")?;
  Ok(TerminalStdinMessage::Resize(TerminalResizeMessage {
    rows,
    cols,
  }))
}

struct RawModeGuard;

impl RawModeGuard {
  fn enable_raw_mode() -> anyhow::Result<Self> {
    crossterm::terminal::enable_raw_mode()
      .context("Failed to enable terminal raw mode")?;
    Ok(Self)
  }
}
impl Drop for RawModeGuard {
  fn drop(&mut self) {
    if let Err(e) = crossterm::terminal::disable_raw_mode() {
      eprintln!("Failed to disable terminal raw mode | {e:?}");
    }
  }
}

async fn future_or_cancel<T, F: Future<Output = T>>(
  fut: F,
  cancel: &CancellationToken,
) -> Option<T> {
  tokio::select! {
    res = fut => Some(res),
    _ = cancel.cancelled() => None
  }
}
