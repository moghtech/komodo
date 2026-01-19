use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, OnceLock},
};

use anyhow::{Context, anyhow};
use arc_swap::ArcSwap;
use cache::{CloneCache, CloneVecCache};
use komodo_client::entities::{
  docker::container::ContainerStats, terminal::TerminalStdinMessage,
};
use mogh_pki::{PkiKind, RotatableKeyPair, SpkiPublicKey};
use periphery_client::transport::EncodedTransportMessage;
use tokio::sync::{Mutex, OnceCell, RwLock, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use transport::channel::BufferedChannel;
use uuid::Uuid;

use crate::{
  config::periphery_config, docker::DockerClient,
  helpers::resolve_host_public_ip, stats::StatsClient,
  terminal::PeripheryTerminal,
};

/// Should call in startup to ensure Periphery errors without valid private key.
pub fn periphery_keys() -> &'static RotatableKeyPair {
  static PERIPHERY_KEYS: OnceLock<RotatableKeyPair> = OnceLock::new();
  PERIPHERY_KEYS.get_or_init(|| {
    let config = periphery_config();
    if let Some(private_key_spec) = config.private_key.as_deref() {
      RotatableKeyPair::from_private_key_spec(
        PkiKind::Mutual,
        private_key_spec,
      )
    } else {
      RotatableKeyPair::from_private_key_spec(
        PkiKind::Mutual,
        &format!(
          "file:{}",
          config
            .root_directory
            .join("keys/periphery.key")
            .to_str()
            .expect("Invalid root directory")
        ),
      )
    }
    .unwrap()
  })
}

pub fn core_public_keys() -> &'static CorePublicKeys {
  static CORE_PUBLIC_KEYS: OnceLock<CorePublicKeys> = OnceLock::new();
  CORE_PUBLIC_KEYS.get_or_init(CorePublicKeys::default)
}

pub struct CorePublicKeys {
  keys: ArcSwap<Vec<SpkiPublicKey>>,
  /// If any keys fail to write, store them here.
  /// For Periphery -> Core connection, Periphery will
  /// write the Core pub keys to these files as they connect.
  to_write: ArcSwap<Vec<PathBuf>>,
}

impl Default for CorePublicKeys {
  fn default() -> Self {
    let keys = CorePublicKeys {
      keys: Default::default(),
      to_write: Default::default(),
    };
    keys.refresh();
    keys
  }
}

impl CorePublicKeys {
  pub fn load(&self) -> arc_swap::Guard<Arc<Vec<SpkiPublicKey>>> {
    self.keys.load()
  }

  pub async fn is_valid(&self, public_key: &str) -> bool {
    // For Periphery -> Core connection, maybe init
    // Core public key file if it doesn't exist.
    self.maybe_write(public_key).await;
    let keys = self.keys.load();
    keys.is_empty() || keys.iter().any(|pk| pk.as_str() == public_key)
  }

  async fn maybe_write(&self, public_key: &str) {
    let to_write = self.to_write.load();
    let path = match to_write.as_slice() {
      // Do nothing if empty
      [] => return,
      [path, _rest @ ..] => path,
    };
    let public_key = match SpkiPublicKey::from_maybe_pem(public_key) {
      Ok(public_key) => public_key,
      Err(e) => {
        error!("Invalid incoming public key | {e:#}");
        return;
      }
    };
    // Check equality at path again before trying to rewrite.
    match SpkiPublicKey::from_file(path) {
      Ok(existing) if existing == public_key => {
        self.refresh();
        return;
      }
      _ => {}
    }
    if let Err(e) = public_key.write_pem_async(path).await {
      warn!("Failed to pin incoming public key | {e:#}");
    }
    self.refresh();
  }

  pub fn refresh(&self) {
    let config = periphery_config();
    let Some(core_public_keys_spec) = config.core_public_keys_spec()
    else {
      return;
    };
    let mut to_write = Vec::new();
    let core_public_keys = core_public_keys_spec
      .iter()
      .flat_map(|public_key| {
        if let Some(path) = public_key.strip_prefix("file:")
        {
          match (SpkiPublicKey::from_file(path), config.server_enabled()) {
            (Ok(public_key), _) => Some(public_key),
            (Err(e), false) => {
              // If only outbound connections, only warn.
              // It will be written when Core public key received.
              warn!("{e:#}");
              to_write.push(path.into());
              None
            }
            (Err(e), true) => {
              // This is too dangerous to allow if server_enabled.
              error!("{e:#}");
              std::process::exit(1)
            }
          }
        } else {
          SpkiPublicKey::from_maybe_pem(public_key)
            .context("Invalid hardcoded public key. If this is supposed to point to file, add 'file:' prefix.")
            .inspect_err(|e| {
              error!("{e:#}");
              std::process::exit(1)
            })
            .ok()
        }
      })
      .collect::<Vec<_>>();
    self.keys.store(Arc::new(core_public_keys));
    self.to_write.store(Arc::new(to_write));
  }
}

/// Core Address / Host -> Channel
pub type CoreConnection = BufferedChannel<EncodedTransportMessage>;
pub type CoreConnections = CloneCache<String, Arc<CoreConnection>>;

pub fn core_connections() -> &'static CoreConnections {
  static CORE_CONNECTIONS: OnceLock<CoreConnections> =
    OnceLock::new();
  CORE_CONNECTIONS.get_or_init(Default::default)
}

pub fn stats_client() -> &'static RwLock<StatsClient> {
  static STATS_CLIENT: OnceLock<RwLock<StatsClient>> =
    OnceLock::new();
  STATS_CLIENT.get_or_init(|| RwLock::new(StatsClient::default()))
}

pub fn terminals() -> &'static CloneVecCache<Arc<PeripheryTerminal>> {
  static TERMINALS: OnceLock<CloneVecCache<Arc<PeripheryTerminal>>> =
    OnceLock::new();
  TERMINALS.get_or_init(Default::default)
}

#[derive(Default)]
pub struct TerminalChannels(CloneCache<Uuid, Arc<TerminalChannel>>);

impl TerminalChannels {
  pub async fn get(
    &self,
    channel: &Uuid,
  ) -> Option<Arc<TerminalChannel>> {
    self.0.get(channel).await
  }

  pub async fn insert(
    &self,
    channel: Uuid,
    terminal: Arc<TerminalChannel>,
  ) -> Option<Arc<TerminalChannel>> {
    self.0.insert(channel, terminal).await
  }

  pub async fn remove(&self, channel: &Uuid) {
    let Some(channel) = self.0.remove(channel).await else {
      return;
    };
    channel.cancel.cancel();
  }
}

pub fn terminal_channels() -> &'static TerminalChannels {
  static TERMINAL_CHANNELS: OnceLock<TerminalChannels> =
    OnceLock::new();
  TERMINAL_CHANNELS.get_or_init(Default::default)
}

#[derive(Debug)]
pub struct TerminalChannel {
  pub sender: mpsc::Sender<TerminalStdinMessage>,
  pub cancel: CancellationToken,
}

pub fn terminal_triggers() -> &'static TerminalTriggers {
  static TERMINAL_TRIGGERS: OnceLock<TerminalTriggers> =
    OnceLock::new();
  TERMINAL_TRIGGERS.get_or_init(Default::default)
}

/// Periphery must wait for Core to finish setting
/// up channel forwarding before sending message,
/// or the first sent messages may be missed.
#[derive(Default)]
pub struct TerminalTriggers(CloneCache<Uuid, Arc<TerminalTrigger>>);

impl TerminalTriggers {
  #[instrument("InsertTerminalTrigger", skip(self))]
  pub async fn insert(&self, channel: Uuid) {
    let (sender, receiver) = oneshot::channel();
    let trigger = Arc::new(TerminalTrigger {
      sender: Some(sender).into(),
      receiver: Some(receiver).into(),
    });
    self.0.insert(channel, trigger).await;
  }

  pub async fn send(&self, channel: &Uuid) -> anyhow::Result<()> {
    let trigger = self.0.get(channel).await.with_context(|| {
      format!("No trigger found for channel {channel}")
    })?;
    trigger.send().await
  }

  pub async fn recv(&self, channel: &Uuid) -> anyhow::Result<()> {
    let trigger = self.0.get(channel).await.with_context(|| {
      format!("No trigger found for channel {channel}")
    })?;
    trigger.wait().await?;
    self.0.remove(channel).await;
    Ok(())
  }
}

#[derive(Debug)]
pub struct TerminalTrigger {
  sender: Mutex<Option<oneshot::Sender<()>>>,
  receiver: Mutex<Option<oneshot::Receiver<()>>>,
}

impl TerminalTrigger {
  /// This consumes the Trigger Sender.
  pub async fn send(&self) -> anyhow::Result<()> {
    let mut sender = self.sender.lock().await;
    let sender = sender
      .take()
      .context("Called TerminalTrigger 'send' more than once.")?;
    sender
      .send(())
      .map_err(|_| anyhow!("TerminalTrigger sender already used"))
  }

  /// This consumes the Trigger Receiver.
  pub async fn wait(&self) -> anyhow::Result<()> {
    let mut receiver = self.receiver.lock().await;
    let receiver = receiver
      .take()
      .context("Called TerminalTrigger 'wait' more than once.")?;
    receiver.await.context("Failed to receive TerminalTrigger")
  }
}

pub fn docker_client() -> &'static SwappableDockerClient {
  static DOCKER_CLIENT: OnceLock<SwappableDockerClient> =
    OnceLock::new();
  DOCKER_CLIENT.get_or_init(SwappableDockerClient::init)
}

#[derive(Default)]
pub struct SwappableDockerClient(ArcSwap<Option<DockerClient>>);

impl SwappableDockerClient {
  pub fn init() -> Self {
    let docker = DockerClient::connect()
      // Only logs on first init, although keeps trying to connect
      .inspect_err(|e| warn!("{e:#}"))
      .ok();
    Self(ArcSwap::new(Arc::new(docker)))
  }

  pub fn load(&self) -> arc_swap::Guard<Arc<Option<DockerClient>>> {
    let res = self.0.load();
    if res.is_some() {
      return res;
    }
    self.reload();
    self.0.load()
  }

  pub fn reload(&self) {
    self.0.store(Arc::new(DockerClient::connect().ok()));
  }
}

pub type ContainerStatsMap = HashMap<String, ContainerStats>;

pub fn container_stats() -> &'static ArcSwap<ContainerStatsMap> {
  static CONTAINER_STATS: OnceLock<ArcSwap<ContainerStatsMap>> =
    OnceLock::new();
  CONTAINER_STATS.get_or_init(Default::default)
}

pub async fn host_public_ip() -> Option<&'static String> {
  static PUBLIC_IPS: OnceCell<Option<String>> = OnceCell::const_new();
  PUBLIC_IPS
    .get_or_init(|| async {
      resolve_host_public_ip()
        .await
        .inspect_err(|e| {
          warn!("Failed to resolve host public ip | {e:#}")
        })
        .ok()
    })
    .await
    .as_ref()
}
