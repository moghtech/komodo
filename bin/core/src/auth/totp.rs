use anyhow::Context as _;

pub fn make_totp(
  secret_bytes: Vec<u8>,
  account_name: impl Into<Option<String>>,
) -> anyhow::Result<totp_rs::TOTP> {
  totp_rs::TOTP::new(
    totp_rs::Algorithm::SHA1,
    6,
    1,
    30,
    secret_bytes,
    Some(String::from("Komodo")),
    account_name.into().unwrap_or_default(),
  )
  .context("Failed to construct TOTP")
}
