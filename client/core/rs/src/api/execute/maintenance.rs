use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

use super::KomodoExecuteRequest;

/// **Admin only.** Clears all repos from the Core repo cache.
/// Response: [Update]
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct ClearRepoCache {}

//

/// **Admin only.** Backs up the Komodo Core database to compressed jsonl files.
/// Response: [Update]. Aliases: `backup-database`, `backup-db`, `backup`.
///
/// Mount a folder to `/backups`, and Core will use it to create
/// timestamped database dumps, which can be restored using
/// the Komodo CLI.
///
/// https://komo.do/docs/setup/backup
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct BackupCoreDatabase {}

//

/// **Admin only.** Trigger a global poll for image updates on Stacks and Deployments
/// with `poll_for_updates` or `auto_update` enabled.
/// Response: [Update]. Alias: `auto-update`.
///
/// 1. `docker compose pull` any Stacks / Deployments with `poll_for_updates` or `auto_update` enabled. This will pick up any available updates.
/// 2. Redeploy Stacks / Deployments that have updates found and 'auto_update' enabled.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct GlobalAutoUpdate {}

//

/// **Admin only.** Rotates all connected Server keys.
/// Response: [Update]. Alias: `rotate-keys`.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RotateAllServerKeys {}

//

/// **Admin only.** Rotates the Core private key,
/// and all Server public keys.
/// Response: [Update].
///
/// If any Server is `NotOk`, this will fail.
/// To proceed anyways, pass `force: true`.
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Serialize, Deserialize, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct RotateCoreKeys {
  /// Force the rotation to proceed even if a Server is `NotOk`.
  /// The Core Public Key in Periphery config may have to be updated manually.
  /// (alias: `f`)
  #[serde(default)]
  #[clap(long, short, alias = "f", default_value_t = false)]
  pub force: bool,
}
