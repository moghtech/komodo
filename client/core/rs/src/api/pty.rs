use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// Connect to a pty (interactive shell) on the given server.
/// TODO: Document calling.
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectPtyQuery {
  /// Server Id or name
  pub server: String,
  /// Each periphery can keep multiple ptys open.
  /// If a ptys with the specified name already exists,
  /// it will be attached to.
  /// Otherwise a new pty will be created for the command,
  /// which will persist until it is deleted using
  /// [DeletePty][crate::api::write::server::DeletePty]
  pub pty: String,
  /// The shell to use, eg. 'sh', 'bash', 'zsh', etc
  #[serde(default = "default_shell")]
  pub shell: String,
  /// Optional. The initial command to execute on connection to the shell.
  pub command: Option<String>,
}

fn default_shell() -> String {
  String::from("bash")
}
