use std::sync::OnceLock;

use super::*;

#[instrument(level = "debug")]
pub async fn send_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let content = standard_alert_content(alert);
  if !content.is_empty() {
    send_message(url, content).await?;
  }
  Ok(())
}

async fn send_message(
  url: &str,
  content: String,
) -> anyhow::Result<()> {
  // pushover needs all information to be encoded in the URL. At minimum they need
  // the user key, the application token, and the message (url encoded).
  // other optional params here: https://pushover.net/api (just add them to the
  // webhook url along with the application token and the user key).
  let content = [("message", content)];

  let response = http_client()
    .post(url)
    .form(&content)
    .send()
    .await
    .context("Failed to send message")?;

  let status = response.status();
  if status.is_success() {
    debug!("pushover alert sent successfully: {}", status);
    Ok(())
  } else {
    let text = response.text().await.with_context(|| {
      format!(
        "Failed to send message to pushover | {status} | failed to get response text"
      )
    })?;
    Err(anyhow!(
      "Failed to send message to pushover | {} | {}",
      status,
      text
    ))
  }
}

fn http_client() -> &'static reqwest::Client {
  static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
  CLIENT.get_or_init(reqwest::Client::new)
}
