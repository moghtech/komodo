use std::time::Duration;

use anyhow::Context as _;
use futures_util::FutureExt as _;
use pin_project_lite::pin_project;

pin_project! {
  pub struct MaybeWithTimeout<F> {
    #[pin]
    inner: F,
  }
}

impl<F> MaybeWithTimeout<F> {
  pub fn new(inner: F) -> MaybeWithTimeout<F> {
    MaybeWithTimeout { inner }
  }
}

impl<F: Future> Future for MaybeWithTimeout<F> {
  type Output = F::Output;
  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    let mut inner = self.project().inner;
    inner.as_mut().poll(cx)
  }
}

impl<
  O,
  E: Into<anyhow::Error>,
  F: Future<Output = Result<O, E>> + Send,
> MaybeWithTimeout<F>
{
  pub fn with_timeout(
    self,
    timeout: Duration,
  ) -> impl Future<Output = anyhow::Result<O>> + Send {
    tokio::time::timeout(timeout, self.inner).map(|res| {
      res
        .context("Timed out waiting for message.")
        .and_then(|inner| inner.map_err(Into::into))
    })
  }
}
