#[derive(Debug, Clone, clap::Parser)]
pub struct Container {
  /// Other container utilities
  #[command(subcommand)]
  pub command: Option<ContainerCommand>,
  /// List all containers, including stopped ones.
  /// This overrides 'down'.
  #[arg(long, short = 'a', default_value_t = false)]
  pub all: bool,
  /// Reverse the ordering of results,
  /// so non-running containers are listed first if --all is passed.
  #[arg(long, short = 'r', default_value_t = false)]
  pub reverse: bool,
  /// List only non-running containers.
  #[arg(long, short = 'd', default_value_t = false)]
  pub down: bool,
  /// Include links. Makes the table very large.
  #[arg(long, short = 'l', default_value_t = false)]
  pub links: bool,
  /// Filter containers by a particular server.
  /// Supports wildcard syntax.
  /// Can be specified multiple times. (alias `s`)
  #[arg(name = "server", long, short = 's')]
  pub servers: Vec<String>,
  /// Filter containers by a name. Supports wildcard syntax.
  /// Can be specified multiple times. (alias `c`)
  #[arg(name = "container", long, short = 'c')]
  pub containers: Vec<String>,
  /// Filter containers by image. Supports wildcard syntax.
  /// Can be specified multiple times. (alias `i`)
  #[arg(name = "image", long, short = 'i')]
  pub images: Vec<String>,
  /// Filter containers by network. Supports wildcard syntax.
  /// Can be specified multiple times. (alias `--net`, `n`)
  #[arg(name = "network", alias = "net", long, short = 'n')]
  pub networks: Vec<String>,
  /// Specify the format of the output.
  #[arg(long, short = 'f', default_value_t = super::CliFormat::Table)]
  pub format: super::CliFormat,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ContainerCommand {
  /// Inspect containers
  #[clap(alias = "i")]
  Inspect(InspectContainer),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct InspectContainer {
  /// The container name. If it matches multiple containers and no server is specified,
  /// each container's inspect info will be logged.
  pub container: String,
  /// Select the particular server that container is on.
  #[arg(name = "server", long, short = 's')]
  pub servers: Vec<String>,
  /// Only show the .State part of the inspect response.
  #[arg(long, short = 'u')]
  pub state: bool,
  /// Only show the .Mounts part of the inspect response.
  #[arg(long, short = 'm')]
  pub mounts: bool,
  /// Only show the .HostConfig part of the inspect response.
  #[arg(long, short = 'f')]
  pub host_config: bool,
  /// Only show the .Config part of the inspect response.
  #[arg(long, short = 'c')]
  pub config: bool,
  /// Only show the .NetworkSettings part of the inspect response.
  #[arg(long, short = 'n')]
  pub network_settings: bool,
}
