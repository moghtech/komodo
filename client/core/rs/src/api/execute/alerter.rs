use clap::Parser;
use derive_empty_traits::EmptyTraits;
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::entities::update::Update;

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
