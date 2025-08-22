//! Module for parsing the Komodo CLI arguments

use std::path::PathBuf;

use crate::api::execute::Execution;

pub mod container;
pub mod database;
pub mod list;
pub mod update;

#[derive(Debug, clap::Parser)]
#[command(name = "komodo-cli", version, about = "", author)]
pub struct CliArgs {
  /// The command to run
  #[command(subcommand)]
  pub command: Command,

  /// Choose a custom [[profile]] name / alias set in a `komodo.cli.toml` file.
  #[arg(long, short = 'p')]
  pub profile: Option<String>,

  /// Sets the path of a config file or directory to use.
  /// Can use multiple times
  #[arg(long, short = 'c')]
  pub config_path: Option<Vec<PathBuf>>,

  /// Sets the keywords to match directory cli config file names on.
  /// Supports wildcard syntax.
  /// Can use multiple times to match multiple patterns independently.
  #[arg(long, short = 'm')]
  pub config_keyword: Option<Vec<String>>,

  /// Whether to debug print on configuration load (on startup)
  #[arg(alias = "debug", long, short = 'd')]
  pub debug_startup: Option<bool>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
  /// Print the CLI config being used. (aliases: `cfg`, `cf`)
  #[clap(alias = "cfg", alias = "cf")]
  Config {
    /// Whether to print the additional profiles picked up
    #[arg(long, short = 'a', default_value_t = false)]
    all_profiles: bool,

    /// Whether to print unsanitized config,
    /// including sensitive credentials.
    #[arg(long, action)]
    unsanitized: bool,
  },

  /// Container info (aliases: `ps`, `cn`, `containers`)
  #[clap(alias = "ps", alias = "cn", alias = "containers")]
  Container(container::Container),

  /// Inspect containers (alias: `i`)
  #[clap(alias = "i")]
  Inspect(container::InspectContainer),

  /// List Komodo resources (aliases: `ls`, `resources`)
  #[clap(alias = "ls", alias = "resources")]
  List(list::List),

  /// Run Komodo executions. (aliases: `x`, `run`, `deploy`, `dep`, `send`)
  #[clap(
    alias = "x",
    alias = "run",
    alias = "deploy",
    alias = "dep",
    alias = "send"
  )]
  Execute(Execute),

  /// Update resource configuration. (alias: `set`)
  #[clap(alias = "set")]
  Update {
    #[command(subcommand)]
    command: update::UpdateCommand,
  },

  /// Database utilities. (alias: `db`)
  #[clap(alias = "db")]
  Database {
    #[command(subcommand)]
    command: database::DatabaseCommand,
  },
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Execute {
  /// The execution to run.
  #[command(subcommand)]
  pub execution: Execution,
  /// Top priority Komodo host.
  /// Eg. "https://demo.komo.do"
  #[arg(long, short = 'a')]
  pub host: Option<String>,
  /// Top priority api key.
  #[arg(long, short = 'k')]
  pub key: Option<String>,
  /// Top priority api secret.
  #[arg(long, short = 's')]
  pub secret: Option<String>,
  /// Always continue on user confirmation prompts.
  #[arg(long, short = 'y', default_value_t = false)]
  pub yes: bool,
}

#[derive(
  Debug, Clone, Copy, Default, strum::Display, clap::ValueEnum,
)]
#[strum(serialize_all = "lowercase")]
pub enum CliFormat {
  /// Table output format. Default. (alias: `t`)
  #[default]
  #[clap(alias = "t")]
  Table,
  /// Json output format. (alias: `j`)
  #[clap(alias = "j")]
  Json,
}

#[derive(
  Debug, Clone, Copy, Default, clap::ValueEnum, strum::Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum CliEnabled {
  #[default]
  #[clap(alias = "y", alias = "true", alias = "t")]
  Yes,
  #[clap(alias = "n", alias = "false", alias = "f")]
  No,
}

impl From<CliEnabled> for bool {
  fn from(value: CliEnabled) -> Self {
    match value {
      CliEnabled::Yes => true,
      CliEnabled::No => false,
    }
  }
}
