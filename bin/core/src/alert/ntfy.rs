use std::sync::OnceLock;

use super::*;

#[instrument(level = "debug")]
pub async fn send_alert(
  url: &str,
  email: Option<&str>,
  alert: &Alert,
) -> anyhow::Result<()> {
  let content = standard_alert_content(alert);
  if !content.is_empty() {
    send_message(url, email, content).await?;
  }
  Ok(())
}

async fn send_message(
  url: &str,
  email: Option<&str>,
  content: String,
) -> anyhow::Result<()> {
  let mut request = http_client()
    .post(url)
    .header("Title", "ntfy Alert")
    .body(content);

  if let Some(email) = email {
    request = request.header("X-Email", email);
  }

  let response =
    request.send().await.context("Failed to send message")?;

  let status = response.status();
  if status.is_success() {
    debug!("ntfy alert sent successfully: {}", status);
    Ok(())
  } else {
    let text = response.text().await.with_context(|| {
      format!(
        "Failed to send message to ntfy | {status} | failed to get response text"
      )
    })?;
    Err(anyhow!(
      "Failed to send message to ntfy | {} | {}",
      status,
      text
    ))
  }
}

fn http_client() -> &'static reqwest::Client {
  static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
  CLIENT.get_or_init(reqwest::Client::new)
}
