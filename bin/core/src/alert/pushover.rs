use std::sync::OnceLock;

use super::*;

pub async fn send_alert(
  url: &str,
  alert: &Alert,
) -> anyhow::Result<()> {
  let content = standard_alert_content(alert);
  if content.is_empty() {
    return Ok(());
  }

  let VariablesAndSecrets { variables, secrets } =
    get_variables_and_secrets().await?;
  let mut url_interpolated = url.to_string();

  let mut interpolator =
    Interpolator::new(Some(&variables), &secrets);

  interpolator.interpolate_string(&mut url_interpolated)?;

  send_message(&url_interpolated, content).await.map_err(|e| {
    let replacers = interpolator
      .secret_replacers
      .into_iter()
      .collect::<Vec<_>>();
    let sanitized_error =
      svi::replace_in_string(&format!("{e:?}"), &replacers);
    anyhow::Error::msg(format!(
      "Error with request to Pushover: {sanitized_error}"
    ))
  })
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
