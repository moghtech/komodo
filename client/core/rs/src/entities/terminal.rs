use anyhow::{Context as _, anyhow};
use derive_variants::EnumVariants;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use tokio_tungstenite::tungstenite;
use typeshare::typeshare;

use crate::entities::I64;

/// Represents an active terminal on a server.
/// Retrieve with [ListTerminals][crate::api::read::server::ListTerminals].
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Terminal {
  /// The name of the terminal.
  pub name: String,
  /// The target resource of the Terminal.
  pub target: TerminalTarget,
  /// The command used to init the shell.
  pub command: String,
  /// The size of the terminal history in memory.
  pub stored_size_kb: f64,
  /// When the Terminal was created.
  /// Unix timestamp milliseconds.
  pub created_at: I64,
}

#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum TerminalTarget {
  Server {
    server: Option<String>,
  },
  Container {
    server: String,
    container: String,
  },
  Stack {
    stack: String,
    service: Option<String>,
  },
  Deployment {
    deployment: String,
  },
}

impl TerminalTarget {
  // Checks for target match in a fixed server context.
  pub fn matches_on_server(&self, other: &TerminalTarget) -> bool {
    match (self, other) {
      (
        TerminalTarget::Server { .. },
        TerminalTarget::Server { .. },
      ) => true,
      (
        TerminalTarget::Container {
          container: target, ..
        },
        TerminalTarget::Container { container, .. },
      ) => target == container,
      (
        TerminalTarget::Stack { stack: target, .. },
        TerminalTarget::Stack { stack, .. },
      ) => target == stack,
      (
        TerminalTarget::Deployment { deployment: target },
        TerminalTarget::Deployment { deployment },
      ) => target == deployment,
      _ => false,
    }
  }
}

/// Specify the container terminal mode (exec or attach)
#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, AsRefStr,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ContainerTerminalMode {
  #[default]
  Exec,
  Attach,
}

/// Configures the behavior of [CreateTerminal] if the
/// specified terminal name already exists.
#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, Serialize, Deserialize, AsRefStr,
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum TerminalRecreateMode {
  /// Never kill the old terminal if it already exists.
  /// If the init command is different, returns error.
  #[default]
  Never,
  /// Always kill the old terminal and create new one
  Always,
  /// Only kill and recreate if the command is different.
  DifferentCommand,
}

/// JSON structure to send new terminal window dimensions
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TerminalResizeMessage {
  pub rows: u16,
  pub cols: u16,
}

#[derive(Debug, Clone)]
pub struct TerminalMessage(Vec<u8>);

impl TerminalMessage {
  /// Suitable to use for PTY stdout -> client messages.
  pub fn from_raw(vec: Vec<u8>) -> Self {
    Self(vec)
  }

  /// Suitable to use for PTY stdout -> client messages.
  pub fn into_raw(self) -> Vec<u8> {
    self.0
  }

  pub fn into_ws_message(self) -> tungstenite::Message {
    tungstenite::Message::Binary(self.0.into())
  }
  /// Message sent from client -> PTY stdin could be
  /// regular bytes, or resize message.
  pub fn into_stdin_message(
    self,
  ) -> anyhow::Result<TerminalStdinMessage> {
    let mut bytes = self.0;
    let variant_byte = bytes.pop().context(
      "Failed to decode Terminal message | bytes are empty",
    )?;
    use TerminalStdinMessageVariant::*;
    match TerminalStdinMessageVariant::from_byte(variant_byte)? {
      Begin => Ok(TerminalStdinMessage::Begin),
      Forward => Ok(TerminalStdinMessage::Forward(bytes)),
      Resize => {
        let message =
          serde_json::from_slice::<TerminalResizeMessage>(&bytes)
            .context("Invalid TerminalResizeMessage bytes")?;
        Ok(TerminalStdinMessage::Resize(message))
      }
    }
  }
}

/// This is message send from clients -> PTY stdin.
#[derive(Debug, EnumVariants)]
#[variant_derive(Debug, Clone, Copy)]
pub enum TerminalStdinMessage {
  /// This triggers forwarding to begin.
  Begin,
  /// Forward these bytes as normal to PTY stdin.
  Forward(Vec<u8>),
  /// Resize the PTY dimensions based on client.
  /// Clients should send this whenever its window resizes.
  Resize(TerminalResizeMessage),
}

impl TerminalStdinMessage {
  pub fn forward(bytes: Vec<u8>) -> Self {
    Self::Forward(bytes)
  }

  pub fn into_terminal_message(
    self,
  ) -> anyhow::Result<TerminalMessage> {
    match self {
      TerminalStdinMessage::Begin => Ok(TerminalMessage(vec![
        TerminalStdinMessageVariant::Begin.as_byte(),
      ])),
      TerminalStdinMessage::Forward(mut bytes) => {
        bytes.push(TerminalStdinMessageVariant::Forward.as_byte());
        Ok(TerminalMessage(bytes))
      }
      TerminalStdinMessage::Resize(message) => {
        let mut bytes = serde_json::to_vec(&message).context(
          "Failed to serialize TerminalResizeMessage to bytes",
        )?;
        bytes.push(TerminalStdinMessageVariant::Resize.as_byte());
        Ok(TerminalMessage(bytes))
      }
    }
  }
}

impl TerminalStdinMessageVariant {
  pub fn from_byte(byte: u8) -> anyhow::Result<Self> {
    use TerminalStdinMessageVariant::*;
    let variant = match byte {
      0x00 => Begin,
      0x01 => Forward,
      0xFF => Resize,
      other => {
        return Err(anyhow!(
          "Got unrecognized TerminalStdinMessageVariant byte: {other}"
        ));
      }
    };
    Ok(variant)
  }

  pub fn as_byte(self) -> u8 {
    use TerminalStdinMessageVariant::*;
    match self {
      Begin => 0x00,
      Forward => 0x01,
      Resize => 0xFF,
    }
  }
}
