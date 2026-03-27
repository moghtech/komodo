use std::path::Path;

use anyhow::Context;
use colored::Colorize;
use database::mungos::mongodb::bson::{Document, doc};
use komodo_client::entities::{
  config::cli::args::database::DatabaseCommand, optional_string,
};

use crate::{command::sanitize_uri, config::cli_config};

pub async fn handle(command: &DatabaseCommand) -> anyhow::Result<()> {
  match command {
    DatabaseCommand::Backup { yes, .. } => backup(*yes).await,
    DatabaseCommand::Restore {
      restore_folder,
      index,
      yes,
      ..
    } => restore(restore_folder.as_deref(), *index, *yes).await,
    DatabaseCommand::Prune { yes, .. } => prune(*yes).await,
    DatabaseCommand::Copy { yes, index, .. } => {
      copy(*index, *yes).await
    }
    DatabaseCommand::V1Downgrade { yes } => v1_downgrade(*yes).await,
  }
}

async fn backup(yes: bool) -> anyhow::Result<()> {
  let config = cli_config();

  println!(
    "\n🦎  {} Database {} Utility  🦎",
    "Komodo".bold(),
    "Backup".green().bold()
  );
  println!(
    "\n{}\n",
    " - Backup all database contents to gzip compressed files."
      .dimmed()
  );
  if let Some(uri) = optional_string(&config.database.uri) {
    println!("{}: {}", " - Source URI".dimmed(), sanitize_uri(&uri));
  }
  if let Some(address) = optional_string(&config.database.address) {
    println!("{}: {address}", " - Source Address".dimmed());
  }
  if let Some(username) = optional_string(&config.database.username) {
    println!("{}: {username}", " - Source Username".dimmed());
  }
  println!(
    "{}: {}\n",
    " - Source Db Name".dimmed(),
    config.database.db_name,
  );
  println!(
    "{}: {:?}",
    " - Backups Folder".dimmed(),
    config.backups_folder
  );
  if config.max_backups == 0 {
    println!(
      "{}{}",
      " - Backup pruning".dimmed(),
      "disabled".red().dimmed()
    );
  } else {
    println!("{}: {}", " - Max Backups".dimmed(), config.max_backups);
  }

  crate::command::wait_for_enter("start backup", yes)?;

  let db = database::init(&config.database).await?;

  database::utils::backup(&db, &config.backups_folder).await?;

  // Early return if backup pruning disabled
  if config.max_backups == 0 {
    return Ok(());
  }

  // Know that new backup was taken successfully at this point,
  // safe to prune old backup folders

  prune_inner().await
}

async fn restore(
  restore_folder: Option<&Path>,
  index: bool,
  yes: bool,
) -> anyhow::Result<()> {
  let config = cli_config();

  println!(
    "\n🦎  {} Database {} Utility  🦎",
    "Komodo".bold(),
    "Restore".purple().bold()
  );
  println!(
    "\n{}\n",
    " - Restores database contents from gzip compressed files."
      .dimmed()
  );
  if let Some(uri) = optional_string(&config.database_target.uri) {
    println!("{}: {}", " - Target URI".dimmed(), sanitize_uri(&uri));
  }
  if let Some(address) =
    optional_string(&config.database_target.address)
  {
    println!("{}: {address}", " - Target Address".dimmed());
  }
  if let Some(username) =
    optional_string(&config.database_target.username)
  {
    println!("{}: {username}", " - Target Username".dimmed());
  }
  println!(
    "{}: {}",
    " - Target Db Name".dimmed(),
    config.database_target.db_name,
  );
  if !index {
    println!(
      "{}: {}",
      " - Target Db Indexing".dimmed(),
      "DISABLED".red(),
    );
  }
  println!(
    "\n{}: {:?}",
    " - Backups Folder".dimmed(),
    config.backups_folder
  );
  if let Some(restore_folder) = restore_folder {
    println!("{}: {restore_folder:?}", " - Restore Folder".dimmed());
  }

  crate::command::wait_for_enter("start restore", yes)?;

  let db = if index {
    database::Client::new(&config.database_target).await?.db
  } else {
    database::init(&config.database_target).await?
  };

  database::utils::restore(
    &db,
    &config.backups_folder,
    restore_folder,
  )
  .await
}

async fn prune(yes: bool) -> anyhow::Result<()> {
  let config = cli_config();

  println!(
    "\n🦎  {} Database {} Utility  🦎",
    "Komodo".bold(),
    "Backup Prune".cyan().bold()
  );
  println!(
    "\n{}\n",
    " - Prunes database backup folders when greater than the configured amount."
      .dimmed()
  );
  println!(
    "{}: {:?}",
    " - Backups Folder".dimmed(),
    config.backups_folder
  );
  if config.max_backups == 0 {
    println!(
      "{}{}",
      " - Backup pruning".dimmed(),
      "disabled".red().dimmed()
    );
  } else {
    println!("{}: {}", " - Max Backups".dimmed(), config.max_backups);
  }

  // Early return if backup pruning disabled
  if config.max_backups == 0 {
    info!(
      "Backup pruning is disabled, enabled using 'max_backups' (KOMODO_CLI_MAX_BACKUPS)"
    );
    return Ok(());
  }

  crate::command::wait_for_enter("start backup prune", yes)?;

  prune_inner().await
}

async fn prune_inner() -> anyhow::Result<()> {
  let config = cli_config();

  let mut backups_dir =
    match tokio::fs::read_dir(&config.backups_folder)
      .await
      .context("Failed to read backups folder for prune")
    {
      Ok(backups_dir) => backups_dir,
      Err(e) => {
        warn!("{e:#}");
        return Ok(());
      }
    };

  let mut backup_folders = Vec::new();
  loop {
    match backups_dir.next_entry().await {
      Ok(Some(entry)) => {
        let Ok(metadata) = entry.metadata().await else {
          continue;
        };
        if metadata.is_dir() {
          backup_folders.push(entry.path());
        }
      }
      Ok(None) => break,
      Err(_) => {
        continue;
      }
    }
  }
  // Ordered from oldest -> newest
  backup_folders.sort();

  let max_backups = config.max_backups as usize;
  let backup_folders_len = backup_folders.len();

  // Early return if under the backup count threshold
  if backup_folders_len <= max_backups {
    info!("No backups to prune");
    return Ok(());
  }

  let to_delete =
    &backup_folders[..(backup_folders_len - max_backups)];

  info!("Pruning old backups: {to_delete:?}");

  for path in to_delete {
    if let Err(e) =
      tokio::fs::remove_dir_all(path).await.with_context(|| {
        format!("Failed to delete backup folder at {path:?}")
      })
    {
      warn!("{e:#}");
    }
  }

  Ok(())
}

async fn copy(index: bool, yes: bool) -> anyhow::Result<()> {
  let config = cli_config();

  println!(
    "\n🦎  {} Database {} Utility  🦎",
    "Komodo".bold(),
    "Copy".blue().bold()
  );
  println!(
    "\n{}\n",
    " - Copies database contents to another database.".dimmed()
  );

  if let Some(uri) = optional_string(&config.database.uri) {
    println!("{}: {}", " - Source URI".dimmed(), sanitize_uri(&uri));
  }
  if let Some(address) = optional_string(&config.database.address) {
    println!("{}: {address}", " - Source Address".dimmed());
  }
  if let Some(username) = optional_string(&config.database.username) {
    println!("{}: {username}", " - Source Username".dimmed());
  }
  println!(
    "{}: {}\n",
    " - Source Db Name".dimmed(),
    config.database.db_name,
  );

  if let Some(uri) = optional_string(&config.database_target.uri) {
    println!("{}: {}", " - Target URI".dimmed(), sanitize_uri(&uri));
  }
  if let Some(address) =
    optional_string(&config.database_target.address)
  {
    println!("{}: {address}", " - Target Address".dimmed());
  }
  if let Some(username) =
    optional_string(&config.database_target.username)
  {
    println!("{}: {username}", " - Target Username".dimmed());
  }
  println!(
    "{}: {}",
    " - Target Db Name".dimmed(),
    config.database_target.db_name,
  );
  if !index {
    println!(
      "{}: {}",
      " - Target Db Indexing".dimmed(),
      "DISABLED".red(),
    );
  }

  crate::command::wait_for_enter("start copy", yes)?;

  let source_db = database::init(&config.database).await?;
  let target_db = if index {
    database::Client::new(&config.database_target).await?.db
  } else {
    database::init(&config.database_target).await?
  };

  database::utils::copy(&source_db, &target_db).await
}

async fn v1_downgrade(yes: bool) -> anyhow::Result<()> {
  let config = cli_config();

  println!(
    "\n🦎  {} Database {}  🦎",
    "Komodo".bold(),
    "V1 Downgrade".purple().bold()
  );
  println!(
    "\n{}\n",
    " - Downgrade the database to V1 compatible data structures."
      .dimmed()
  );
  if let Some(uri) = optional_string(&config.database.uri) {
    println!("{}: {}", " - URI".dimmed(), sanitize_uri(&uri));
  }
  if let Some(address) = optional_string(&config.database.address) {
    println!("{}: {address}", " - Address".dimmed());
  }
  if let Some(username) = optional_string(&config.database.username) {
    println!("{}: {username}", " - Username".dimmed());
  }
  println!(
    "{}: {}\n",
    " - Db Name".dimmed(),
    config.database.db_name,
  );

  crate::command::wait_for_enter("run downgrade", yes)?;

  let db = database::init(&config.database).await?;

  db.collection::<Document>("Server")
    .update_many(doc! {}, doc! { "$set": { "info": null } })
    .await
    .context("Failed to downgrade Server schema")?;

  db.collection::<Document>("Deployment")
    .update_many(doc! {}, doc! { "$set": { "info": null } })
    .await
    .context("Failed to downgrade Deployment schema")?;

  info!(
    "V1 Downgrade complete. Ready to downgrade to komodo-core:1 ✅"
  );

  Ok(())
}
