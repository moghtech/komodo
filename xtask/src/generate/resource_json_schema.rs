use crate::XTask;
use anyhow::Result;
use clap::Args;
use komodo_client::entities::toml::ResourcesToml;
use schemars::_private::serde_json;
use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ResourceJsonSchema {
  #[clap(long)]
  pretty: bool,
  #[clap(flatten)]
  output: Output,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct Output {
  #[clap(long)]
  stdout: bool,

  #[clap(long)]
  file: Option<PathBuf>,
}

impl XTask for ResourceJsonSchema {
  fn run(self) -> Result<()> {
    let schema = schema_for!(ResourcesToml);

    let schema_data = if self.pretty {
      serde_json::to_string_pretty(&schema)?
    } else {
      serde_json::to_string(&schema)?
    };

    if self.output.stdout {
      println!("{schema_data}")
    } else if let Some(file) = self.output.file {
      fs::write(&file, schema_data)?;
    }

    Ok(())
  }
}
