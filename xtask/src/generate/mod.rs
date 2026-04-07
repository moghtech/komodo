mod resource_json_schema;

use crate::XTask;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Generate {
  ResourceJsonSchema(resource_json_schema::ResourceJsonSchema),
}

impl XTask for Generate {
  fn run(self) -> anyhow::Result<()> {
    match self {
      Generate::ResourceJsonSchema(cmd) => cmd.run(),
    }
  }
}
