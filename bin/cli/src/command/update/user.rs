use anyhow::Context;
use colored::Colorize;
use database::mungos::mongodb::bson::doc;
use komodo_client::entities::{
  config::{
    cli::args::{CliEnabled, update::UpdateUserCommand},
    empty_or_redacted,
  },
  optional_string,
};

use crate::{command::sanitize_uri, config::cli_config};

pub async fn update(
  username: &str,
  command: &UpdateUserCommand,
) -> anyhow::Result<()> {
  match command {
    UpdateUserCommand::Password {
      password,
      unsanitized,
      yes,
    } => {
      update_password(username, password, *unsanitized, *yes).await
    }
    UpdateUserCommand::SuperAdmin { enabled, yes } => {
      update_super_admin(username, *enabled, *yes).await
    }
    UpdateUserCommand::Clear2fa { yes } => {
      clear_2fa(username, *yes).await
    }
  }
}

async fn update_password(
  username: &str,
  password: &str,
  unsanitized: bool,
  yes: bool,
) -> anyhow::Result<()> {
  println!("\n{}: Update Password\n", "Mode".dimmed());
  println!(" - {}: {username}", "Username".dimmed());
  if unsanitized {
    println!(" - {}: {password}", "Password".dimmed());
  } else {
    println!(
      " - {}: {}",
      "Password".dimmed(),
      empty_or_redacted(password)
    );
  }

  crate::command::wait_for_enter("update password", yes)?;

  info!("Updating password...");

  let db = database::Client::new(&cli_config().database).await?;

  let user = db
    .users
    .find_one(doc! { "username": username })
    .await
    .context("Failed to query database for user")?
    .context("No user found with given username")?;

  db.set_user_password(&user, password).await?;

  info!("Password updated ✅");

  Ok(())
}

async fn update_super_admin(
  username: &str,
  super_admin: CliEnabled,
  yes: bool,
) -> anyhow::Result<()> {
  let config = cli_config();

  println!("\n{}: Update Super Admin\n", "Mode".dimmed());
  println!(" - {}: {username}", "Username".dimmed());
  println!(" - {}: {super_admin}\n", "Super Admin".dimmed());

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
    "{}: {}",
    " - Source Db Name".dimmed(),
    config.database.db_name,
  );

  crate::command::wait_for_enter("update super admin", yes)?;

  info!("Updating super admin...");

  let db = database::Client::new(&config.database).await?;

  // Make sure the user exists first before saying it is successful.
  let user = db
    .users
    .find_one(doc! { "username": username })
    .await
    .context("Failed to query database for user")?
    .context("No user found with given username")?;

  let super_admin: bool = super_admin.into();
  db.users
    .update_one(
      doc! { "username": user.username },
      doc! { "$set": { "super_admin": super_admin } },
    )
    .await
    .context("Failed to update user super admin on db")?;

  info!("Super admin updated ✅");

  Ok(())
}

async fn clear_2fa(username: &str, yes: bool) -> anyhow::Result<()> {
  println!("\n{}: Clear 2FA Methods\n", "Mode".dimmed());
  println!(" - {}: {username}", "Username".dimmed());

  crate::command::wait_for_enter("clear user 2FA methods", yes)?;

  info!("Clearing 2FA methods...");

  let db = database::Client::new(&cli_config().database).await?;

  db.clear_user_2fa_methods(username).await?;

  info!("2FA methods cleared ✅");

  Ok(())
}
