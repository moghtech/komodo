use std::io::Read;

use anyhow::{Context, anyhow};
use chrono::TimeZone;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Table};
use komodo_client::{
  KomodoClient,
  entities::config::cli::{CliTableBorders, args::CliFormat},
};
use serde::Serialize;
use tokio::sync::OnceCell;
use wildcard::Wildcard;

use crate::config::cli_config;

pub mod container;
pub mod database;
pub mod execute;
pub mod list;
pub mod update;

async fn komodo_client() -> anyhow::Result<&'static KomodoClient> {
  static KOMODO_CLIENT: OnceCell<KomodoClient> =
    OnceCell::const_new();
  KOMODO_CLIENT
    .get_or_try_init(|| async {
      let config = cli_config();
      let (Some(key), Some(secret)) =
        (&config.cli_key, &config.cli_secret)
      else {
        return Err(anyhow!(
          "Must provide both cli_key and cli_secret"
        ));
      };
      KomodoClient::new(&config.host, key, secret)
        .with_healthcheck()
        .await
    })
    .await
}

fn wait_for_enter(
  press_enter_to: &str,
  skip: bool,
) -> anyhow::Result<()> {
  if skip {
    println!();
    return Ok(());
  }
  println!(
    "\nPress {} to {}\n",
    "ENTER".green(),
    press_enter_to.bold()
  );
  let buffer = &mut [0u8];
  std::io::stdin()
    .read_exact(buffer)
    .context("failed to read ENTER")?;
  Ok(())
}

/// Sanitizes uris of the form:
/// `protocol://username:password@address`
fn sanitize_uri(uri: &str) -> String {
  // protocol: `mongodb`
  // credentials_address: `username:password@address`
  let Some((protocol, credentials_address)) = uri.split_once("://")
  else {
    // If no protocol, return as-is
    return uri.to_string();
  };

  // credentials: `username:password`
  let Some((credentials, address)) =
    credentials_address.split_once('@')
  else {
    // If no credentials, return as-is
    return uri.to_string();
  };

  match credentials.split_once(':') {
    Some((username, _)) => {
      format!("{protocol}://{username}:*****@{address}")
    }
    None => {
      format!("{protocol}://*****@{address}")
    }
  }
}

fn print_items<T: PrintTable + Serialize>(
  items: Vec<T>,
  format: CliFormat,
  links: bool,
) -> anyhow::Result<()> {
  match format {
    CliFormat::Table => {
      let mut table = Table::new();
      let preset = {
        use comfy_table::presets::*;
        match cli_config().table_borders {
          None | Some(CliTableBorders::Horizontal) => {
            UTF8_HORIZONTAL_ONLY
          }
          Some(CliTableBorders::Vertical) => UTF8_FULL_CONDENSED,
          Some(CliTableBorders::Inside) => UTF8_NO_BORDERS,
          Some(CliTableBorders::Outside) => UTF8_BORDERS_ONLY,
          Some(CliTableBorders::All) => UTF8_FULL,
        }
      };
      table.load_preset(preset).set_header(
        T::header(links)
          .iter()
          .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
      );
      for item in items {
        table.add_row(item.row(links));
      }
      println!("{table}");
    }
    CliFormat::Json => {
      println!(
        "{}",
        serde_json::to_string_pretty(&items)
          .context("Failed to serialize items to JSON")?
      );
    }
  }
  Ok(())
}

trait PrintTable {
  fn header(links: bool) -> &'static [&'static str];
  fn row(self, links: bool) -> Vec<Cell>;
}

fn parse_wildcards(items: &[String]) -> Vec<Wildcard<'_>> {
  items
    .iter()
    .flat_map(|i| {
      Wildcard::new(i.as_bytes()).inspect_err(|e| {
        warn!("Failed to parse wildcard: {i} | {e:?}")
      })
    })
    .collect::<Vec<_>>()
}

fn matches_wildcards(
  wildcards: &[Wildcard<'_>],
  items: &[&str],
) -> bool {
  if wildcards.is_empty() {
    return true;
  }
  items.iter().any(|item| {
    wildcards.iter().any(|wc| wc.is_match(item.as_bytes()))
  })
}

fn format_timetamp(ts: i64) -> anyhow::Result<String> {
  let ts = chrono::Local
    .timestamp_millis_opt(ts)
    .single()
    .context("Invalid ts")?
    .format("%m/%d %H:%M:%S")
    .to_string();
  Ok(ts)
}

fn clamp_sha(maybe_sha: &str) -> String {
  if maybe_sha.starts_with("sha256:") {
    maybe_sha[0..20].to_string() + "..."
  } else {
    maybe_sha.to_string()
  }
}

// fn text_link(link: &str, text: &str) -> String {
//   format!("\x1b]8;;{link}\x07{text}\x1b]8;;\x07")
// }
