use komodo_client::entities::NoData;
use periphery_client::api::terminal::{
  DeleteTerminal, ExecuteTerminal, ListTerminals,
};
use resolver_api::Resolve;
use serror::Json;

use crate::terminal::{
  delete_terminal, list_terminals, run_command_on_terminal,
};

impl Resolve<super::Args> for ListTerminals {
  #[instrument(name = "ListTerminals", level = "debug")]
  async fn resolve(
    self,
    _: &super::Args,
  ) -> serror::Result<Vec<String>> {
    Ok(list_terminals())
  }
}

impl Resolve<super::Args> for DeleteTerminal {
  #[instrument(name = "DeleteTerminal", level = "debug")]
  async fn resolve(self, _: &super::Args) -> serror::Result<NoData> {
    delete_terminal(&self.terminal);
    Ok(NoData {})
  }
}

pub async fn exec(
  Json(ExecuteTerminal {
    terminal: name,
    command,
  }): Json<ExecuteTerminal>,
) -> serror::Result<axum::body::Body> {
  let stdout = run_command_on_terminal(name, command).await?;
  Ok(axum::body::Body::from_stream(stdout))
}
