#[derive(Debug, Clone, clap::Subcommand)]
pub enum CreateCommand {
  /// Create an API key.
  ApiKey(CreateApiKey),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct CreateApiKey {
  /// Pass optional name for the api key
  pub name: Option<String>,
  /// The user username to create the API key for.
  #[arg(long = "for", short = 'f')]
  pub for_user: String,
  /// Pass api key expiry in number of days. Default: Unlimited.
  #[arg(long, short = 'e')]
  pub expires: Option<i64>,
}
