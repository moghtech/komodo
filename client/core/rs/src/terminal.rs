use anyhow::{Context, anyhow};
use futures::{Stream, TryStreamExt};
use serror::deserialize_error;
use tokio_util::codec::LinesCodecError;

use crate::{KomodoClient, api::terminal::ExecuteTerminal};

impl KomodoClient {
  pub async fn execute_terminal(
    &self,
    body: ExecuteTerminal,
  ) -> anyhow::Result<TerminalStreamResponse> {
    let req = self
      .reqwest
      .post(format!("{}/terminal", self.address))
      .header("x-api-key", &self.key)
      .header("x-api-secret", &self.secret)
      .header("content-type", "application/json")
      .json(&body);
    let res =
      req.send().await.context("failed to reach Komodo API")?;
    let status = res.status();
    if status.is_success() {
      Ok(TerminalStreamResponse(res))
    } else {
      match res.text().await {
        Ok(res) => Err(deserialize_error(res).context(status)),
        Err(e) => Err(anyhow!("{e:?}").context(status)),
      }
    }
  }
}

pub struct TerminalStreamResponse(pub reqwest::Response);

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
