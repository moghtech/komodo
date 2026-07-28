use std::time::Duration;

use anyhow::anyhow;
use command::{CommandOptions, run_standard_command};

/// Returns error if git not installed
pub async fn check_installed() -> anyhow::Result<()> {
  if run_standard_command(
    "which git",
    CommandOptions::default().timeout(Duration::from_secs(1)),
  )
  .await
  .success()
  {
    Ok(())
  } else {
    Err(anyhow!(
      "Failed: 'git' is not installed or available on $PATH"
    ))
  }
}
