#[derive(Debug, Clone, clap::Subcommand)]
pub enum UpdateCommand {
  /// Update a Build's configuration. (alias: `bld`)
  #[clap(alias = "bld")]
  Build(UpdateResource),
  /// Update a Deployments's configuration. (alias: `dep`)
  #[clap(alias = "dep")]
  Deployment(UpdateResource),
  /// Update a Repos's configuration.
  Repo(UpdateResource),
  /// Update a Servers's configuration. (alias: `srv`)
  #[clap(alias = "srv")]
  Server(UpdateResource),
  /// Update a Stacks's configuration. (alias: `stk`)
  #[clap(alias = "stk")]
  Stack(UpdateResource),
  /// Update a Syncs's configuration.
  Sync(UpdateResource),
  /// Update a Variable's value. (alias: `var`)
  #[clap(alias = "var")]
  Variable {
    /// The name of the variable.
    name: String,
    /// The value to set variable to.
    value: String,
    /// Whether the value should be set to secret.
    /// If unset, will leave the variable secret setting as-is.
    #[arg(long, short = 's')]
    secret: Option<bool>,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Update a user's configuration, including assigning resetting password and assigning Super Admin
  User {
    /// The user to update
    username: String,
    #[command(subcommand)]
    command: UpdateUserCommand,
  },
}

#[derive(Debug, Clone, clap::Parser)]
pub struct UpdateResource {
  /// The name / id of the Resource.
  pub resource: String,
  /// The update string, parsed using 'https://docs.rs/serde_qs/latest/serde_qs'.
  ///
  /// The fields can be found here: 'https://docs.rs/komodo_client/latest/komodo_client/entities/sync/struct.ResourceSyncConfig.html'
  ///
  /// Example: `km update build example-build "branch=testing"`
  ///
  /// Note. Should be enclosed in single or double quotes.
  /// Values containing complex characters (like URLs)
  /// will need to be url-encoded in order to be parsed correctly.
  pub update: String,
  /// Always continue on user confirmation prompts.
  #[arg(long, short = 'y', default_value_t = false)]
  pub yes: bool,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum UpdateUserCommand {
  /// Update the users password. Fails if user is not "Local" user (ie OIDC). (alias: `pw`)
  #[clap(alias = "pw")]
  Password {
    /// The new password to use.
    password: String,
    /// Whether to print unsanitized config,
    /// including sensitive credentials.
    #[arg(long, action)]
    unsanitized: bool,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Un/assign super admin to user. (aliases: `supa`, `sa`)
  #[clap(alias = "supa", alias = "sa")]
  SuperAdmin {
    #[clap(default_value_t = super::CliEnabled::Yes)]
    enabled: super::CliEnabled,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Clears user 2fa factors in case user loses access to them.
  Clear2fa {
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
}
