use komodo_client::entities::NoData;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<String>)]
#[error(serror::Error)]
pub struct ListPtys {}

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(NoData)]
#[error(serror::Error)]
pub struct DeletePty {
  pub pty: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectPtyQuery {
  /// Each periphery can keep multiple ptys open.
  /// If a ptys with the specified name already exists,
  /// it will be attached to.
  /// Otherwise a new pty will be created for the command,
  /// which will persist until it is deleted using [DeletePty]
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
