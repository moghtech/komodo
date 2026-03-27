use crate::entities::resource::TemplatesQueryBehavior;

#[derive(Debug, Clone, clap::Parser)]
pub struct List {
  /// List specific resources
  #[command(subcommand)]
  pub command: Option<ListCommand>,
  /// List all resources, including down ones.
  #[arg(long, short = 'a', default_value_t = false)]
  pub all: bool,
  /// Reverse the ordering of results,
  /// so non-running containers are listed first if --all is passed.
  #[arg(long, short = 'r', default_value_t = false)]
  pub reverse: bool,
  /// List only non-running / non-ok resources.
  #[arg(long, short = 'd', default_value_t = false)]
  pub down: bool,
  /// List only "in progress" / "pending" resources, like Actions / Procedures that are running (alias: `pending`)
  #[arg(
    long,
    short = 'p',
    alias = "pending",
    default_value_t = false
  )]
  pub in_progress: bool,
  /// Include links. Makes the table very large.
  #[arg(long, short = 'l', default_value_t = false)]
  pub links: bool,
  /// Whether to include resources marked as templates in results. Default: 'exclude'.
  #[arg(
    long,
    short = 'm',
    default_value_t = TemplatesQueryBehavior::Exclude,
  )]
  pub templates: TemplatesQueryBehavior,
  /// Filter by a particular name. Supports wildcard.
  /// Can be specified multiple times. (alias `n`)
  #[arg(name = "name", long, short = 'n')]
  pub names: Vec<String>,
  /// Filter by a particular tag.
  /// Can be specified multiple times. (alias `t`)
  #[arg(name = "tag", long, short = 't')]
  pub tags: Vec<String>,
  /// Filter by a particular server. Supports wildcard.
  /// Can be specified multiple times. (alias `s`)
  #[arg(name = "server", long, short = 's')]
  pub servers: Vec<String>,
  /// Filter by a particular builder. Supports wildcard.
  /// Can be specified multiple times. (alias `b`)
  #[arg(name = "builder", long, short = 'b')]
  pub builders: Vec<String>,
  /// Specify the format of the output.
  #[arg(long, short = 'f', default_value_t = super::CliFormat::Table)]
  pub format: super::CliFormat,
}

impl From<List> for ResourceFilters {
  fn from(value: List) -> Self {
    Self {
      all: value.all,
      reverse: value.reverse,
      down: value.down,
      in_progress: value.in_progress,
      links: value.links,
      templates: value.templates,
      names: value.names,
      tags: value.tags,
      servers: value.servers,
      builders: value.builders,
      format: value.format,
    }
  }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ListCommand {
  /// List Servers (aliases: `server`, `sv`)
  #[clap(alias = "server", alias = "sv")]
  Servers(ResourceFilters),
  /// List Stacks (aliases: `stack`, `st`)
  #[clap(alias = "stack", alias = "st")]
  Stacks(ResourceFilters),
  /// List Deployments (aliases: `deployment`, `dp`)
  #[clap(alias = "deployment", alias = "dp")]
  Deployments(ResourceFilters),
  /// List Builds (aliases: `build`, `bd`)
  #[clap(alias = "build", alias = "bd")]
  Builds(ResourceFilters),
  /// List Repos (aliases: `repo`, `rp`)
  #[clap(alias = "repo", alias = "rp")]
  Repos(ResourceFilters),
  /// List Procedures (aliases: `procedure`, `pr`)
  #[clap(alias = "procedure", alias = "pr")]
  Procedures(ResourceFilters),
  /// List Actions (aliases: `action`, `ac`)
  #[clap(alias = "action", alias = "ac")]
  Actions(ResourceFilters),
  /// List Syncs (aliases: `sync`, `sn`)
  #[clap(alias = "sync", alias = "sn")]
  Syncs(ResourceFilters),
  /// List Terminals (aliases: `term`, `tm`)
  #[clap(alias = "term", alias = "tm")]
  Terminals(ResourceFilters),
  /// List scheduled Procedures / Actions (aliases: `sched`, `sc`)
  #[clap(alias = "sched", alias = "sc")]
  Schedules(ResourceFilters),
  /// List Builders (aliases: `builder`, `bldr`)
  #[clap(alias = "builder", alias = "bldr")]
  Builders(ResourceFilters),
  /// List Alerters (aliases: `alerter`, `alrt`)
  #[clap(alias = "alerter", alias = "alrt")]
  Alerters(ResourceFilters),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct ResourceFilters {
  /// List all resources, including down ones.
  #[arg(long, short = 'a', default_value_t = false)]
  pub all: bool,
  /// Reverse the ordering of results,
  /// so non-running containers are listed first if --all is passed.
  #[arg(long, short = 'r', default_value_t = false)]
  pub reverse: bool,
  /// List only non-running / non-ok resources.
  #[arg(long, short = 'd', default_value_t = false)]
  pub down: bool,
  /// List only "in progress" / "pending" resources, like Actions / Procedures that are running (alias: `pending`)
  #[arg(
    long,
    short = 'p',
    alias = "pending",
    default_value_t = false
  )]
  pub in_progress: bool,
  /// Include links. Makes the table very large.
  #[arg(long, short = 'l', default_value_t = false)]
  pub links: bool,
  /// Whether to include resources marked as templates in results. Default: 'exclude'.
  #[arg(
    long,
    short = 'm',
    default_value_t = TemplatesQueryBehavior::Exclude,
  )]
  pub templates: TemplatesQueryBehavior,
  /// Filter by a particular name. Supports wildcard.
  /// Can be specified multiple times. (alias `n`)
  #[arg(name = "name", long, short = 'n')]
  pub names: Vec<String>,
  /// Filter by a particular tag.
  /// Can be specified multiple times. (alias `t`)
  #[arg(name = "tag", long, short = 't')]
  pub tags: Vec<String>,
  /// Filter by a particular server. Supports wildcard.
  /// Can be specified multiple times. (alias `s`)
  #[arg(name = "server", long, short = 's')]
  pub servers: Vec<String>,
  /// Filter by a particular builder. Supports wildcard.
  /// Can be specified multiple times. (alias `b`)
  #[arg(name = "builder", long, short = 'b')]
  pub builders: Vec<String>,
  /// Specify the format of the output.
  #[arg(long, short = 'f', default_value_t = super::CliFormat::Table)]
  pub format: super::CliFormat,
}
