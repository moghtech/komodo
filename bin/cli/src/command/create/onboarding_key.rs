use anyhow::Context as _;
use komodo_client::entities::{
  config::cli::args::create::CreateOnboardingKey, komodo_timestamp,
};

pub async fn create(
  CreateOnboardingKey {
    name,
    expires,
    private_key,
    tags,
    privileged,
    copy_server,
    create_builder,
  }: &CreateOnboardingKey,
) -> anyhow::Result<()> {
  let expires = if let Some(expires_days) = expires {
    // now + expires in ms
    komodo_timestamp() + expires_days * 24 * 60 * 60 * 1000
  } else {
    0
  };

  // USE API
  let client = crate::command::komodo_client().await?;

  let key = client
    .write(komodo_client::api::write::CreateOnboardingKey {
      name: name.clone().unwrap_or_default(),
      expires,
      private_key: private_key.clone(),
      tags: tags.clone(),
      privileged: *privileged,
      copy_server: copy_server.clone().unwrap_or_default(),
      create_builder: *create_builder,
    })
    .await?;

  println!(
    "{}",
    serde_json::to_string_pretty(&key)
      .context("Failed to serialize onboarding key to JSON")?
  );

  Ok(())
}
