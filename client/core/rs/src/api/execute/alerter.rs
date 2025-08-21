use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::{alert::SeverityLevel, update::Update};

use super::KomodoExecuteRequest;

/// Tests an Alerters ability to reach the configured endpoint. Response: [Update]
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct TestAlerter {
  /// Name or id
  pub alerter: String,
}

//

/// Send a custom alert message to configured Alerters. Response: [Update]
#[typeshare]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  PartialEq,
  Resolve,
  EmptyTraits,
  Parser,
)]
#[empty_traits(KomodoExecuteRequest)]
#[response(Update)]
#[error(serror::Error)]
pub struct SendAlert {
  /// The alert level. Default: Ok
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
