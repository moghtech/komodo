use std::path::PathBuf;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum DatabaseCommand {
  /// Triggers database backup to compressed files
  /// organized by time the backup was taken. (alias: `bkp`)
  #[clap(alias = "bkp")]
  Backup {
    /// Optionally provide a specific backups folder.
    /// Default: `/backups`
    #[arg(long, short = 'f')]
    backups_folder: Option<PathBuf>,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Restores the database from backup files. (alias: `rst`)
  #[clap(alias = "rst")]
  Restore {
    /// Optionally provide a specific backups folder.
    /// Default: `/backups`
    #[arg(long, short = 'f')]
    backups_folder: Option<PathBuf>,
    /// Optionally provide a specific restore folder.
    /// If not provided, will use the most recent backup folder.
    ///
    /// Example: `2025-08-01_05-04-53`
    #[arg(long, short = 'r')]
    restore_folder: Option<PathBuf>,
    /// Whether to index the target database. Default: true
    #[arg(long, short = 'i', default_value_t = true)]
    index: bool,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Prunes database backups if there are greater than
  /// the configured `max_backups` (KOMODO_CLI_MAX_BACKUPS).
  Prune {
    /// Optionally provide a specific backups folder.
    /// Default: `/backups`
    #[arg(long, short = 'f')]
    backups_folder: Option<PathBuf>,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Copy the database to another running database. (alias: `cp`)
  #[clap(alias = "cp")]
  Copy {
    /// The target database uri to copy to.
    #[arg(long)]
    uri: Option<String>,
    /// The target database address to copy to
    #[arg(long, short = 'a')]
    address: Option<String>,
    /// The target database username
    #[arg(long, short = 'u')]
    username: Option<String>,
    /// The target database password
    #[arg(long, short = 'p')]
    password: Option<String>,
    /// The target db name to copy to.
    #[arg(long, short = 'd')]
    db_name: Option<String>,
    /// Whether to index the target database. Default: true
    #[arg(long, short = 'i', default_value_t = true)]
    index: bool,
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
  /// Downgrade the database to V1 compatible data structures.
  V1Downgrade {
    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
}
