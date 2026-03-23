#[derive(Debug, Clone, clap::Subcommand)]
pub enum CreateCommand {
  /// Create a new api key. (alias: `ak`)
  #[clap(alias = "ak")]
  ApiKey(CreateApiKey),
  /// Create a new onboarding key. (alias: `ok`)
  #[clap(alias = "ok")]
  OnboardingKey(CreateOnboardingKey),
}

/// Create a new API key.
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

/// Create a new onboarding key.
#[derive(Debug, Clone, clap::Parser)]
pub struct CreateOnboardingKey {
  /// Pass optional name for the onboarding key
  pub name: Option<String>,
  /// Pass onboarding key expiry in number of days. Default: Unlimited.
  #[arg(long, short = 'e')]
  pub expires: Option<i64>,
  /// Pass an existing private key, otherwise one will be generated. (alias: `pk`)
  #[arg(long, alias = "pk", short = 'k')]
  pub private_key: Option<String>,
  /// Add creation tags to onboarding key.
  /// Can be specified multiple times. (alias `t`)
  #[arg(name = "tag", long, short = 't')]
  pub tags: Vec<String>,
  /// Make the onboarding key privileged.
  #[arg(long, short = 'p', default_value_t = false)]
  pub privileged: bool,
  /// Optional. Onboarded Servers copy this Server's config. (aliases: `copy`, `server`)
  #[arg(long, alias = "copy", alias = "server", short = 's')]
  pub copy_server: Option<String>,
  /// Optional. Whether to also create a Builder for the Server. (alias: `builder`)
  #[arg(
    long,
    alias = "builder",
    short = 'b',
    default_value_t = false
  )]
  pub create_builder: bool,
}
