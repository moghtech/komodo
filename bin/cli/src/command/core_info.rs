use anyhow::Context as _;
use komodo_client::api::read::GetCoreInfo;

pub async fn handle() -> anyhow::Result<()> {
  let client = super::komodo_client().await?;
  let info = client.read(GetCoreInfo {}).await?;
  println!(
    "{}",
    serde_json::to_string_pretty(&info)
      .context("Failed to serialize core info to JSON")?
  );
  Ok(())
}
