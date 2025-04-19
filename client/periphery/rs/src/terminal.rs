use anyhow::Context;
use futures_util::{Stream, TryStreamExt};
use tokio_util::codec::LinesCodecError;

use crate::{
  PeripheryClient, api::terminal::ExecuteTerminal,
  periphery_http_client,
};

impl PeripheryClient {
  /// Executes command on specified terminal,
  /// and streams the response ending in [KOMODO_EXIT_CODE][komodo_client::entities::KOMODO_EXIT_CODE]
  /// sentinal value as the expected final line of the stream.
  ///
  /// Example final line:
  /// ```
  /// __KOMODO_EXIT_CODE__:0
  /// ```
  ///
  /// This means the command exited with code 0 (success).
  ///
  /// If this value is NOT the final item before stream closes, it means
  /// the terminal exited mid command, before giving status. Example: running `exit`.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_terminal(
    &self,
    terminal_name: String,
    command: String,
  ) -> anyhow::Result<TerminalStreamResponse> {
    tracing::trace!(
      "sending request | type: ExecuteTerminal | terminal name: {terminal_name} | command: {command}",
    );
    let req = periphery_http_client()
      .post(format!("{}/terminal/exec", self.address))
      .json(&ExecuteTerminal {
        terminal: terminal_name,
        command,
      })
      .header("authorization", &self.passkey);
    let res =
      req.send().await.context("Failed at request to periphery")?;
    let status = res.status();
    tracing::debug!(
      "got response | type: ExecuteTerminal | {status} | response: {res:?}",
    );
    if status.is_success() {
      Ok(TerminalStreamResponse(res))
    } else {
      tracing::debug!("response is non-200");

      let text = res
        .text()
        .await
        .context("Failed to convert response to text")?;

      tracing::debug!("got response text, deserializing error");

      let error = serror::deserialize_error(text).context(status);

      Err(error)
    }
  }
}

pub struct TerminalStreamResponse(reqwest::Response);

impl TerminalStreamResponse {
  pub fn into_line_stream(
    self,
  ) -> impl Stream<Item = Result<String, LinesCodecError>> {
    tokio_util::codec::FramedRead::new(
      tokio_util::io::StreamReader::new(
        self.0.bytes_stream().map_err(|e| std::io::Error::other(e)),
      ),
      tokio_util::codec::LinesCodec::new(),
    )
  }
}
