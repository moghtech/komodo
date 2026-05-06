mod resource_schema;

use crate::XTask;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Generate {
  ResourceSchema(resource_schema::ResourceSchema),
}

impl XTask for Generate {
  fn run(self) -> anyhow::Result<()> {
    match self {
      Generate::ResourceSchema(cmd) => cmd.run(),
    }
  }
}
