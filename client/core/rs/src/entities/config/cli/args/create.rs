#[derive(Debug, Clone, clap::Subcommand)]
pub enum CreateCommand {
  /// Create an API key. (alias: `ak`)
  #[clap(alias = "ak")]
  ApiKey(CreateApiKey),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct CreateApiKey {
  /// Pass optional name for the api key
  pub name: Option<String>,
  /// The user username to create the API key for.
  /// If `--use-api`, this is optional, and will create an api key for a service user.
  /// If NOT `--use-api` (default), this field is REQUIRED.
  #[arg(long = "for", short = 'f')]
  pub for_user: Option<String>,
  /// Pass api key expiry in number of days. Default: Unlimited.
  #[arg(long, short = 'e')]
  pub expires: Option<i64>,
  /// Use the Komodo API rather than direct database connection.
  /// This requires existing km credentials.
  #[arg(long, short = 'a', default_value_t = false)]
  pub use_api: bool,
}
