use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Args;
use komodo_client::entities::toml::ResourcesToml;
use schemars::{_private::serde_json, generate::SchemaSettings};

use crate::XTask;

#[derive(Debug, Args)]
pub struct ResourceSchema {
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

impl XTask for ResourceSchema {
  fn run(self) -> Result<()> {
    let schema = SchemaSettings::draft07()
      .into_generator()
      .into_root_schema_for::<ResourcesToml>();

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
