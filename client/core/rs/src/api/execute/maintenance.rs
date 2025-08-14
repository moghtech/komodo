use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::KomodoExecuteRequest;

/// Clears all repos from the Core repo cache. Admin only.
/// Response: [Update]
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct ClearRepoCache {}

/// Backs up the Komodo Core database to compressed jsonl files.
/// Admin only. Response: [Update]
///
/// Mount a folder to `/backups`, and Core will use it to create
/// timestamped database dumps, which can be restored using
/// the Komodo CLI.
///
/// TODO: Link to docs
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct BackupCoreDatabase {}

/// Trigger a global poll for image updateson Stacks and Deployments
/// with `poll_for_updates` or `auto_update` enabled.
/// Admin only. Response: [Update]
///
/// 1. `docker compose pull` any Stacks / Deployments with `poll_for_updates` or `auto_update` enabled. This will pick up any available updates.
/// 2. Redeploy Stacks / Deployments that have updates found.
#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Serialize,
  Deserialize,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct GlobalAutoUpdate {}
