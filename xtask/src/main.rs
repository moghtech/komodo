mod generate;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
  #[command(subcommand)]
  Generate(generate::Generate),
}

trait XTask {
  fn run(self) -> Result<()>;
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Generate(cmd) => cmd.run(),
  }
}
