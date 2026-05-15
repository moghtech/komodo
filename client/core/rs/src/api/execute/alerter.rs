use clap::Parser;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{alert::SeverityLevel, update::Update};

use super::KomodoExecuteRequest;

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/TestAlerter",
  description = "Tests an Alerter's ability to reach the configured endpoint.",
  request_body(content = TestAlerter),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn test_alerter() {}

/// Tests an Alerters ability to reach the configured endpoint. Response: [Update]
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct TestAlerter {
  /// Name or id
  pub alerter: String,
}

//

#[cfg(feature = "utoipa")]
#[utoipa::path(
  post,
  path = "/SendAlert",
  description = "Send a custom alert message to configured Alerters.",
  request_body(content = SendAlert),
  responses(
    (status = 200, description = "The update", body = Update),
  ),
)]
pub fn send_alert() {}

/// Send a custom alert message to configured Alerters. Response: [Update].
/// Alias: `alert`
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, PartialEq, Resolve, Parser,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(mogh_error::Error)]
pub struct SendAlert {
  /// The alert level.
  #[serde(default)]
  #[clap(long, short = 'l', default_value_t = SeverityLevel::Ok)]
  pub level: SeverityLevel,
  /// The alert message. Required.
  pub message: String,
  /// The alert details. Optional.
  #[serde(default)]
  #[arg(long, short = 'd', default_value_t = String::new())]
  pub details: String,
  /// Specific alerter names or ids.
  /// If empty / not passed, sends to all configured alerters
  /// with the `Custom` alert type whitelisted / not blacklisted.
  #[serde(default)]
  #[arg(long, short = 'a')]
  pub alerters: Vec<String>,
}
