use futures_util::{Stream, StreamExt, TryStreamExt};

pub struct TerminalStreamResponse(pub reqwest::Response);

impl TerminalStreamResponse {
  pub fn into_line_stream(
    self,
  ) -> impl Stream<Item = Result<String, tokio_util::codec::LinesCodecError>>
  {
    tokio_util::codec::FramedRead::new(
      tokio_util::io::StreamReader::new(
        self.0.bytes_stream().map_err(std::io::Error::other),
      ),
      tokio_util::codec::LinesCodec::new(),
    )
    .map(|line| line.map(|line| line + "\n"))
  }
}
